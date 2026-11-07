//! Client intents are checked against the real session and simulation inventory.
use super::super::{content, session};
use super::*;
use serde_json::{Value, json};

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin))
        .init_resource::<EquipmentAdapter>();
    content::setup(app.world_mut());
    session::setup(app.world_mut());
    session::start_native(app.world_mut());
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    process_requests(app.world_mut());
    let player = app.world().resource::<ClientSession>().player().unwrap();
    (app, player)
}

fn equip(player: Entity, slot: usize, id: u16, request_id: u32) -> Value {
    json!({"type":"Equip","player_id":player.to_bits().to_string(),"slot":slot,
        "equipment_id":id,"request_id":request_id})
}
fn unequip(player: Entity, slot: usize, instance: impl ToString, request_id: u32) -> Value {
    json!({"type":"Unequip","player_id":player.to_bits().to_string(),"slot":slot,
        "instance_id":instance.to_string(),"request_id":request_id})
}
fn batch(app: &mut App, requests: Value, paused: bool, captured: bool) {
    process_batch(
        app.world_mut(),
        &requests.to_string(),
        ManagementFacts { paused, captured },
    );
}
fn ack(app: &App) -> Value {
    serde_json::to_value(&app.world().resource::<EquipmentAdapter>().ack).unwrap()
}
fn projection(app: &App) -> Value {
    let world = app.world();
    let player = world.resource::<ClientSession>().player();
    serde_json::to_value(EquipmentView::new(
        player.and_then(|player| {
            Some((
                world.get::<EquipmentSlots>(player)?,
                world.get::<UnitState>(player)?,
                world.get::<ControlState>(player)?,
            ))
        }),
        world.resource::<EquipmentCatalog>(),
        world.resource::<AbilityCatalog>(),
        true,
        &world.resource::<EquipmentAdapter>().ack,
    ))
    .unwrap()
}

#[test]
fn paused_fifo_management_updates_real_attributes_and_projects_six_actual_slots() {
    let (mut app, player) = app();
    let baseline = app
        .world()
        .get::<Unit>(player)
        .unwrap()
        .attributes()
        .attack_damage();
    batch(
        &mut app,
        json!([equip(player, 0, 1, 1), equip(player, 1, 1, 2)]),
        true,
        false,
    );
    assert_eq!(
        ack(&app),
        json!({"request_id":2,"accepted":true,"error":null})
    );
    assert_eq!(
        app.world()
            .get::<Unit>(player)
            .unwrap()
            .attributes()
            .attack_damage(),
        baseline + 40.0
    );
    let view = projection(&app);
    assert_eq!(view["slots"].as_array().unwrap().len(), 6);
    assert_eq!(view["first_empty_slot"], 2);
    let first = view["slots"][0]["item"]["instance_id"].as_str().unwrap();
    let second = view["slots"][1]["item"]["instance_id"].as_str().unwrap();
    assert_ne!(first, second);
    batch(&mut app, json!([unequip(player, 0, first, 3)]), true, false);
    assert_eq!(
        app.world()
            .get::<Unit>(player)
            .unwrap()
            .attributes()
            .attack_damage(),
        baseline + 20.0
    );
    assert_eq!(projection(&app)["first_empty_slot"], 0);
    assert_eq!(projection(&app)["slots"][1]["item"]["instance_id"], second);
}

#[test]
fn running_capture_old_player_and_stale_instances_cannot_mutate_inventory() {
    let (mut app, player) = app();
    for (paused, captured) in [(false, false), (true, true)] {
        batch(&mut app, json!([equip(player, 0, 1, 1)]), paused, captured);
        assert_eq!(ack(&app)["accepted"], false);
        assert!(
            app.world()
                .get::<EquipmentSlots>(player)
                .unwrap()
                .item(0)
                .is_none()
        );
    }
    let mut old = equip(player, 0, 1, 2);
    old["player_id"] = json!("old-player");
    batch(&mut app, json!([old]), true, false);
    assert_eq!(ack(&app)["accepted"], false);
    batch(&mut app, json!([equip(player, 0, 7, 3)]), true, false);
    let original = app
        .world()
        .get::<EquipmentSlots>(player)
        .unwrap()
        .item(0)
        .unwrap()
        .instance();
    for request in [
        equip(player, 0, 2, 4),
        equip(player, 6, 1, 5),
        unequip(player, 0, "stale", 6),
    ] {
        batch(&mut app, json!([request]), true, false);
        assert_eq!(ack(&app)["accepted"], false);
        assert!(
            ack(&app)["error"]
                .as_str()
                .is_some_and(|error| !error.is_empty())
        );
        assert_eq!(
            app.world()
                .get::<EquipmentSlots>(player)
                .unwrap()
                .item(0)
                .unwrap()
                .instance(),
            original
        );
    }
}

