//! status: who of the demo users exists and how many assets they own.
//! check: the same, plus every showcase asset must answer on the site (detail and preview), else the run fails.

use std::collections::HashMap;

use anyhow::Result;

use crate::api::Session;
use crate::config::Config;
use crate::fixtures::assets::{ASSETS, SHOWCASE};
use crate::fixtures::users::USERS;
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

/// Links to the showcase assets that exist: (owner, name) -> id.
pub fn showcase_links(
    config: &Config,
    ids: &HashMap<(String, String), String>,
    summary: &mut Summary,
) {
    for (icon, owner, name, hint) in SHOWCASE {
        if let Some(id) = ids.get(&(owner.to_string(), name.to_string())) {
            summary.links.push((
                icon,
                name.to_string(),
                hint,
                format!("{}/assets/{id}", config.site_url()),
            ));
        }
    }
}

pub async fn run(config: &Config, deep: bool) -> Result<Summary> {
    let mut summary = Summary::new(if deep { "check" } else { "status" }, false);
    if let Err(error) = check_services(config).await {
        // Only check reports a down service as a result; status just fails.
        if !deep {
            return Err(error);
        }
        summary
            .problems
            .push(format!("Služby neodpovídají: {error:#}"));
        return Ok(summary);
    }

    let accounts = log_in_all(config, false, true).await?;
    let mut ids = HashMap::new();
    let mut assets = 0;
    // Demo assets whose tags differ from the fixtures (seeded before tags, or changed on the site).
    let mut untagged = 0;
    for account in &accounts {
        let state = match &account.account {
            Some(owner) => {
                let owned = account.session.assets_of(&owner.id).await?;
                assets += owned.len();
                for asset in &owned {
                    let fixture = ASSETS.iter().find(|fixture| {
                        fixture.owner == account.user.username && fixture.name == asset.name
                    });
                    if let Some(fixture) = fixture {
                        let mut wanted: Vec<&str> = fixture.tags.to_vec();
                        wanted.sort_unstable();
                        if asset.tag_list != wanted {
                            untagged += 1;
                        }
                    }
                }
                for asset in &owned {
                    ids.insert(
                        (account.user.username.to_string(), asset.name.clone()),
                        asset.id.clone(),
                    );
                }
                format!("{} assetů", owned.len())
            }
            None if account.suspended => "🚫 zablokovaný".to_string(),
            None => "— neexistuje".to_string(),
        };
        println!(
            "  {:<16} {:<10} {state}",
            account.user.username,
            format!("{:?}", account.user.kind)
        );
        summary.users.push((
            account.user.username.to_string(),
            format!("{:?}", account.user.kind),
            state,
        ));
    }
    let existing = accounts
        .iter()
        .filter(|account| account.account.is_some() || account.suspended)
        .count();
    summary.count("👤", "Demo uživatelé", existing);
    summary.count("📦", "Demo assety", assets);
    if existing == 0 {
        summary
            .notes
            .push("Demo data nejsou na webu. Vytvoří je `seed`.".into());
    } else if untagged > 0 {
        summary.notes.push(format!(
            "{untagged} demo assetů nemá tagy podle fixtur. Doplní je `seed`."
        ));
    }
    if existing > 0 && (existing < USERS.len() || assets < ASSETS.len()) {
        summary.notes.push(format!(
            "Neúplný stav (plný je {} účtů a {} assetů). Doplní ho `seed`.",
            USERS.len(),
            ASSETS.len()
        ));
    }
    showcase_links(config, &ids, &mut summary);

    if deep {
        let probe = Session::new(config)?;
        let mut checked = 0;
        for (_, owner, name, _) in SHOWCASE {
            let Some(id) = ids.get(&(owner.to_string(), name.to_string())) else {
                continue;
            };
            if let Err(error) = probe.check_asset(id, name.ends_with("valley")).await {
                summary.problems.push(format!("{name}: {error:#}"));
            }
            checked += 1;
        }
        summary.count(
            "🩺",
            "Zkontrolované ukázkové assety (detail + náhled)",
            checked,
        );
    }
    Ok(summary)
}
