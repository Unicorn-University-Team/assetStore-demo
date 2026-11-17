use anyhow::Result;

use crate::api::Session;
use crate::config::Config;
use crate::fixtures::assets::ASSETS;
use crate::summary::Summary;
use crate::users::log_in_all;

pub async fn check_services(config: &Config) -> Result<()> {
    let probe = Session::new(config)?;
    for service in ["identity", "catalog", "community"] {
        probe.check_health(service).await?;
    }
    println!("{}: identity, catalog and community are up", config.api_url);
    Ok(())
}

/// Which demo users exist and how many assets each one owns.
pub async fn run(config: &Config) -> Result<Summary> {
    check_services(config).await?;
    let accounts = log_in_all(config, false, true).await?;
    let mut summary = Summary::new("Demo data: status");
    let mut assets = 0;
    for account in &accounts {
        let state = match &account.account {
            Some(owner) => {
                let count = account.session.assets_of(&owner.id).await?.len();
                assets += count;
                format!("{count} assets")
            }
            None if account.suspended => "suspended".to_string(),
            None => "missing".to_string(),
        };
        println!(
            "  {:<16} {:<10} {state}",
            account.user.username,
            format!("{:?}", account.user.kind)
        );
    }
    let existing = accounts
        .iter()
        .filter(|account| account.account.is_some() || account.suspended)
        .count();
    summary.count("Demo uživatelé", existing);
    summary.count("Demo assety", assets);
    summary.notes.push(format!(
        "Plný stav: {} uživatelů a {} assetů.",
        accounts.len(),
        ASSETS.len()
    ));
    Ok(summary)
}
