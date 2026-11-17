//! Logging in every demo user, shared by seed, wipe and status.

use anyhow::{Result, bail};

use crate::api::{Login, Session, User};
use crate::config::Config;
use crate::fixtures::users::{DemoUser, USERS};

pub struct Account<'a> {
    pub user: &'static DemoUser,
    pub session: Session<'a>,
    /// None: the account does not exist (yet), or it is suspended and cannot log in.
    pub account: Option<User>,
    pub suspended: bool,
    pub created: bool,
}

/// Logs every demo user in. With `create`, registers the missing ones first (unless `dry_run`).
pub async fn log_in_all(config: &Config, create: bool, dry_run: bool) -> Result<Vec<Account<'_>>> {
    let mut accounts = Vec::new();
    for user in USERS {
        let session = Session::new(config)?;
        let email = user.email();
        let mut login = session.login(&email).await?;
        let mut created = false;
        if matches!(login, Login::Missing) && create && !dry_run {
            if !session.register(user.username, &email).await? {
                bail!(
                    "{} or {email} is taken, but the demo password does not work for it: \
                     either DEMO_PASSWORD changed or a real user has this name",
                    user.username
                );
            }
            created = true;
            login = session.login(&email).await?;
        }
        let (account, suspended) = match login {
            Login::Ok(account) => (Some(account), false),
            Login::Suspended => (None, true),
            Login::Missing => (None, false),
        };
        accounts.push(Account {
            user,
            session,
            account,
            suspended,
            created,
        });
    }
    Ok(accounts)
}
