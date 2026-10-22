use riot::api::pd::entitlements::{
    ITEM_TYPE_AGENTS, ITEM_TYPE_BUDDIES, ITEM_TYPE_PLAYER_TITLES, ITEM_TYPE_SKIN_LEVELS,
};

use super::cache::{self, MAX_AGE_SECS, Manifest};
use super::fixtures;
use super::images::{
    cache_file, content_type_for, image_url, is_allowed_source, source_from_request_uri, url_hash,
};
use super::index::{AssetIndex, AssetSource, Documents};
use super::{Endpoint, ItemKind};

fn index() -> AssetIndex {
    AssetIndex::from_documents(&fixtures::documents(), AssetSource::Cache)
}

#[test]
fn bundled_index_has_agent_and_map_names_only() {
    let bundled = AssetIndex::bundled();

    let jett = bundled
        .agent("ADD6443A-41BD-E414-F6AD-E58D267F4E95")
        .expect("jett");
    assert_eq!(jett.display_name, "Jett");
    assert!(jett.display_icon.is_none());
    assert_eq!(
        bundled
            .map_by_internal_id("Duality")
            .expect("bind")
            .display_name,
        "Bind"
    );
    assert_eq!(bundled.source(), AssetSource::Bundled);
}

#[test]
fn agent_lookup_is_case_insensitive_and_tolerates_nulls() {
    let index = index();

    let jett = index
        .agent("add6443a-41bd-e414-f6ad-e58d267f4e95")
        .expect("jett");
    assert_eq!(
        jett.role.as_ref().map(|role| role.display_name.as_str()),
        Some("Duelist")
    );
    assert!(jett.full_portrait.is_none());
    assert_eq!(jett.abilities[0].display_name, "Cloudburst");
    let kayo = index
        .agent("601DBBE7-43CE-BE57-2A40-4ABD24953621")
        .expect("kayo");
    assert!(kayo.abilities.is_empty());
}

#[test]
fn map_internal_id_comes_from_map_url() {
    let index = index();

    assert_eq!(
        index.map_by_internal_id("Duality").expect("bind").uuid,
        "map-bind"
    );
    assert!(index.map_by_internal_id("duality").is_none());
    assert_eq!(index.maps().len(), 1);
}

#[test]
fn skin_levels_and_chromas_resolve_to_their_skin_and_weapon() {
    let index = index();

    let level = index.skin_level("level-1").expect("level 1");
    assert_eq!(level.display_name, "Reaver Vandal");
    let skin = index.skin_of_level("LEVEL-2").expect("parent skin");
    assert_eq!(skin.uuid, "skin-reaver");
    assert!(
        index
            .chroma("chroma-1")
            .expect("chroma")
            .streamed_video
            .is_some()
    );
    assert_eq!(
        index
            .weapon_of_skin("skin-reaver")
            .expect("weapon")
            .display_name,
        "Vandal"
    );
}

#[test]
fn tier_uses_the_latest_table() {
    let index = index();

    assert_eq!(index.tier(24).expect("immortal 1").tier_name, "IMMORTAL 1");
    assert!(index.tier(0).expect("unranked").small_icon.is_none());
    assert_eq!(index.summary().tiers, 2);
}

#[test]
fn level_border_for_picks_highest_starting_level_not_above() {
    let index = index();

    assert_eq!(
        index.level_border_for(35).expect("border").uuid,
        "border-20"
    );
    assert_eq!(
        index.level_border_for(500).expect("border").uuid,
        "border-50"
    );
}

#[test]
fn item_by_type_and_id_maps_entitlement_types() {
    let index = index();

    let level = index
        .item_by_type_and_id(ITEM_TYPE_SKIN_LEVELS, "LEVEL-1")
        .expect("skin level");
    assert_eq!(
        (level.kind, level.name.as_str()),
        (ItemKind::SkinLevel, "Reaver Vandal")
    );
    let buddy = index
        .item_by_type_and_id(ITEM_TYPE_BUDDIES, "buddy-level-1")
        .expect("buddy");
    assert_eq!(buddy.name, "Ace Buddy");
    let agent = index
        .item_by_type_and_id(
            &ITEM_TYPE_AGENTS.to_uppercase(),
            "add6443a-41bd-e414-f6ad-e58d267f4e95",
        )
        .expect("agent");
    assert_eq!(agent.name, "Jett");
}

