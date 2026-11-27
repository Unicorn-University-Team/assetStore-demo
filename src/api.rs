//! A session against the site's public API. Each demo user gets its own session (its own cookies), because
//! identity keeps the login in httpOnly cookies.

use anyhow::{Context, Result, bail};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;
use crate::fixtures::assets::DemoAsset;

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
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub name: String,
    /// Tag names, sorted by the catalog.
    #[serde(default)]
    pub tag_list: Vec<String>,
    /// null for an asset uploaded before the catalog required a preview image.
    #[serde(default)]
    pub preview_url: Option<String>,
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

    /// Publishes a demo asset: its file and the preview image the catalog requires.
    pub async fn upload(&self, asset: &DemoAsset) -> Result<Asset> {
        let name = asset.name;
        let file = Part::bytes(asset.bytes())
            .file_name(asset.file_name())
            .mime_str(asset.content_type())?;
        // The catalog requires a preview image (png/jpg/webp, at most 2 MB) for every asset.
        let preview = Part::bytes(asset.preview())
            .file_name("preview.png")
            .mime_str("image/png")?;
        // Text fields first, the files last (the catalog reads the stream in order).
        let mut form = Form::new()
            .text("name", name.to_string())
            .text("description", asset.description())
            .text("type", asset.kind.to_string());
        // One field per tag; every tag must exist already (create_tag).
        for tag in asset.tags {
            form = form.text("tagList", tag.to_string());
        }
        let form = form.part("file", file).part("preview", preview);
        let response = self
            .request(self.client.post(self.url("/assets")).multipart(form))
            .send()
            .await?;
        Ok(ok(response, &format!("upload {name}"))
            .await?
            .json()
            .await?)
    }

    /// Creates the tag (POST /api/tags); false when it exists already (409).
    pub async fn create_tag(&self, name: &str) -> Result<bool> {
        let response = self
            .request(
                self.client
                    .post(self.url("/tags"))
                    .json(&json!({ "name": name })),
            )
            .send()
            .await?;
        if response.status() == StatusCode::CONFLICT {
            return Ok(false);
        }
        ok(response, &format!("create tag {name}")).await?;
        Ok(true)
    }

    /// Brings an asset seeded by an older version up to date (PUT /api/assets/:id; description and file stay):
    /// `tags` replaces its tags with the fixture's, `preview` adds the preview image it is missing.
    pub async fn update_existing(
        &self,
        asset: &Asset,
        fixture: &DemoAsset,
        tags: bool,
        preview: bool,
    ) -> Result<()> {
        let url = self.url(&format!("/assets/{}", asset.id));
        let builder = if preview {
            // A new preview needs multipart; tagList then goes as one JSON array field.
            let mut form = Form::new().text("name", asset.name.clone());
            if tags {
                form = form.text("tagList", serde_json::to_string(fixture.tags)?);
            }
            let image = Part::bytes(fixture.preview())
                .file_name("preview.png")
                .mime_str("image/png")?;
            self.client.put(url).multipart(form.part("preview", image))
        } else {
            self.client
                .put(url)
                .json(&json!({ "name": asset.name, "tagList": fixture.tags }))
        };
        let response = self.request(builder).send().await?;
        ok(response, &format!("update {}", asset.name))
            .await
            .map(|_| ())
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

    pub async fn report(&self, asset_id: &str, reason: &str) -> Result<Report> {
        let body = json!({ "reason": reason });
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

    /// The asset page data answers, its preview image answers, and its file gives the first bytes (a ranged request,
    /// so nothing is downloaded and the download count stays). `no_preview`: a file type the site cannot show
    /// (the file must answer 404).
    pub async fn check_asset(&self, id: &str, no_preview: bool) -> Result<()> {
        let detail = self
            .request(self.client.get(self.url(&format!("/assets/{id}"))))
            .send()
            .await?;
        ok(detail, "detail").await?;
        let image = self
            .request(self.client.get(self.url(&format!("/assets/{id}/preview"))))
            .send()
            .await?;
        let image = ok(image, "preview image").await?;
        let is_image = image
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("image/"));
        if !is_image {
            bail!("obrázek náhledu nemá typ image/*");
        }
        let preview = self
            .request(
                self.client
                    .get(self.url(&format!("/assets/{id}/file")))
                    .header("Range", "bytes=0-15"),
            )
            .send()
            .await?;
        match (no_preview, preview.status()) {
            (false, StatusCode::PARTIAL_CONTENT) | (true, StatusCode::NOT_FOUND) => Ok(()),
            (_, status) => bail!("náhled odpověděl {status}"),
        }
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
