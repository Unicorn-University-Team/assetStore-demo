//! The demo catalogue: hand-written names and descriptions, every file made by `files`.

use crate::files;

pub enum FileSpec {
    /// width, height, hue
    Png(u32, u32, u32),
    /// hue, box size x/y/z
    Glb(u32, [f32; 3]),
    /// tone in Hz, length in ms
    Wav(f32, u32),
    /// width, height in tiles
    Tmx(u32, u32),
    /// six 64x64 sky faces, hue of the first one
    SkyboxZip(u32),
}

pub struct DemoAsset {
    pub owner: &'static str,
    /// 2d | 3d | map | sound | environment
    pub kind: &'static str,
    pub name: &'static str,
    description: &'static str,
    pub file: FileSpec,
    /// How long ago it was published (created_at), for a believable "newest first".
    pub days_ago: u32,
    pub downloads: u32,
}

/// Marks the one asset whose description is exactly at the catalog's 2000 character limit.
const LONG: &str = "@long";

impl DemoAsset {
    pub fn description(&self) -> String {
        if self.description != LONG {
            return self.description.to_string();
        }
        let sentence = "A modular dungeon kit with walls, floors, doors, stairs and props that snap to a 2 m grid. ";
        sentence.repeat(30).chars().take(2000).collect()
    }

    fn slug(&self) -> String {
        let slug: String = self
            .name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        let parts: Vec<&str> = slug.split('-').filter(|part| !part.is_empty()).collect();
        parts.join("-").chars().take(60).collect()
    }

    pub fn file_name(&self) -> String {
        let extension = match self.file {
            FileSpec::Png(..) => "png",
            FileSpec::Glb(..) => "glb",
            FileSpec::Wav(..) => "wav",
            FileSpec::Tmx(..) => "tmx",
            FileSpec::SkyboxZip(..) => "zip",
        };
        format!("{}.{extension}", self.slug())
    }

    pub fn content_type(&self) -> &'static str {
        match self.file {
            FileSpec::Png(..) => "image/png",
            FileSpec::Glb(..) => "model/gltf-binary",
            FileSpec::Wav(..) => "audio/wav",
            FileSpec::Tmx(..) => "application/xml",
            FileSpec::SkyboxZip(..) => "application/zip",
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self.file {
            FileSpec::Png(width, height, hue) => files::png(width, height, hue),
            FileSpec::Glb(hue, size) => files::glb(hue, size),
            FileSpec::Wav(tone, millis) => files::wav(tone, millis),
            FileSpec::Tmx(width, height) => files::tmx(width, height),
            FileSpec::SkyboxZip(hue) => {
                let faces = ["right", "left", "top", "bottom", "front", "back"];
                let entries: Vec<(String, Vec<u8>)> = faces
                    .iter()
                    .enumerate()
                    .map(|(i, face)| {
                        (
                            format!("{face}.png"),
                            files::png(64, 64, hue + i as u32 * 8),
                        )
                    })
                    .collect();
                let borrowed: Vec<(&str, Vec<u8>)> = entries
                    .iter()
                    .map(|(name, data)| (name.as_str(), data.clone()))
                    .collect();
                files::zip(&borrowed)
            }
        }
    }
}

const fn a(
    owner: &'static str,
    kind: &'static str,
    name: &'static str,
    description: &'static str,
    file: FileSpec,
    days_ago: u32,
    downloads: u32,
) -> DemoAsset {
    DemoAsset {
        owner,
        kind,
        name,
        description,
        file,
        days_ago,
        downloads,
    }
}

use FileSpec::{Glb, Png, SkyboxZip, Tmx, Wav};

