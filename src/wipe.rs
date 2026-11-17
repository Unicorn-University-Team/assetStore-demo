//! wipe: deletes the demo assets through the API (the catalog removes the files from S3 too), then writes SQL that
//! deletes the demo reports and accounts. Touches nothing outside the demo e-mail domain.

use anyhow::Result;

use crate::config::Config;
use crate::fixtures::users::DEMO_EMAIL_DOMAIN;
use crate::sql::{SqlFile, quote, uuid};
use crate::summary::Summary;
use crate::users::log_in_all;

pub async fn run(config: &Config, dry_run: bool, sql: &mut SqlFile) -> Result<Summary> {
    crate::status::check_services(config).await?;
    let accounts = log_in_all(config, false, dry_run).await?;
    let mut summary = Summary::new(if dry_run {
        "Demo data: wipe (dry run, nothing changed)"
    } else {
        "Demo data: wipe"
    });

    let mut deleted = 0;
    let mut ids = Vec::new();
    for account in &accounts {
        let Some(owner) = &account.account else {
            continue;
        };
        ids.push(uuid(&owner.id)?);
        for asset in account.session.assets_of(&owner.id).await? {
            if !dry_run {
                account.session.delete_asset(&asset.id).await?;
                println!("deleted {} / {}", account.user.username, asset.name);
            }
            deleted += 1;
        }
    }
    summary.count("Smazané assety", deleted);
    summary.count(
        "Demo účty ke smazání",
        accounts
            .iter()
            .filter(|account| account.account.is_some() || account.suspended)
            .count(),
    );

    if !ids.is_empty() {
        let list = ids.join(", ");
        sql.add(
            "community",
            format!("DELETE FROM report WHERE author_id IN ({list}) OR asset_owner_id IN ({list})"),
        );
    }
    // Sessions go with the accounts (ON DELETE CASCADE).
    sql.add(
        "identity",
        format!(
            "DELETE FROM user_account WHERE email LIKE {}",
            quote(&format!("%@{DEMO_EMAIL_DOMAIN}"))
        ),
    );
    if accounts.iter().any(|account| account.suspended) {
        summary.notes.push(
            "Zablokovaný demo účet se nedá přihlásit: jeho reporty (pokud nějaké má) SQL nesmaže."
                .into(),
        );
    }
    Ok(summary)
}
