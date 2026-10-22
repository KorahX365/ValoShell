//! Menu sounds from the player's own disk, per menu style: the real button clicks of games they
//! have installed (Minecraft's assets, CS 1.6 under Steam) and any files they drop into the
//! app's `sounds` folder. Nothing is bundled or copied; the UI plays the files where they lie.
//! A cue with no file here falls back to the style's synthesised sound in the UI.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

/// The cues the menu plays.
pub const CUES: [&str; 5] = ["hover", "tab", "confirm", "back", "found"];
/// Extensions the webview can decode.
const AUDIO_EXTENSIONS: [&str; 3] = ["wav", "ogg", "mp3"];
/// Folder name in the sounds folder for files used by every style.
pub const ALL_STYLES: &str = "all";

/// Minecraft asset keys per cue: `ui.button.click` plays `random/click_stereo` (sounds.json).
/// The game has no hover sound, so neither does its style.
const MINECRAFT_SOUNDS: [(&str, &str); 4] = [
    ("tab", "minecraft/sounds/random/click_stereo.ogg"),
    ("confirm", "minecraft/sounds/random/click_stereo.ogg"),
    ("back", "minecraft/sounds/random/click_stereo.ogg"),
    ("found", "minecraft/sounds/random/levelup.ogg"),
];

/// CS 1.6 files per cue, relative to the Half-Life folder; the first that exists wins.
const CS16_SOUNDS: [(&str, &[&str]); 5] = [
    (
        "hover",
        &[
            "valve/sound/UI/buttonrollover.wav",
            "valve/sound/common/launch_glow1.wav",
        ],
    ),
    (
        "tab",
        &[
            "valve/sound/UI/buttonclick.wav",
            "valve/sound/common/launch_select2.wav",
        ],
    ),
    (
        "confirm",
        &[
            "valve/sound/UI/buttonclick.wav",
            "valve/sound/common/launch_select2.wav",
        ],
    ),
    (
        "back",
        &[
            "valve/sound/UI/buttonclickrelease.wav",
            "valve/sound/common/launch_dnmenu1.wav",
        ],
    ),
    (
        "found",
        &[
            "cstrike/sound/radio/locknload.wav",
            "cstrike/sound/radio/letsgo.wav",
        ],
    ),
];

const STEAM_ROOTS: [&str; 4] = [
    "C:/Program Files (x86)/Steam",
    "C:/Program Files/Steam",
    "D:/Steam",
    "D:/SteamLibrary",
];
const HALF_LIFE_DIR: &str = "steamapps/common/Half-Life";
const LIBRARY_FOLDERS: &str = "steamapps/libraryfolders.vdf";

/// One sound file for one cue of one style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct UiSound {
    /// Style id (`minecraft`, `cs16`, ...) or `all`.
    pub skin: String,
    /// `hover`, `tab`, `confirm`, `back` or `found`.
    pub cue: String,
    /// Absolute path; the UI turns it into an `asset:` url.
    pub path: String,
    /// Where it came from: `minecraft`, `cs16` or `folder`.
    pub source: String,
}

/// Every sound found: the player's folder first (it overrides), then the installed games.
pub fn find_sounds(app_data: Option<&Path>, sounds_dir: &Path) -> Vec<UiSound> {
    let mut found = folder_sounds(sounds_dir);
    if let Some(root) = app_data {
        found.extend(minecraft_sounds(&root.join(".minecraft")));
    }
    if let Some(half_life) = find_half_life() {
        found.extend(cs16_sounds(&half_life));
    }
    found
}

/// `<sounds>/<style>/<cue>.<ext>`, plus `<sounds>/all/<cue>.<ext>` for every style.
pub fn folder_sounds(sounds_dir: &Path) -> Vec<UiSound> {
    let Ok(entries) = std::fs::read_dir(sounds_dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        let Some(skin) = dir.is_dir().then(|| dir.file_name()).flatten() else {
            continue;
        };
        let skin = skin.to_string_lossy().to_ascii_lowercase();
        for cue in CUES {
            let file = AUDIO_EXTENSIONS
                .iter()
                .map(|extension| dir.join(format!("{cue}.{extension}")))
                .find(|path| path.is_file());
            if let Some(path) = file {
                found.push(sound(&skin, cue, &path, "folder"));
            }
        }
    }
    found
}

/// Minecraft's sounds through its newest asset index (`assets/indexes/*.json`), whose
/// objects live at `assets/objects/<first two hash chars>/<hash>`.
pub fn minecraft_sounds(minecraft: &Path) -> Vec<UiSound> {
    let assets = minecraft.join("assets");
    let Some(index) = newest_index(&assets.join("indexes")) else {
        return Vec::new();
    };
    let objects = &index["objects"];
    MINECRAFT_SOUNDS
        .iter()
        .filter_map(|(cue, key)| {
            let hash = objects[*key]["hash"].as_str()?;
            let prefix = hash.get(..2)?;
            let path = assets.join("objects").join(prefix).join(hash);
            path.is_file()
                .then(|| sound("minecraft", cue, &path, "minecraft"))
        })
        .collect()
}

