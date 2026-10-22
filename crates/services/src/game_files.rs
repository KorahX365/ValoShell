//! The installed game's own files: where VALORANT lives and the movies it ships (the home-screen
//! loops under `ShooterGame/Content/Movies/Menu`), so the menu can play the real thing as its
//! backdrop. Read-only: nothing here writes into the install.

use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;
use ts_rs::TS;

/// Riot Client metadata naming the install folder, relative to `%ProgramData%`.
const PRODUCT_SETTINGS: &str =
    "Riot Games/Metadata/valorant.live/valorant.live.product_settings.yaml";
const INSTALL_KEY: &str = "product_install_full_path:";
const MOVIES_DIR: &str = "ShooterGame/Content/Movies";
const USUAL_INSTALLS: [&str; 3] = [
    "C:/Riot Games/VALORANT/live",
    "D:/Riot Games/VALORANT/live",
    "E:/Riot Games/VALORANT/live",
];
const VIDEO_EXTENSIONS: [&str; 2] = ["mp4", "webm"];
/// `Movies/<group>/<sub>/file`: deep enough for every layout the game has used.
const MAX_DEPTH: usize = 3;

/// One video file shipped with the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct GameVideo {
    /// Absolute path; the UI turns it into an `asset:` url.
    pub path: String,
    /// Readable name from the file name (`HomeScreen Ep9 Act3`).
    pub label: String,
    /// Made for the home screen: the menu's own background.
    pub is_home_screen: bool,
}

/// Where VALORANT is installed: the Riot Client's metadata first, then the usual folders.
pub fn find_install(program_data: Option<&Path>) -> Option<PathBuf> {
    let from_metadata = program_data
        .and_then(|root| std::fs::read_to_string(root.join(PRODUCT_SETTINGS)).ok())
        .and_then(|text| parse_install_path(&text));
    from_metadata
        .into_iter()
        .chain(USUAL_INSTALLS.iter().map(PathBuf::from))
        .find(|install| movies_dir(install).is_dir())
}

/// The movies folder of an install, with separators normalized for the platform.
pub fn movies_dir(install: &Path) -> PathBuf {
    install.join(MOVIES_DIR).components().collect()
}

/// Every playable video under `movies`: home-screen loops first, newest first within each.
pub fn list_videos(movies: &Path) -> Vec<GameVideo> {
    let mut found = Vec::new();
    collect(movies, 0, &mut found);
    found.sort_by_key(|(modified, video)| Reverse((video.is_home_screen, *modified)));
    found.into_iter().map(|(_, video)| video).collect()
}

/// `product_install_full_path: "C:/Riot Games/VALORANT/live"` → that path.
fn parse_install_path(yaml: &str) -> Option<PathBuf> {
    yaml.lines()
        .find_map(|line| line.trim().strip_prefix(INSTALL_KEY))
        .map(|value| value.trim().trim_matches(['"', '\'']))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn collect(dir: &Path, depth: usize, found: &mut Vec<(SystemTime, GameVideo)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < MAX_DEPTH {
                collect(&path, depth + 1, found);
            }
        } else if let Some(video) = video_at(&path) {
            let modified = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            found.push((modified, video));
        }
    }
}

fn video_at(path: &Path) -> Option<GameVideo> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if !VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    let folder = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    Some(GameVideo {
        path: path.to_string_lossy().into_owned(),
        label: label_of(stem),
        is_home_screen: stem.to_ascii_lowercase().contains("home")
            || folder.eq_ignore_ascii_case("menu"),
    })
}

/// `HomeScreen_Ep9_Act3_1920x1080` → `HomeScreen Ep9 Act3`.
fn label_of(stem: &str) -> String {
    let words: Vec<&str> = stem
        .split(['_', '-', ' '])
        .filter(|word| !word.is_empty() && !is_resolution(word))
        .collect();
    if words.is_empty() {
        stem.to_owned()
    } else {
        words.join(" ")
    }
}

fn is_resolution(word: &str) -> bool {
    let is_number = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    word.split_once(['x', 'X'])
        .is_some_and(|(width, height)| is_number(width) && is_number(height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vl-game-files-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        std::fs::write(path, b"").expect("file");
    }

    #[test]
    fn reads_the_install_path_from_riot_metadata() {
        let yaml = "product_install_root: \"C:/Riot Games\"\nproduct_install_full_path: \"D:/Games/VALORANT/live\"\n";
        assert_eq!(
            parse_install_path(yaml),
            Some(PathBuf::from("D:/Games/VALORANT/live"))
        );
        assert_eq!(
            parse_install_path("product_install_full_path: \"\"\n"),
            None
        );
        assert_eq!(parse_install_path("other: 1\n"), None);
    }

    #[test]
    fn labels_drop_separators_and_resolutions() {
        assert_eq!(
            label_of("HomeScreen_Ep9_Act3_1920x1080"),
            "HomeScreen Ep9 Act3"
        );
        assert_eq!(label_of("Menu-Loop"), "Menu Loop");
        assert_eq!(label_of("1920x1080"), "1920x1080");
        assert!(!is_resolution("x1080"));
    }

    #[test]
    fn finds_the_install_named_by_metadata() {
        let root = scratch("install");
        let install = root.join("VALORANT/live");
        std::fs::create_dir_all(movies_dir(&install)).expect("movies");
        let metadata = root.join("ProgramData").join(PRODUCT_SETTINGS);
        std::fs::create_dir_all(metadata.parent().expect("parent")).expect("dirs");
        let yaml = format!("product_install_full_path: \"{}\"\n", install.display());
        std::fs::write(&metadata, yaml).expect("yaml");
        assert_eq!(find_install(Some(&root.join("ProgramData"))), Some(install));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lists_home_screen_videos_first_and_skips_other_files() {
        let movies = scratch("movies");
        touch(&movies.join("Agents/Intro_Jett.mp4"));
        touch(&movies.join("Menu/HomeScreen_Ep9_Act3_1920x1080.mp4"));
        touch(&movies.join("Menu/readme.txt"));
        touch(&movies.join("Menu/Old.bk2"));
        let videos = list_videos(&movies);
        let labels: Vec<&str> = videos.iter().map(|video| video.label.as_str()).collect();
        assert_eq!(labels, ["HomeScreen Ep9 Act3", "Intro Jett"]);
        assert!(videos[0].is_home_screen);
        assert!(!videos[1].is_home_screen);
        let _ = std::fs::remove_dir_all(movies);
    }
}
