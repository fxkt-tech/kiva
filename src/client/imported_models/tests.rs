use super::catalog::*;
use serde_json::Value;
use std::collections::HashSet;

fn document(bytes: &[u8]) -> Value {
    assert_eq!(&bytes[..4], b"glTF");
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 2);
    assert_eq!(
        u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        bytes.len()
    );
    let size = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    serde_json::from_slice(&bytes[20..20 + size]).unwrap()
}

#[test]
fn every_packaged_model_has_its_authored_clips_and_only_visible_material_geometry() {
    let models: [&[u8]; 6] = [
        include_bytes!("../../../assets/models/league/jarvan-base.glb"),
        include_bytes!("../../../assets/models/league/ashe-project.glb"),
        include_bytes!("../../../assets/models/league/ha_orderminionmelee.glb"),
        include_bytes!("../../../assets/models/league/ha_chaosminionmelee.glb"),
        include_bytes!("../../../assets/models/league/ha_orderminionranged.glb"),
        include_bytes!("../../../assets/models/league/ha_chaosminionranged.glb"),
    ];
    for (profile, bytes) in PROFILES.iter().zip(models) {
        let model = document(bytes);
        let clips: HashSet<_> = model["animations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|clip| clip["name"].as_str().unwrap())
            .collect();
        for clip in profile.clips() {
            assert!(clips.contains(clip), "{} missing {clip}", profile.path);
        }
        assert!(!model["skins"][0]["joints"].as_array().unwrap().is_empty());
        assert!(!model["images"].as_array().unwrap().is_empty());
        assert!(
            model["extensionsRequired"]
                .as_array()
                .is_none_or(Vec::is_empty)
        );
        for mesh in model["meshes"].as_array().unwrap() {
            for primitive in mesh["primitives"].as_array().unwrap() {
                let material = primitive["material"].as_u64().unwrap() as usize;
                assert_ne!(
                    model["materials"][material]["extras"]["visible"].as_bool(),
                    Some(false)
                );
                let position = primitive["attributes"]["POSITION"].as_u64().unwrap() as usize;
                let count = model["accessors"][position]["count"].as_u64().unwrap();
                let indices = primitive["indices"].as_u64().unwrap() as usize;
                let bounds = &model["accessors"][indices];
                assert_eq!(bounds["min"][0].as_u64(), Some(0));
                assert_eq!(bounds["max"][0].as_u64(), Some(count - 1));
                assert!(count > 0 && count <= bounds["count"].as_u64().unwrap());
            }
        }
    }
}

#[test]
fn imported_profiles_follow_explicit_mode_identity_and_team() {
    assert_eq!(
        profile(
            Some(crate::gameplay::UnitRole::Hero),
            Some(crate::simulation::CharacterId(1)),
            crate::simulation::Team::Ally
        ),
        Some(0)
    );
    assert_eq!(
        profile(
            Some(crate::gameplay::UnitRole::Hero),
            Some(crate::simulation::CharacterId(2)),
            crate::simulation::Team::Enemy
        ),
        Some(1)
    );
    assert_eq!(
        profile(
            None,
            Some(crate::simulation::CharacterId(1)),
            crate::simulation::Team::Ally
        ),
        None,
        "generic simulation rigs retain their own representation"
    );
    for role in [
        crate::gameplay::UnitRole::MeleeMinion,
        crate::gameplay::UnitRole::RangedMinion,
    ] {
        let ally = profile(Some(role), None, crate::simulation::Team::Ally).unwrap();
        let enemy = profile(Some(role), None, crate::simulation::Team::Enemy).unwrap();
        assert_ne!(ally, enemy);
        assert_eq!(PROFILES[ally].skin, "Order");
        assert_eq!(PROFILES[enemy].skin, "Chaos");
    }
}

#[test]
fn icon_mapping_follows_ability_identity_even_when_slots_change() {
    use crate::simulation::{AbilityId, CharacterId};
    assert_eq!(
        ability_icon(AbilityId(23)),
        Some("assets/icons/league/ashe-e.png")
    );
    assert_eq!(
        ability_icon(AbilityId(11)),
        Some("assets/icons/league/jarvan-q.png")
    );
    assert_eq!(ability_icon(AbilityId(65535)), None);
    assert_eq!(
        portrait(CharacterId(2)),
        Some("assets/icons/league/ashe-project.jpg")
    );
}
