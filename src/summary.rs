//! What a run did: printed to the terminal and, with --summary, written as Markdown for the GitHub job summary.

use std::fmt::Write;

use crate::config::Config;
use crate::fixtures::users::{Kind, USERS};

#[derive(Default)]
pub struct Summary {
    pub title: String,
    /// (what, count)
    pub counts: Vec<(String, usize)>,
    pub notes: Vec<String>,
}

impl Summary {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }

    pub fn count(&mut self, what: &str, count: usize) {
        self.counts.push((what.to_string(), count));
    }

    pub fn print(&self) {
        println!("\n{}", self.title);
        for (what, count) in &self.counts {
            println!("  {what}: {count}");
        }
        for note in &self.notes {
            println!("  {note}");
        }
    }

    /// Markdown with the results, the site and the demo logins (never the password).
    pub fn markdown(&self, config: &Config, with_logins: bool) -> String {
        let site = config.api_url.trim_end_matches("/api");
        let mut out = format!("## {}\n\n| | |\n|---|---:|\n", self.title);
        for (what, count) in &self.counts {
            let _ = writeln!(out, "| {what} | {count} |");
        }
        for note in &self.notes {
            let _ = write!(out, "\n> {note}\n");
        }
        let _ = write!(out, "\n**Web:** {site}  \n**Přihlášení:** {site}/login\n");
        if with_logins {
            out.push_str("\n### Demo účty\n\nHeslo všech účtů je v secretu `DEMO_PASSWORD` (zná ho Yura).\n\n| Účet | E-mail | Co ukazuje |\n|---|---|---|\n");
            for user in USERS {
                let role = match user.kind {
                    Kind::Author => "autor assetů",
                    Kind::Visitor => "běžný uživatel, posílá reporty",
                    Kind::Admin => "admin: fronta reportů v /admin/reports",
                    Kind::Suspended => "zablokovaný účet (přihlášení odmítne)",
                };
                let _ = writeln!(out, "| `{}` | `{}` | {role} |", user.username, user.email());
            }
        }
        out
    }
}
