/// Every demo account has an e-mail in this domain; wipe deletes only these accounts and what they own.
pub const DEMO_EMAIL_DOMAIN: &str = "demo.assetstore.test";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Publishes assets.
    Author,
    /// Owns nothing, sends reports.
    Visitor,
    /// Role ADMIN, decides reports.
    Admin,
    /// Status SUSPENDED, shows a blocked account.
    Suspended,
}

pub struct DemoUser {
    pub username: &'static str,
    pub kind: Kind,
}

impl DemoUser {
    pub fn email(&self) -> String {
        format!("{}@{DEMO_EMAIL_DOMAIN}", self.username.replace('_', "."))
    }
}

pub const USERS: &[DemoUser] = &[
    DemoUser {
        username: "pixel_forge",
        kind: Kind::Author,
    },
    DemoUser {
        username: "lowpoly_lena",
        kind: Kind::Author,
    },
    DemoUser {
        username: "tomas_tiles",
        kind: Kind::Author,
    },
    DemoUser {
        username: "synthwave_sam",
        kind: Kind::Author,
    },
    DemoUser {
        username: "mapmaker_mira",
        kind: Kind::Author,
    },
    DemoUser {
        username: "foley_filip",
        kind: Kind::Author,
    },
    DemoUser {
        username: "voxel_vera",
        kind: Kind::Author,
    },
    DemoUser {
        username: "skybox_karel",
        kind: Kind::Author,
    },
    DemoUser {
        username: "curious_eva",
        kind: Kind::Visitor,
    },
    DemoUser {
        username: "jan_plays",
        kind: Kind::Visitor,
    },
    DemoUser {
        username: "demo_admin",
        kind: Kind::Admin,
    },
    DemoUser {
        username: "demo_suspended",
        kind: Kind::Suspended,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn usernames_and_emails_are_unique_ignoring_case() {
        let names: HashSet<_> = USERS
            .iter()
            .map(|user| user.username.to_lowercase())
            .collect();
        let emails: HashSet<_> = USERS.iter().map(DemoUser::email).collect();
        assert_eq!(names.len(), USERS.len());
        assert_eq!(emails.len(), USERS.len());
    }

    #[test]
    fn usernames_pass_identity_rules() {
        // identity: 1 to 50 characters after trim
        assert!(
            USERS
                .iter()
                .all(|user| !user.username.is_empty() && user.username.len() <= 50)
        );
    }

    #[test]
    fn every_kind_is_present() {
        for kind in [Kind::Author, Kind::Visitor, Kind::Admin, Kind::Suspended] {
            assert!(
                USERS.iter().any(|user| user.kind == kind),
                "{kind:?} is missing"
            );
        }
    }
}
