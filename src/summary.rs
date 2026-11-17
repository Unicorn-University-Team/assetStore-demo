//! What a run did: printed to the terminal and, with --summary, written as Markdown for the GitHub job summary.

use std::fmt::Write;

use crate::config::Config;
use crate::fixtures::users::{Kind, USERS};

#[derive(Default)]
pub struct Summary {
    /// e.g. "seed"; picks the icon of the heading.
    pub command: &'static str,
    pub dry_run: bool,
    /// (icon, what, count)
    pub counts: Vec<(&'static str, String, usize)>,
    /// Per demo user: (username, kind, state), only status and check fill it.
    pub users: Vec<(String, String, String)>,
    /// (icon, label, hint, url) to open on the site: the 3D model, the sound, the picture...
    pub links: Vec<(&'static str, String, &'static str, String)>,
    pub notes: Vec<String>,
    /// Something is broken: check fails the run, after the summary is written.
    pub problems: Vec<String>,
    pub show_logins: bool,
    pub seconds: f32,
}

impl Summary {
    pub fn new(command: &'static str, dry_run: bool) -> Self {
        Self {
            command,
            dry_run,
            ..Self::default()
        }
    }

    pub fn count(&mut self, icon: &'static str, what: &str, count: usize) {
        self.counts.push((icon, what.to_string(), count));
    }

    fn icon(&self) -> &'static str {
        match self.command {
            "seed" => "🌱",
            "wipe" => "🧹",
            "check" => "🩺",
            _ => "📊",
        }
    }

    pub fn print(&self) {
        println!(
            "\n{} demo data: {}{}",
            self.icon(),
            self.command,
            if self.dry_run { " (dry run)" } else { "" }
        );
        for (_, what, count) in &self.counts {
            println!("  {what}: {count}");
        }
        for (_, label, _, url) in &self.links {
            println!("  {label}: {url}");
        }
        for note in &self.notes {
            println!("  {note}");
        }
        for problem in &self.problems {
            println!("  PROBLEM: {problem}");
        }
    }

    /// Markdown with the result, links to try and the demo logins (never the password).
    pub fn markdown(&self, config: &Config) -> String {
        let site = config.site_url();
        let mut out = format!("## {} Demo data · `{}`\n\n", self.icon(), self.command);
        let verdict = match (self.dry_run, self.problems.is_empty()) {
            (true, _) => "🧪 **Zkouška nanečisto**: nic se nezměnilo.".to_string(),
            (false, true) => "✅ **Hotovo**".to_string(),
            (false, false) => format!("❌ **{} problém(y)**", self.problems.len()),
        };
        let _ = writeln!(
            out,
            "{verdict} za {:.0} s · 🌐 [{site}]({site})\n",
            self.seconds.max(1.0)
        );
        for problem in &self.problems {
            let _ = writeln!(out, "> [!CAUTION]\n> {problem}\n");
        }
        if !self.counts.is_empty() {
            out.push_str("| | Počet |\n|---|---:|\n");
            for (icon, what, count) in &self.counts {
                let _ = writeln!(out, "| {icon} {what} | **{count}** |");
            }
            out.push('\n');
        }
        for note in &self.notes {
            let _ = writeln!(out, "> {note}\n");
        }
        if !self.links.is_empty() {
            out.push_str("### 🔗 Vyzkoušej na webu\n\n");
            for (icon, label, hint, url) in &self.links {
                let _ = writeln!(out, "- {icon} [{label}]({url}): {hint}");
            }
            out.push('\n');
        }
        if !self.users.is_empty() {
            out.push_str("<details><summary>👥 Demo účty a jejich assety</summary>\n\n| Účet | Druh | Stav |\n|---|---|---|\n");
            for (username, kind, state) in &self.users {
                let _ = writeln!(out, "| `{username}` | {kind} | {state} |");
            }
            out.push_str("\n</details>\n\n");
        }
        if self.show_logins {
            let _ = write!(
                out,
                "### 🔑 Přihlášení\n\n[{site}/login]({site}/login) · heslo všech demo účtů je v secretu `DEMO_PASSWORD` (zná ho Yura).\n\n\
                 | E-mail | Co ukazuje |\n|---|---|\n"
            );
            for user in USERS {
                let _ = writeln!(out, "| `{}` | {} |", user.email(), kind_hint(user.kind));
            }
        }
        out
    }
}

pub fn kind_hint(kind: Kind) -> &'static str {
    match kind {
        Kind::Author => "autor assetů, „My assets“",
        Kind::Visitor => "běžný uživatel, posílá reporty",
        Kind::Admin => "admin: fronta reportů v `/admin/reports`",
        Kind::Suspended => "zablokovaný účet, přihlášení odmítne",
    }
}
