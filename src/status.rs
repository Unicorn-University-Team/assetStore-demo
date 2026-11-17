use anyhow::Result;

use crate::api::Session;
use crate::config::Config;
use crate::fixtures::USERS;

/// Which demo users exist and how many assets each one owns.
pub async fn run(config: &Config) -> Result<()> {
    let probe = Session::new(config)?;
    for service in ["identity", "catalog", "community"] {
        probe.check_health(service).await?;
    }
    println!("{}: identity, catalog and community are up", config.api_url);

    let (mut users, mut assets) = (0, 0);
    for user in USERS {
        let session = Session::new(config)?;
        match session.login(&user.email()).await? {
            Some(account) => {
                let count = session.asset_count(&account.id).await?;
                println!(
                    "  {:<16} {:<10} {count} assets",
                    user.username,
                    format!("{:?}", user.kind)
                );
                users += 1;
                assets += count;
            }
            None => println!(
                "  {:<16} {:<10} missing",
                user.username,
                format!("{:?}", user.kind)
            ),
        }
    }
    println!(
        "demo users: {users} of {}, demo assets: {assets}",
        USERS.len()
    );
    Ok(())
}