pub const ASSETS: &[DemoAsset] = &[
    // pixel_forge: the busiest author, more than one page in "My assets"
    a(
        "pixel_forge",
        "2d",
        "Knight sprite sheet",
        "A 16-frame walk and attack cycle for a small pixel knight. Transparent background, 32x32 frames.",
        Png(256, 128, 210),
        2,
        4810,
    ),
    a(
        "pixel_forge",
        "2d",
        "Slime enemy pack",
        "Green, blue and red slimes with idle, hop and squish animations.",
        Png(192, 64, 110),
        3,
        2210,
    ),
    a(
        "pixel_forge",
        "2d",
        "Forest tileset 16x16",
        "Grass, dirt paths, trees, bushes and flowers for top-down RPGs.",
        Png(256, 256, 95),
        5,
        6120,
    ),
    a(
        "pixel_forge",
        "2d",
        "Dungeon tileset 16x16",
        "Stone walls, floors, doors, torches and chests.",
        Png(256, 256, 30),
        6,
        3975,
    ),
    a(
        "pixel_forge",
        "2d",
        "Pixel UI kit",
        "Buttons, panels, health bars and icons in a warm retro palette.",
        Png(192, 192, 40),
        8,
        1890,
    ),
    a(
        "pixel_forge",
        "2d",
        "Coin and gem pickups",
        "Spinning coins and sparkling gems, 8 frames each.",
        Png(128, 64, 50),
        9,
        1544,
    ),
    a(
        "pixel_forge",
        "2d",
        "Pixel font sheet",
        "Upper and lower case letters, digits and punctuation on a 8x8 grid.",
        Png(128, 64, 0),
        11,
        980,
    ),
    a(
        "pixel_forge",
        "2d",
        "Parallax mountains",
        "Four layers of mountains and clouds for side-scrolling backgrounds.",
        Png(256, 144, 220),
        12,
        2675,
    ),
    a(
        "pixel_forge",
        "2d",
        "Ninja character",
        "Run, jump, wall slide and dash animations.",
        Png(256, 64, 270),
        14,
        3302,
    ),
    a(
        "pixel_forge",
        "2d",
        "Spell effects",
        "Fireball, ice shard, lightning and heal effects, 10 frames each.",
        Png(256, 128, 15),
        15,
        2088,
    ),
    a(
        "pixel_forge",
        "2d",
        "Village houses",
        "Six cottages with roofs in different colours, plus a well and a market stall.",
        Png(256, 192, 25),
        17,
        1415,
    ),
    a(
        "pixel_forge",
        "2d",
        "Cave tileset",
        "Rocky cave walls, crystals and stalactites.",
        Png(192, 192, 260),
        19,
        860,
    ),
    a(
        "pixel_forge",
        "2d",
        "Pirate ship",
        "A side view ship with sails in three states: furled, half and full.",
        Png(256, 128, 200),
        21,
        1290,
    ),
    a(
        "pixel_forge",
        "2d",
        "Weapon icons",
        "Swords, bows, axes and staves as 32x32 inventory icons.",
        Png(160, 160, 0),
        23,
        3420,
    ),
    a(
        "pixel_forge",
        "2d",
        "Food icons",
        "Bread, cheese, apples, fish and potions for crafting games.",
        Png(160, 160, 35),
        25,
        1112,
    ),
    a(
        "pixel_forge",
        "2d",
        "Snow tileset",
        "Snowy ground, ice, frozen lakes and pine trees.",
        Png(256, 256, 190),
        27,
        745,
    ),
    a(
        "pixel_forge",
        "2d",
        "Desert tileset",
        "Sand, dunes, cacti, ruins and oasis water.",
        Png(256, 256, 45),
        29,
        690,
    ),
    a(
        "pixel_forge",
        "2d",
        "Robot enemies",
        "Three robots with patrol, alert and explode animations.",
        Png(192, 64, 180),
        31,
        1530,
    ),
    a(
        "pixel_forge",
        "2d",
        "Particles pack",
        "Dust, smoke, sparks and leaves for small effects.",
        Png(128, 128, 60),
        34,
        920,
    ),
    a(
        "pixel_forge",
        "2d",
        "Portrait frames",
        "Ornate frames for character portraits in dialogue boxes.",
        Png(128, 128, 300),
        37,
        410,
    ),
    a(
        "pixel_forge",
        "2d",
        "Cursor set",
        "",
        Png(64, 64, 330),
        40,
        230,
    ),
    a(
        "pixel_forge",
        "2d",
        "Extremely detailed hand-painted medieval market square background with merchants and stalls",
        "A large painted background, at the very edge of the name length the catalog accepts.",
        Png(256, 160, 20),
        44,
        118,
    ),
    // lowpoly_lena: 3D
    a(
        "lowpoly_lena",
        "3d",
        "Low poly crate",
        "A wooden crate, 12 triangles, game ready. Rotate it in the 3D preview.",
        Glb(30, [1.0, 1.0, 1.0]),
        1,
        3120,
    ),
    a(
        "lowpoly_lena",
        "3d",
        "Stone pillar",
        "A tall pillar for temples and ruins.",
        Glb(40, [0.6, 2.4, 0.6]),
        4,
        1405,
    ),
    a(
        "lowpoly_lena",
        "3d",
        "Market table",
        "A long low table for market scenes.",
        Glb(25, [2.4, 0.5, 1.0]),
        7,
        812,
    ),
    a(
        "lowpoly_lena",
        "3d",
        "Ice block",
        "A translucent-looking ice block for winter levels.",
        Glb(195, [1.2, 1.2, 1.2]),
        13,
        664,
    ),
    a(
        "lowpoly_lena",
        "3d",
        "Brick wall segment",
        "A modular wall piece, 4 m long.",
        Glb(10, [4.0, 2.0, 0.4]),
        20,
        1950,
    ),
    a(
        "lowpoly_lena",
        "3d",
        "Modular dungeon kit",
        LONG,
        Glb(260, [2.0, 2.0, 2.0]),
        33,
        2210,
    ),
    // voxel_vera: 3D
    a(
        "voxel_vera",
        "3d",
        "Voxel tree",
        "A chunky voxel tree trunk, pairs with the voxel leaves.",
        Glb(100, [0.8, 3.0, 0.8]),
        2,
        1702,
    ),
    a(
        "voxel_vera",
        "3d",
        "Voxel cottage",
        "A tiny house block, ready for city builders.",
        Glb(15, [3.0, 2.2, 3.0]),
        10,
        940,
    ),
    a(
        "voxel_vera",
        "3d",
        "Voxel bridge",
        "A plank bridge that spans 6 m.",
        Glb(35, [6.0, 0.3, 1.5]),
        18,
        455,
    ),
    a(
        "voxel_vera",
        "3d",
        "Žlutý kámen – dekorace",
        "Kámen s diakritikou v názvu: ověřuje, že čeština projde katalogem i vyhledáváním.",
        Glb(55, [1.0, 0.7, 1.3]),
        26,
        87,
    ),
    // tomas_tiles: maps
    a(
        "tomas_tiles",
        "map",
        "River valley",
        "A Tiled map with a river crossing a grass valley. Uses a 3-tile terrain set.",
        Tmx(40, 30),
        3,
        1220,
    ),
    a(
        "tomas_tiles",
        "map",
        "Castle courtyard",
        "A walled courtyard with gates on two sides.",
        Tmx(32, 32),
        9,
        860,
    ),
    a(
        "tomas_tiles",
        "map",
        "Arena",
        "A small square arena for local multiplayer.",
        Tmx(20, 20),
        16,
        1490,
    ),
    a(
        "tomas_tiles",
        "map",
        "Overworld preview",
        "A painted overview of the whole overworld.",
        Png(256, 192, 120),
        24,
        575,
    ),
    a("tomas_tiles", "map", "Maze level", "", Tmx(28, 28), 39, 302),
    // mapmaker_mira: maps
    a(
        "mapmaker_mira",
        "map",
        "Harbour town",
        "Docks, warehouses and a lighthouse on a 48x36 map.",
        Tmx(48, 36),
        5,
        1880,
    ),
    a(
        "mapmaker_mira",
        "map",
        "Swamp trail",
        "A winding trail through a swamp with a hidden shortcut.",
        Tmx(36, 24),
        12,
        640,
    ),
    a(
        "mapmaker_mira",
        "map",
        "Island map",
        "A painted map of a small island with treasure marks.",
        Png(256, 256, 170),
        22,
        1310,
    ),
    a(
        "mapmaker_mira",
        "map",
        "Tower floors",
        "Five stacked floors of a wizard tower.",
        Tmx(24, 24),
        35,
        520,
    ),
    // synthwave_sam: music and tones
    a(
        "synthwave_sam",
        "sound",
        "Menu confirm",
        "A bright two-note confirm sound for menus.",
        Wav(880.0, 300),
        1,
        5230,
    ),
    a(
        "synthwave_sam",
        "sound",
        "Level up jingle",
        "A short rising tone for level ups and achievements.",
        Wav(660.0, 1200),
        6,
        3890,
    ),
    a(
        "synthwave_sam",
        "sound",
        "Low drone",
        "A low synth drone for dark ambience, loops cleanly.",
        Wav(110.0, 3000),
        11,
        1460,
    ),
    a(
        "synthwave_sam",
        "sound",
        "Laser shot",
        "A punchy laser for space shooters.",
        Wav(1320.0, 250),
        18,
        2740,
    ),
    a(
        "synthwave_sam",
        "sound",
        "Power down",
        "A falling tone for game over screens.",
        Wav(220.0, 1500),
        28,
        1180,
    ),
    a(
        "synthwave_sam",
        "sound",
        "Pickup blip",
        "A tiny blip for coins and pickups.",
        Wav(1760.0, 150),
        36,
        3010,
    ),
    // foley_filip: effects
    a(
        "foley_filip",
        "sound",
        "Door knock",
        "Three knocks on a wooden door.",
        Wav(180.0, 700),
        4,
        760,
    ),
    a(
        "foley_filip",
        "sound",
        "Bell chime",
        "A clear bell for notifications and quest updates.",
        Wav(1046.0, 2000),
        10,
        1325,
    ),
    a(
        "foley_filip",
        "sound",
        "Footstep on stone",
        "A single footstep, layer it for walking.",
        Wav(140.0, 200),
        21,
        2105,
    ),
    a(
        "foley_filip",
        "sound",
        "Alarm",
        "A repeating alarm tone for stealth games.",
        Wav(980.0, 2500),
        32,
        615,
    ),
    // skybox_karel: environments
    a(
        "skybox_karel",
        "environment",
        "Sunset skybox",
        "Six 64x64 faces of a warm sunset sky.",
        SkyboxZip(20),
        2,
        2950,
    ),
    a(
        "skybox_karel",
        "environment",
        "Night sky skybox",
        "A deep blue night sky, six faces.",
        SkyboxZip(230),
        7,
        2260,
    ),
    a(
        "skybox_karel",
        "environment",
        "Overcast skybox",
        "Soft grey clouds for moody scenes.",
        SkyboxZip(210),
        13,
        980,
    ),
    a(
        "skybox_karel",
        "environment",
        "Alien world skybox",
        "A purple sky with a green haze.",
        SkyboxZip(280),
        19,
        1540,
    ),
    a(
        "skybox_karel",
        "environment",
        "Desert ground plane",
        "A large flat sand plane to build a scene on.",
        Glb(45, [20.0, 0.1, 20.0]),
        24,
        720,
    ),
    a(
        "skybox_karel",
        "environment",
        "Grass ground plane",
        "A large flat grass plane.",
        Glb(100, [20.0, 0.1, 20.0]),
        30,
        1110,
    ),
    a(
        "skybox_karel",
        "environment",
        "Ocean surface",
        "A flat blue ocean plane with a slight tint.",
        Glb(205, [30.0, 0.05, 30.0]),
        38,
        640,
    ),
    a(
        "skybox_karel",
        "environment",
        "Fog panorama",
        "A wide foggy panorama for backgrounds.",
        Png(256, 96, 200),
        45,
        330,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::users::{Kind, USERS};
    use std::collections::HashSet;

    /// The catalog's whitelist (services/catalog/src/models/asset.model.ts) for the extensions used here.
    const ALLOWED_EXTENSIONS: &[&str] = &["png", "glb", "wav", "tmx", "zip"];
    const TYPES: &[&str] = &["2d", "3d", "map", "sound", "environment"];

    #[test]
    fn fit_the_catalog_rules() {
        for asset in ASSETS {
            assert!(TYPES.contains(&asset.kind), "{}: type", asset.name);
            assert!(
                !asset.name.trim().is_empty() && asset.name.chars().count() <= 100,
                "{}: name length",
                asset.name
            );
            assert!(
                asset.description().chars().count() <= 2000,
                "{}: description",
                asset.name
            );
            let extension = asset.file_name().rsplit('.').next().unwrap().to_string();
            assert!(
                ALLOWED_EXTENSIONS.contains(&extension.as_str()),
                "{}: extension",
                asset.name
            );
        }
    }

    #[test]
    fn owners_are_demo_authors() {
        for asset in ASSETS {
            let owner = USERS.iter().find(|user| user.username == asset.owner);
            assert!(
                matches!(owner, Some(user) if user.kind == Kind::Author),
                "{}: owner {}",
                asset.name,
                asset.owner
            );
        }
    }

    #[test]
    fn names_are_unique_per_owner() {
        let keys: HashSet<_> = ASSETS
            .iter()
            .map(|asset| (asset.owner, asset.name))
            .collect();
        assert_eq!(keys.len(), ASSETS.len());
    }

    #[test]
    fn every_type_has_at_least_six_and_one_author_has_two_pages() {
        for kind in TYPES {
            assert!(
                ASSETS.iter().filter(|asset| asset.kind == *kind).count() >= 6,
                "{kind}"
            );
        }
        assert!(
            ASSETS
                .iter()
                .filter(|asset| asset.owner == "pixel_forge")
                .count()
                > 20
        );
    }

    #[test]
    fn edge_cases_are_present() {
        assert!(ASSETS.iter().any(|asset| asset.description().is_empty()));
        assert!(
            ASSETS
                .iter()
                .any(|asset| asset.description().chars().count() == 2000)
        );
        assert!(ASSETS.iter().any(|asset| asset.name.chars().count() > 80));
        assert!(ASSETS.iter().any(|asset| !asset.name.is_ascii()));
    }

    #[test]
    fn all_files_together_stay_small() {
        let total: usize = ASSETS.iter().map(|asset| asset.bytes().len()).sum();
        assert!(total < 20 * 1024 * 1024, "{total} bytes");
        assert!(ASSETS.iter().all(|asset| asset.bytes().len() < 250 * 1024));
    }
}
