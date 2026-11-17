/// A report from a demo visitor on a demo asset. `rejected` ones are decided by demo_admin, the rest stay open.
pub struct DemoReport {
    pub author: &'static str,
    pub asset_owner: &'static str,
    pub asset_name: &'static str,
    /// copyright | inappropriate | malware | spam | other
    pub reason: &'static str,
    pub comment: &'static str,
    pub rejected: bool,
}

const fn r(
    author: &'static str,
    asset_owner: &'static str,
    asset_name: &'static str,
    reason: &'static str,
    comment: &'static str,
    rejected: bool,
) -> DemoReport {
    DemoReport {
        author,
        asset_owner,
        asset_name,
        reason,
        comment,
        rejected,
    }
}

pub const REPORTS: &[DemoReport] = &[
    r(
        "curious_eva",
        "pixel_forge",
        "Ninja character",
        "copyright",
        "Looks very close to a sprite from a commercial game.",
        false,
    ),
    r(
        "jan_plays",
        "synthwave_sam",
        "Laser shot",
        "spam",
        "",
        false,
    ),
    r(
        "curious_eva",
        "skybox_karel",
        "Alien world skybox",
        "inappropriate",
        "The colours give me a headache.",
        true,
    ),
    r(
        "jan_plays",
        "lowpoly_lena",
        "Low poly crate",
        "malware",
        "My antivirus complained after the download.",
        false,
    ),
    r(
        "curious_eva",
        "tomas_tiles",
        "Maze level",
        "other",
        "The description is empty, not sure what this is.",
        true,
    ),
    r(
        "jan_plays",
        "pixel_forge",
        "Weapon icons",
        "copyright",
        "These icons are from a paid pack.",
        false,
    ),
    r("curious_eva", "foley_filip", "Alarm", "other", "", true),
    r(
        "jan_plays",
        "mapmaker_mira",
        "Island map",
        "spam",
        "Same map uploaded twice under different names.",
        false,
    ),
    r(
        "curious_eva",
        "voxel_vera",
        "Voxel bridge",
        "inappropriate",
        "",
        true,
    ),
    r(
        "jan_plays",
        "pixel_forge",
        "Pixel UI kit",
        "other",
        "Download link gives a different file name than expected.",
        false,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::assets::ASSETS;
    use crate::fixtures::users::{Kind, USERS};

    #[test]
    fn reports_point_at_demo_assets_from_visitors() {
        const REASONS: &[&str] = &["copyright", "inappropriate", "malware", "spam", "other"];
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
            assert!(report.comment.chars().count() <= 500);
        }
        assert!(
            REPORTS.iter().any(|report| report.rejected)
                && REPORTS.iter().any(|report| !report.rejected)
        );
    }
}
