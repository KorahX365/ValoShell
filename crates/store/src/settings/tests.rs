use super::*;

fn temp_file(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("valoshell-settings-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.join("settings.json")
}

#[test]
fn missing_file_loads_defaults() {
    let store = SettingsStore::load(temp_file("missing"));
    assert_eq!(store.get(), Settings::default());
}

#[test]
fn partial_file_fills_missing_fields_with_defaults() {
    let path = temp_file("partial");
    std::fs::write(&path, r#"{"match_flow":{"should_hide_on_lock":false}}"#).expect("write");

    let settings = SettingsStore::load(path).get();

    assert!(!settings.match_flow.should_hide_on_lock);
    assert!(settings.match_flow.should_hide_on_match_start);
    assert!(!settings.lobby.should_rank_enemies_in_agent_select);
}

#[test]
fn unknown_fields_are_ignored() {
    let path = temp_file("unknown");
    std::fs::write(
        &path,
        r#"{"retired":{"x":1},"streamer":{"is_enabled":true}}"#,
    )
    .expect("write");

    let settings = SettingsStore::load(path).get();

    assert!(settings.streamer.is_enabled);
}

#[test]
fn corrupt_file_is_backed_up_and_defaults_are_used() {
    let path = temp_file("corrupt");
    std::fs::write(&path, "{not json").expect("write");

    let settings = SettingsStore::load(path.clone()).get();

    assert_eq!(settings, Settings::default());
    assert!(path.with_extension("json.corrupt").exists());
}

#[test]
fn update_persists_and_notifies_subscribers() {
    let path = temp_file("update");
    let store = SettingsStore::load(path.clone());
    let changes = store.subscribe();

    store
        .update(|settings| settings.streamer.is_enabled = true)
        .expect("writable");

    assert!(changes.has_changed().expect("sender alive"));
    assert!(SettingsStore::load(path).get().streamer.is_enabled);
}

#[test]
fn volume_is_clamped_and_never_left_unreadable() {
    let mut settings = Settings::default();
    settings.interface.ui_volume = 4.0;
    assert_eq!(settings.normalized().interface.ui_volume, 1.0);

    let mut settings = Settings::default();
    settings.interface.ui_volume = f64::NAN;
    assert_eq!(settings.normalized().interface.ui_volume, DEFAULT_UI_VOLUME);
}