#[test]
fn player_title_item_uses_title_text_and_unknown_ids_are_none() {
    let index = index();

    let title = index
        .item_by_type_and_id(ITEM_TYPE_PLAYER_TITLES, "title-1")
        .expect("title");
    assert_eq!(title.name, "Radiant");
    assert_eq!(
        index.item_by_type_and_id(ITEM_TYPE_PLAYER_TITLES, "missing"),
        None
    );
    assert_eq!(index.item_by_type_and_id("not-a-type", "title-1"), None);
}

#[test]
fn queue_and_currency_lookups() {
    let index = index();

    assert_eq!(
        index.queue("Swiftplay").expect("queue").display_name,
        "Swiftplay"
    );
    let vp = index
        .currency("85AD13F7-3D1B-5128-9EB2-7CD8EE0B5741")
        .expect("vp");
    assert_eq!(vp.display_name, "VALORANT Points");
}

#[test]
fn bad_document_leaves_that_table_empty_without_failing() {
    let mut docs: Documents = fixtures::documents();
    docs.insert(Endpoint::Agents, "not json".to_owned());
    docs.insert(Endpoint::Maps, r#"{"status":404,"data":[]}"#.to_owned());

    let index = AssetIndex::from_documents(&docs, AssetSource::Network);

    assert_eq!(index.summary().agents, 0);
    assert_eq!(index.summary().maps, 0);
    assert_eq!(index.summary().skins, 1);
    assert_eq!(index.status().manifest_id.as_deref(), Some("MANIFEST-A"));
}

#[test]
fn manifest_is_current_only_for_same_id_within_seven_days() {
    let manifest = Manifest {
        manifest_id: "A".to_owned(),
        fetched_at: 1_000,
    };

    assert!(manifest.is_current("A", 1_000 + MAX_AGE_SECS - 1));
    assert!(!manifest.is_current("A", 1_000 + MAX_AGE_SECS));
    assert!(!manifest.is_current("B", 1_001));
}

#[test]
fn disk_cache_round_trips_documents_and_manifest() {
    let dir = std::env::temp_dir().join(format!("valoshell-assets-{}", std::process::id()));
    let manifest = Manifest {
        manifest_id: "A".to_owned(),
        fetched_at: 5,
    };

    cache::write_document(&dir, Endpoint::Agents, fixtures::AGENTS).expect("write agents");
    cache::write_manifest(&dir, &manifest).expect("write manifest");

    let docs = cache::read_documents(&dir);
    assert_eq!(docs.len(), 1);
    assert_eq!(cache::read_manifest(&dir), Some(manifest));
    assert_eq!(
        AssetIndex::from_documents(&docs, AssetSource::Cache)
            .summary()
            .agents,
        2
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn image_url_round_trips_through_the_request_uri() {
    let original = "https://media.valorant-api.com/agents/a b/displayicon.png?x=1&y=2";

    let url = image_url(original);

    assert!(url.contains(&url_hash(original)));
    assert_eq!(source_from_request_uri(&url).as_deref(), Some(original));
}

#[test]
fn image_sources_are_limited_to_valorant_media() {
    assert!(is_allowed_source("https://media.valorant-api.com/x.png"));
    assert!(!is_allowed_source("http://media.valorant-api.com/x.png"));
    assert!(!is_allowed_source(
        "https://evil.example/https://media.valorant-api.com/x.png"
    ));
}

#[test]
fn content_type_follows_the_extension_and_cache_file_is_stable() {
    assert_eq!(content_type_for("https://m/x/y.PNG?z=1"), "image/png");
    assert_eq!(content_type_for("https://m/x/y.mp4"), "video/mp4");
    assert_eq!(
        content_type_for("https://m/x/y"),
        "application/octet-stream"
    );
    let first = cache_file(std::path::Path::new("c"), "https://m/x/y.png");
    assert_eq!(
        first,
        cache_file(std::path::Path::new("c"), "https://m/x/y.png")
    );
    assert!(first.to_string_lossy().ends_with(".png"));
}