#[test]
fn malformed_batches_are_consumed_without_executing_a_valid_prefix() {
    let (mut app, player) = app();
    let mut extra = equip(player, 1, 1, 2);
    extra["flat"] = json!({"attack_damage":999});
    for raw in [
        "{".into(),
        json!([equip(player, 0, 1, 1), extra]).to_string(),
        json!(vec![equip(player, 0, 1, 1); 33]).to_string(),
    ] {
        process_batch(
            app.world_mut(),
            &raw,
            ManagementFacts {
                paused: true,
                captured: false,
            },
        );
        assert_eq!(ack(&app)["request_id"], Value::Null);
        assert_eq!(ack(&app)["accepted"], false);
        assert!(
            app.world()
                .get::<EquipmentSlots>(player)
                .unwrap()
                .item(0)
                .is_none()
        );
    }
    batch(
        &mut app,
        json!([equip(player, 0, 1, 8), equip(player, 0, 2, 9)]),
        true,
        false,
    );
    assert_eq!(ack(&app)["request_id"], 9);
    assert_eq!(ack(&app)["accepted"], false);
    assert_eq!(
        app.world()
            .get::<EquipmentSlots>(player)
            .unwrap()
            .item(0)
            .unwrap()
            .equipment(),
        EquipmentId(1),
        "semantic rejection does not roll back the earlier accepted intent"
    );
}

#[test]
fn selection_and_new_player_reset_ack_and_reject_an_old_round_request() {
    let (mut app, player) = app();
    batch(&mut app, json!([equip(player, 0, 1, 1)]), true, false);
    crate::gameplay::reset_match(app.world_mut());
    session::setup(app.world_mut());
    process_requests(app.world_mut());
    assert_eq!(
        ack(&app),
        json!({"request_id":0,"accepted":true,"error":null})
    );
    let empty = projection(&app);
    assert_eq!(empty["can_manage"], false);
    assert_eq!(empty["first_empty_slot"], Value::Null);
    assert!(
        empty["slots"]
            .as_array()
            .unwrap()
            .iter()
            .all(|slot| slot["item"].is_null())
    );
    session::start_native(app.world_mut());
    process_requests(app.world_mut());
    let new_player = app.world().resource::<ClientSession>().player().unwrap();
    assert_ne!(player, new_player);
    batch(&mut app, json!([equip(player, 0, 1, 2)]), true, false);
    assert_eq!(ack(&app)["accepted"], false);
    assert!(
        app.world()
            .get::<EquipmentSlots>(new_player)
            .unwrap()
            .item(0)
            .is_none()
    );
}

#[test]
fn catalog_and_runtime_share_slot_free_active_metadata_and_never_equip_defaults() {
    let (mut app, player) = app();
    assert_eq!(projection(&app)["first_empty_slot"], 0);
    let catalog = serde_json::to_value(catalog_view(
        app.world().resource::<EquipmentCatalog>(),
        app.world().resource::<AbilityCatalog>(),
    ))
    .unwrap();
    assert_eq!(catalog.as_array().unwrap().len(), 9);
    assert_eq!(catalog[1]["bonuses"], json!(["攻距 +50", "攻速 +25%"]));
    assert!(
        catalog[5]["passives"][0]["description"]
            .as_str()
            .unwrap()
            .contains("20%")
    );
    assert!(catalog[6]["active"].get("slot").is_none());
    batch(&mut app, json!([equip(player, 4, 7, 1)]), true, false);
    let view = projection(&app);
    let runtime = &view["slots"][4]["item"]["active"];
    assert_eq!(runtime["slot"], 4);
    assert_eq!(runtime["cooldown"], 0.0);
    assert_eq!(runtime["available"], true);
    for (key, value) in catalog[6]["active"].as_object().unwrap() {
        assert_eq!(&runtime[key], value);
    }
}
