//! A session against the site's public API. Each demo user gets its own session (its own cookies), because
//! identity keeps the login in httpOnly cookies.

use anyhow::{Context, Result, bail};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;

pub struct Session<'a> {
    config: &'a Config,
    client: Client,
}

#[derive(Deserialize, Clone)]
pub struct User {
    pub id: String,
}

#[derive(Deserialize)]
struct UserBody {
    user: User,
}

#[derive(Deserialize, Clone)]
pub struct Asset {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    total: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetList {
    item_list: Vec<Asset>,
    page_info: PageInfo,
}

#[derive(Deserialize)]
pub struct Report {
    pub id: String,
}

pub enum Login {
    Ok(User),
    /// No such account, or a different password.
    Missing,
    /// The account exists but is blocked (403).
    Suspended,
}

impl<'a> Session<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent("assetstore-demo")
            .build()?;
        Ok(Self { config, client })
    }

    fn request(&self, builder: RequestBuilder) -> RequestBuilder {
        match &self.config.site_auth {
            Some((user, password)) => builder.basic_auth(user, Some(password)),
            None => builder,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.api_url, path)
    }

    pub async fn login(&self, email: &str) -> Result<Login> {
        let body = json!({ "email": email, "password": self.config.demo_password });
        let response = self
            .request(self.client.post(self.url("/auth/login")).json(&body))
            .send()
            .await?;
        match response.status() {
            StatusCode::UNAUTHORIZED => return Ok(Login::Missing),
            StatusCode::FORBIDDEN => return Ok(Login::Suspended),
            _ => {}
        }
        ok(response, "login").await?;
        let me = self
            .request(self.client.get(self.url("/auth/me")))
            .send()
            .await?;
        Ok(Login::Ok(
            ok(me, "auth/me").await?.json::<UserBody>().await?.user,
        ))
    }

    /// Creates the account; false when the e-mail or the name is taken already (409).
    pub async fn register(&self, username: &str, email: &str) -> Result<bool> {
        let body =
            json!({ "username": username, "email": email, "password": self.config.demo_password });
        let response = self
            .request(self.client.post(self.url("/auth/register")).json(&body))
            .send()
            .await?;
        if response.status() == StatusCode::CONFLICT {
            return Ok(false);
        }
        ok(response, &format!("register {username}")).await?;
        Ok(true)
    }

    /// All assets of one owner (GET /api/assets?ownerId=, every page).
    pub async fn assets_of(&self, owner_id: &str) -> Result<Vec<Asset>> {
        let mut all = Vec::new();
        for page in 0.. {
            let query = [
                ("ownerId", owner_id.to_string()),
                ("pageIndex", page.to_string()),
                ("pageSize", "100".to_string()),
            ];
            let response = self
                .request(self.client.get(self.url("/assets")).query(&query))
                .send()
                .await?;
            let list = ok(response, "assets").await?.json::<AssetList>().await?;
            let done = list.item_list.is_empty()
                || all.len() + list.item_list.len() >= list.page_info.total as usize;
            all.extend(list.item_list);
            if done {
                break;
            }
        }
        Ok(all)
    }

    pub async fn upload(
        &self,
        name: &str,
        description: &str,
        kind: &str,
        file_name: String,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<Asset> {
        let file = Part::bytes(bytes)
            .file_name(file_name)
            .mime_str(content_type)?;
        // Text fields first, the file last (the catalog reads the stream in order).
        let form = Form::new()
            .text("name", name.to_string())
            .text("description", description.to_string())
            .text("type", kind.to_string())
            .part("file", file);
        let response = self
            .request(self.client.post(self.url("/assets")).multipart(form))
            .send()
            .await?;
        Ok(ok(response, &format!("upload {name}"))
            .await?
            .json()
            .await?)
    }

    pub async fn delete_asset(&self, id: &str) -> Result<()> {
        let response = self
            .request(self.client.delete(self.url(&format!("/assets/{id}"))))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        ok(response, &format!("delete asset {id}"))
            .await
            .map(|_| ())
    }

    pub async fn report(&self, asset_id: &str, reason: &str, comment: &str) -> Result<Report> {
        let body = json!({ "reason": reason, "comment": comment });
        let response = self
            .request(
                self.client
                    .post(self.url(&format!("/assets/{asset_id}/reports")))
                    .json(&body),
            )
            .send()
            .await?;
        Ok(ok(response, "report").await?.json().await?)
    }

    /// Fails with a readable message when a service is down.
    pub async fn check_health(&self, service: &str) -> Result<()> {
        let response = self
            .request(self.client.get(self.url(&format!("/health/{service}"))))
            .send()
            .await;
        let response =
            response.with_context(|| format!("{} is not reachable", self.config.api_url))?;
        ok(response, &format!("health of {service}"))
            .await
            .map(|_| ())
    }
}

/// The response when it is 2xx, otherwise an error with the status and the start of the body.
async fn ok(response: Response, what: &str) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    bail!(
        "{what}: {status} {}",
        body.chars().take(200).collect::<String>()
    )
}
