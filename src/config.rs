use anyhow::{Context, Result, bail};

/// Settings from the environment (.env.example lists them).
pub struct Config {
    /// The site's API without a trailing slash, e.g. https://3-125-15-78.sslip.io/api
    pub api_url: String,
    pub demo_password: String,
    /// Caddy basic auth when the site runs with SITE_AUTH=on.
    pub site_auth: Option<(String, String)>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("API_URL").context("API_URL is not set (see .env.example)")?;
        let demo_password = std::env::var("DEMO_PASSWORD").unwrap_or_default();
        // The same rule as registration in identity: at least 8 characters.
        if demo_password.chars().count() < 8 {
            bail!("DEMO_PASSWORD must have at least 8 characters (see .env.example)");
        }
        let site_auth = match (non_empty("SITE_AUTH_USER"), non_empty("SITE_AUTH_PASSWORD")) {
            (Some(user), Some(password)) => Some((user, password)),
            (None, None) => None,
            _ => bail!("set both SITE_AUTH_USER and SITE_AUTH_PASSWORD, or neither"),
        };
        Ok(Self {
            api_url: api_url.trim_end_matches('/').to_string(),
            demo_password,
            site_auth,
        })
    }

    /// The site itself: the API URL without its /api.
    pub fn site_url(&self) -> &str {
        self.api_url.strip_suffix("/api").unwrap_or(&self.api_url)
    }

    /// localhost and 127.0.0.1 are local; anything else needs --yes before data changes.
    pub fn is_local(&self) -> bool {
        let host = self
            .api_url
            .split("://")
            .nth(1)
            .unwrap_or("")
            .split(['/', ':'])
            .next()
            .unwrap_or("");
        matches!(host, "localhost" | "127.0.0.1")
    }
}

fn non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::Config;

    fn config(api_url: &str) -> Config {
        Config {
            api_url: api_url.into(),
            demo_password: "password1".into(),
            site_auth: None,
        }
    }

    #[test]
    fn only_localhost_is_local() {
        assert!(config("http://localhost:5173/api").is_local());
        assert!(config("http://127.0.0.1/api").is_local());
        assert!(!config("https://3-125-15-78.sslip.io/api").is_local());
        assert!(!config("https://localhost.example.com/api").is_local());
    }
}
