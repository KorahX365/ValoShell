use super::*;
use std::collections::HashSet;

#[test]
fn tables_have_expected_sizes_and_unique_keys() {
    let names: HashSet<_> = FALLBACK_AGENTS.iter().map(|entry| entry.name).collect();
    let uuids: HashSet<_> = FALLBACK_AGENTS.iter().map(|entry| entry.uuid).collect();
    let internal: HashSet<_> = FALLBACK_MAPS
        .iter()
        .map(|entry| entry.internal_id)
        .collect();
    assert_eq!((names.len(), uuids.len(), internal.len()), (29, 29, 18));
}

#[test]
fn agent_uuids_are_lowercase_hyphenated_uuids() {
    for entry in FALLBACK_AGENTS {
        assert_eq!(entry.uuid.len(), 36, "{}", entry.name);
        assert_eq!(entry.uuid, entry.uuid.to_lowercase());
        assert_eq!(entry.uuid.matches('-').count(), 4);
    }
}

#[test]
fn agent_tables_are_sorted_like_the_live_api() {
    assert!(
        FALLBACK_AGENTS
            .windows(2)
            .all(|pair| pair[0].name < pair[1].name)
    );
    assert!(
        FALLBACK_MAPS
            .windows(2)
            .all(|pair| pair[0].name < pair[1].name)
    );
}

#[test]
fn agent_by_name_is_case_insensitive() {
    assert_eq!(
        agent_by_name("jett").map(|a| a.uuid),
        Some("add6443a-41bd-e414-f6ad-e58d267f4e95")
    );
    assert_eq!(agent_by_name(" Sova ").map(|a| a.name), Some("SOVA"));
    assert_eq!(agent_by_name("nobody"), None);
}

#[test]
fn kay_o_is_found_with_slash_underscore_or_space() {
    for spelling in ["KAY/O", "kay/o", "Kay/O", "KAY_O", "kay o"] {
        assert_eq!(
            agent_by_name(spelling).map(|a| a.name),
            Some("KAY/O"),
            "{spelling}"
        );
    }
}

#[test]
fn agent_display_name_keeps_the_slash_capitalised() {
    assert_eq!(
        agent_by_name("KAY/O").map(|a| a.display_name()),
        Some("Kay/O".to_owned())
    );
    assert_eq!(
        agent_by_name("JETT").map(|a| a.display_name()),
        Some("Jett".to_owned())
    );
}

#[test]
fn agent_by_uuid_ignores_case() {
    let upper = "ADD6443A-41BD-E414-F6AD-E58D267F4E95";
    assert_eq!(agent_by_uuid(upper).map(|a| a.name), Some("JETT"));
    assert_eq!(agent_by_uuid("00000000"), None);
}

#[test]
fn agent_id_is_the_uuid() {
    let jett = agent_by_name("JETT").map(AgentEntry::agent_id);
    assert_eq!(
        jett.as_ref().map(AgentId::as_str),
        Some("add6443a-41bd-e414-f6ad-e58d267f4e95")
    );
}

#[test]
fn map_internal_ids_match_the_spec() {
    let pairs = [
        ("BIND", "Duality"),
        ("HAVEN", "Triad"),
        ("ICEBOX", "Port"),
        ("SPLIT", "Bonsai"),
    ];
    for (name, internal) in pairs {
        assert_eq!(map_by_name(name).map(|m| m.internal_id), Some(internal));
        assert_eq!(map_by_internal_id(internal).map(|m| m.name), Some(name));
    }
}

#[test]
fn map_by_name_accepts_display_names() {
    assert_eq!(map_by_name("Bind").map(|m| m.name), Some("BIND"));
    assert_eq!(map_by_name("ascent").map(|m| m.name), Some("ASCENT"));
    assert_eq!(map_by_name("The Range"), None);
}

#[test]
fn map_by_internal_id_is_case_sensitive() {
    assert_eq!(map_by_internal_id("duality"), None);
    assert_eq!(map_by_internal_id(""), None);
}

#[test]
fn map_by_map_id_path_uses_second_to_last_segment() {
    assert_eq!(
        map_by_map_id_path("/Game/Maps/Duality/Duality").map(|m| m.name),
        Some("BIND")
    );
    let hurm = map_by_map_id_path("/Game/Maps/HURM/HURM_Alley/HURM_Alley");
    assert_eq!(hurm.map(|m| m.name), Some("DISTRICT"));
    assert_eq!(map_by_map_id_path("/Game/Maps/Mystery/Mystery"), None);
    assert_eq!(map_by_map_id_path("Duality"), None);
}

#[test]
fn team_deathmatch_maps_are_the_hurm_ones() {
    let hurm: Vec<_> = FALLBACK_MAPS
        .iter()
        .filter(|m| m.is_team_deathmatch())
        .map(|m| m.name)
        .collect();
    assert_eq!(hurm, ["DISTRICT", "DRIFT", "GLITCH", "KASBAH", "PIAZZA"]);
}

#[test]
fn map_display_name_resolves_internal_ids() {
    assert_eq!(map_display_name("Duality"), "Bind");
    assert_eq!(map_display_name("duality"), "Bind");
    assert_eq!(map_display_name("HURM_Alley"), "District");
    assert_eq!(map_display_name("hurmalley"), "District");
}

#[test]
fn map_display_name_titles_unknown_ids_and_passes_empty_through() {
    assert_eq!(map_display_name("New_Map"), "New Map");
    assert_eq!(map_display_name(""), "");
    assert_eq!(map_display_name("Unknown"), "Unknown");
}
