//! A logged-in session against the site's API. Each demo user gets its own session (its own cookies),
//! because identity keeps the login in httpOnly cookies.

use anyhow::{Context, Result, bail};
use reqwest::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;

pub struct Session<'a> {
    config: &'a Config,
    client: Client,
}

#[derive(Deserialize)]
pub struct User {
    pub id: String,
}

#[derive(Deserialize)]
struct UserBody {
    user: User,
}

#[derive(Deserialize)]
struct PageInfo {
    total: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetList {
    page_info: PageInfo,
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

    /// Logs in; None when the account does not exist or the password is wrong (401).
    pub async fn login(&self, email: &str) -> Result<Option<User>> {
        let body = json!({ "email": email, "password": self.config.demo_password });
        let response = self
            .request(self.client.post(self.url("/auth/login")).json(&body))
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(None);
        }
        ok(response, "login").await?;
        let me = self
            .request(self.client.get(self.url("/auth/me")))
            .send()
            .await?;
        Ok(Some(
            ok(me, "auth/me").await?.json::<UserBody>().await?.user,
        ))
    }

    /// How many assets the user owns (GET /api/assets?ownerId=).
    pub async fn asset_count(&self, owner_id: &str) -> Result<u64> {
        let url = self.url("/assets");
        let query = [("ownerId", owner_id), ("pageSize", "1")];
        let response = self
            .request(self.client.get(url).query(&query))
            .send()
            .await?;
        Ok(ok(response, "assets")
            .await?
            .json::<AssetList>()
            .await?
            .page_info
            .total)
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