/// CS 1.6's menu and radio sounds from a Half-Life install folder.
pub fn cs16_sounds(half_life: &Path) -> Vec<UiSound> {
    CS16_SOUNDS
        .iter()
        .filter_map(|(cue, candidates)| {
            let path = candidates
                .iter()
                .map(|relative| native(&half_life.join(relative)))
                .find(|path| path.is_file())?;
            Some(sound("cs16", cue, &path, "cs16"))
        })
        .collect()
}

/// The Half-Life folder (CS 1.6 installs into it) in any Steam library.
fn find_half_life() -> Option<PathBuf> {
    let roots: Vec<PathBuf> = STEAM_ROOTS.iter().map(PathBuf::from).collect();
    let libraries = roots
        .iter()
        .filter_map(|root| std::fs::read_to_string(root.join(LIBRARY_FOLDERS)).ok())
        .flat_map(|text| parse_library_paths(&text));
    roots
        .iter()
        .cloned()
        .chain(libraries)
        .map(|library| native(&library.join(HALF_LIFE_DIR)))
        .find(|dir| dir.is_dir())
}

/// `"path"  "D:\\SteamLibrary"` lines of `libraryfolders.vdf` → those folders.
fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| line.trim().strip_prefix("\"path\""))
        .map(|rest| rest.trim().trim_matches('"').replace("\\\\", "\\"))
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// The asset index changed most recently: the version the player runs now.
fn newest_index(indexes: &Path) -> Option<serde_json::Value> {
    let newest = std::fs::read_dir(indexes)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .max_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })?;
    serde_json::from_str(&std::fs::read_to_string(newest).ok()?).ok()
}

fn native(path: &Path) -> PathBuf {
    path.components().collect()
}

fn sound(skin: &str, cue: &str, path: &Path, source: &str) -> UiSound {
    UiSound {
        skin: skin.to_owned(),
        cue: cue.to_owned(),
        path: path.to_string_lossy().into_owned(),
        source: source.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vl-game-sounds-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        std::fs::write(path, b"x").expect("file");
    }

    #[test]
    fn folder_sounds_reads_cues_per_style() {
        let dir = scratch("folder");
        touch(&dir.join("csgo/hover.wav"));
        touch(&dir.join("csgo/confirm.ogg"));
        touch(&dir.join("csgo/notes.txt"));
        touch(&dir.join("All/back.mp3"));
        touch(&dir.join("loose.wav"));
        let mut found = folder_sounds(&dir);
        found.sort_by(|a, b| (&a.skin, &a.cue).cmp(&(&b.skin, &b.cue)));
        let pairs: Vec<(&str, &str)> = found
            .iter()
            .map(|s| (s.skin.as_str(), s.cue.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [("all", "back"), ("csgo", "confirm"), ("csgo", "hover")]
        );
        assert!(found.iter().all(|s| s.source == "folder"));
    }

    #[test]
    fn minecraft_sounds_resolve_through_the_newest_index() {
        let dir = scratch("minecraft");
        let index = r#"{"objects":{
            "minecraft/sounds/random/click_stereo.ogg":{"hash":"ab12cd","size":1},
            "minecraft/sounds/random/levelup.ogg":{"hash":"ff00ee","size":1}}}"#;
        std::fs::create_dir_all(dir.join("assets/indexes")).expect("dirs");
        std::fs::write(dir.join("assets/indexes/17.json"), index).expect("index");
        touch(&dir.join("assets/objects/ab/ab12cd"));
        let found = minecraft_sounds(&dir);
        let cues: Vec<&str> = found.iter().map(|s| s.cue.as_str()).collect();
        // levelup's object is missing on disk, so `found` has no file.
        assert_eq!(cues, ["tab", "confirm", "back"]);
        assert!(found[0].path.ends_with("ab12cd"));
    }

    #[test]
    fn cs16_sounds_take_the_first_file_that_exists() {
        let dir = scratch("cs16");
        touch(&dir.join("valve/sound/common/launch_select2.wav"));
        touch(&dir.join("cstrike/sound/radio/letsgo.wav"));
        let found = cs16_sounds(&dir);
        let cues: Vec<&str> = found.iter().map(|s| s.cue.as_str()).collect();
        assert_eq!(cues, ["tab", "confirm", "found"]);
        assert!(found[2].path.ends_with("letsgo.wav"));
    }

    #[test]
    fn library_paths_unescape_backslashes() {
        let vdf =
            "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}";
        assert_eq!(
            parse_library_paths(vdf),
            [PathBuf::from("D:\\SteamLibrary")]
        );
    }
}
