/// A report from a demo visitor on a demo asset. `rejected` ones are decided by demo_admin, the rest stay open.
pub struct DemoReport {
    pub author: &'static str,
    pub asset_owner: &'static str,
    pub asset_name: &'static str,
    /// AM report reasons: inappropriateContent | copyright | spam | brokenFile | other
    pub reason: &'static str,
    pub rejected: bool,
}

const fn r(
    author: &'static str,
    asset_owner: &'static str,
    asset_name: &'static str,
    reason: &'static str,
    rejected: bool,
) -> DemoReport {
    DemoReport {
        author,
        asset_owner,
        asset_name,
        reason,
        rejected,
    }
}

pub const REPORTS: &[DemoReport] = &[
    r(
        "curious_eva",
        "pixel_forge",
        "Ninja character",
        "copyright",
        false,
    ),
    r("jan_plays", "synthwave_sam", "Laser shot", "spam", false),
    r(
        "curious_eva",
        "skybox_karel",
        "Alien world skybox",
        "inappropriateContent",
        true,
    ),
    r(
        "jan_plays",
        "lowpoly_lena",
        "Low poly crate",
        "brokenFile",
        false,
    ),
    r("curious_eva", "tomas_tiles", "Maze level", "other", true),
    r(
        "jan_plays",
        "pixel_forge",
        "Weapon icons",
        "copyright",
        false,
    ),
    r("curious_eva", "foley_filip", "Alarm", "other", true),
    r("jan_plays", "mapmaker_mira", "Island map", "spam", false),
    r(
        "curious_eva",
        "voxel_vera",
        "Voxel bridge",
        "inappropriateContent",
        true,
    ),
    r("jan_plays", "pixel_forge", "Pixel UI kit", "other", false),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::assets::ASSETS;
    use crate::fixtures::users::{Kind, USERS};

    #[test]
    fn reports_point_at_demo_assets_from_visitors() {
        const REASONS: &[&str] = &[
            "copyright",
            "inappropriateContent",
            "brokenFile",
            "spam",
            "other",
        ];
        for report in REPORTS {
            assert!(
                ASSETS
                    .iter()
                    .any(|asset| asset.owner == report.asset_owner
                        && asset.name == report.asset_name),
                "{}",
                report.asset_name
            );
            assert!(
                USERS
                    .iter()
                    .any(|user| user.username == report.author && user.kind == Kind::Visitor),
                "{}",
                report.author
            );
            assert!(REASONS.contains(&report.reason));
        }
        assert!(
            REPORTS.iter().any(|report| report.rejected)
                && REPORTS.iter().any(|report| !report.rejected)
        );
    }
}
