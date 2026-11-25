//! seed: demo users, assets with real files and reports, through the public API. What the API cannot do goes to
//! the SQL file (sql.rs).

use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::fixtures::assets::ASSETS;
use crate::fixtures::reports::REPORTS;
use crate::fixtures::users::Kind;
use crate::sql::{SqlFile, quote, uuid};
use crate::summary::Summary;
use crate::users::log_in_all;

pub async fn run(config: &Config, dry_run: bool, sql: &mut SqlFile) -> Result<Summary> {
    crate::status::check_services(config).await?;
    let accounts = log_in_all(config, true, dry_run).await?;
    let mut summary = Summary::new("seed", dry_run);
    summary.show_logins = true;

    // Roles and blocked accounts: only SQL can set them. By e-mail, so it works before the first login too.
    for account in &accounts {
        let email = quote(&account.user.email());
        match account.user.kind {
            Kind::Admin => sql.add(
                "identity",
                format!("UPDATE user_account SET role = 'ADMIN' WHERE email = {email}"),
            ),
            Kind::Suspended => sql.add(
                "identity",
                format!("UPDATE user_account SET status = 'SUSPENDED' WHERE email = {email}"),
            ),
            _ => {}
        }
    }
    let created_users = accounts.iter().filter(|account| account.created).count();
    summary.count("👤", "Nově vytvoření uživatelé", created_users);
    let existing = accounts
        .iter()
        .filter(|account| account.account.is_some() || account.suspended)
        .count();
    summary.count(
        "👥",
        "Uživatelé, kteří už existovali",
        existing - created_users,
    );

    // Assets: skip the ones the owner already has (same name), upload the rest.
    let mut asset_ids: HashMap<(&str, &str), String> = HashMap::new();
    let mut new_assets = Vec::new();
    for account in &accounts {
        let Some(owner) = &account.account else {
            continue;
        };
        let existing = account.session.assets_of(&owner.id).await?;
        for asset in ASSETS
            .iter()
            .filter(|asset| asset.owner == account.user.username)
        {
            if let Some(found) = existing.iter().find(|item| item.name == asset.name) {
                asset_ids.insert((asset.owner, asset.name), found.id.clone());
                continue;
            }
            if dry_run {
                println!("would upload {} / {}", asset.owner, asset.name);
                new_assets.push((asset.owner, asset.name));
                continue;
            }
            let uploaded = account.session.upload(asset).await?;
            println!("uploaded {} / {}", asset.owner, asset.name);
            asset_ids.insert((asset.owner, asset.name), uploaded.id);
            new_assets.push((asset.owner, asset.name));
        }
    }
    summary.count("📦", "Nahrané assety", new_assets.len());
    let uploaded = if dry_run { 0 } else { new_assets.len() };
    summary.count(
        "🗃️",
        "Assety, které už existovaly",
        asset_ids.len() - uploaded,
    );

    // Publication dates spread over the last weeks and download counts (never lower than the real ones).
    for asset in ASSETS {
        let Some(id) = asset_ids.get(&(asset.owner, asset.name)) else {
            continue;
        };
        sql.add(
            "catalog",
            format!(
                "UPDATE asset SET created_at = now() - interval '{} days', updated_at = now() - interval '{} days', \
                 download_count = GREATEST(download_count, {}) WHERE id = {}",
                asset.days_ago,
                asset.days_ago,
                asset.downloads,
                uuid(id)?
            ),
        );
    }

    // Reports only on assets uploaded in this run, so running seed again never reports twice.
    let admin = accounts
        .iter()
        .find(|account| account.user.kind == Kind::Admin)
        .and_then(|account| account.account.clone());
    let mut reports = 0;
    for report in REPORTS {
        if !new_assets.contains(&(report.asset_owner, report.asset_name)) {
            continue;
        }
        reports += 1;
        if dry_run {
            continue;
        }
        let author = accounts
            .iter()
            .find(|account| account.user.username == report.author)
            .context("report author")?;
        let asset_id = asset_ids
            .get(&(report.asset_owner, report.asset_name))
            .context("reported asset")?;
        let created = author.session.report(asset_id, report.reason).await?;
        if report.rejected
            && let Some(admin) = &admin
        {
            sql.add(
                "community",
                format!(
                    "UPDATE report SET status = 'rejected', decided_by = {}, decided_at = now() \
                     WHERE id = {} AND status = 'open'",
                    uuid(&admin.id)?,
                    uuid(&created.id)?
                ),
            );
        }
    }
    summary.count("🚩", "Nové reporty", reports);
    let ids = asset_ids
        .iter()
        .map(|((owner, name), id)| ((owner.to_string(), name.to_string()), id.clone()))
        .collect();
    crate::status::showcase_links(config, &ids, &mut summary);
    Ok(summary)
}
