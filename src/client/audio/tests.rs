use super::*;
use crate::gameplay::{self, MatchState};

fn arena() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin, CombatAudioPlugin));
    gameplay::prepare_world(app.world_mut());
    let handle = gameplay::start_match(app.world_mut(), CharacterId(1)).unwrap();
    app.insert_resource(ClientSession::Active {
        player: handle.player,
        selection_revision: 1,
        match_id: handle.id,
    });
    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    (app, handle.player)
}

#[test]
fn terrain_recast_does_not_replay_the_initial_leap_sound() {
    let (mut app, player) = arena();
    let origin = app.world().get::<UnitPose>(player).unwrap().position();
    let target = hero(
        &mut app,
        2,
        Team::Enemy,
        origin + Vec2::NEG_Y * 4.0,
        LoadoutSelection::CharacterDefault,
    );
    command(
        &mut app,
        player,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::Unit(target),
        },
    );
    step(&mut app, 100);
    let tick = app.world().resource::<SimulationTick>().0;
    assert!(
        app.world()
            .get::<UnitMechanics>(player)
            .unwrap()
            .recast_ready(3, tick)
    );
    let initial = pending(&app)
        .iter()
        .filter(|cue| cue.cue == "jarvan.r")
        .count();
    assert_eq!(initial, 1);
    command(
        &mut app,
        player,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(!app.world().get::<UnitMechanics>(player).unwrap().recast(3));
    assert_eq!(
        pending(&app)
            .iter()
            .filter(|cue| cue.cue == "jarvan.r")
            .count(),
        initial
    );
}

fn command(app: &mut App, actor: Entity, command: UnitCommand) {
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(actor, command);
}

fn step(app: &mut App, count: usize) {
    for _ in 0..count {
        app.world_mut().run_schedule(FixedUpdate);
    }
}

fn pending(app: &App) -> Vec<AudioCue> {
    app.world().resource::<AudioRuntime>().pending.clone()
}

fn hero(
    app: &mut App,
    character: u16,
    team: Team,
    position: Vec2,
    loadout: LoadoutSelection,
) -> Entity {
    spawn_character(
        app.world_mut(),
        CharacterSpawnSpec {
            character: CharacterId(character),
            team,
            position,
            loadout,
            respawn_delay: Some(0.1),
        },
    )
    .unwrap()
}

#[test]
fn real_commit_plays_once_while_rejected_cast_is_silent() {
    let (mut app, player) = arena();
    command(
        &mut app,
        player,
        UnitCommand::CastAbility {
            slot: 3,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(
        pending(&app).is_empty(),
        "a rejected R must not produce audio"
    );
    command(
        &mut app,
        player,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert_eq!(
        pending(&app)
            .iter()
            .filter(|cue| cue.cue == "jarvan.w")
            .count(),
        1
    );
    step(&mut app, 3);
    assert_eq!(
        pending(&app)
            .iter()
            .filter(|cue| cue.cue == "jarvan.w")
            .count(),
        1,
        "each fixed-step cursor consumes a committed result once"
    );
}

#[test]
fn mixed_loadout_uses_the_committed_ability_instead_of_character_or_slot() {
    let (mut app, player) = arena();
    let position = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 3.0;
    let ashe = hero(
        &mut app,
        2,
        Team::Ally,
        position,
        LoadoutSelection::Explicit([Some(AbilityId(12)), None, None, None]),
    );
    command(
        &mut app,
        ashe,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    let cues = pending(&app);
    assert_eq!(cues.len(), 1);
    assert_eq!(cues[0].cue, "jarvan.w");
    assert_eq!(cues[0].source, ashe.to_bits().to_string());
}

#[test]
fn real_empowered_attacks_use_project_burst_audio_after_the_attack_commits() {
    let (mut app, player) = arena();
    let position = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 3.0;
    let ashe = hero(
        &mut app,
        2,
        Team::Ally,
        position,
        LoadoutSelection::CharacterDefault,
    );
    let target = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Audio target".into(),
            team: Team::Enemy,
            position: position + Vec2::X * 3.0,
            stats: UnitStats {
                max_health: 100_000.0,
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    command(&mut app, ashe, UnitCommand::AttackTarget(target));
    for _ in 0..600 {
        step(&mut app, 1);
        if app.world().get::<UnitMechanics>(ashe).unwrap().focus() >= 4 {
            break;
        }
    }
    assert_eq!(app.world().get::<UnitMechanics>(ashe).unwrap().focus(), 4);
    command(
        &mut app,
        ashe,
        UnitCommand::CastAbility {
            slot: 0,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(pending(&app).iter().any(|cue| cue.cue == "ashe.q"));
    assert!(app.world().get::<UnitMechanics>(ashe).unwrap().empowered());
    assert!(
        !pending(&app)
            .iter()
            .any(|cue| cue.cue == "ashe.empowered-attack")
    );
    command(&mut app, ashe, UnitCommand::AttackTarget(target));
    for _ in 0..150 {
        step(&mut app, 1);
        if pending(&app)
            .iter()
            .any(|cue| cue.cue == "ashe.empowered-attack")
        {
            return;
        }
    }
    panic!("the actual empowered basic attack must play its burst cue");
}

#[test]
fn death_and_respawn_between_display_frames_keep_each_original_life_and_position() {
    let (mut app, player) = arena();
    let home = app.world().get::<UnitPose>(player).unwrap().position();
    let destination = home + Vec2::X * 2.0;
    command(&mut app, player, UnitCommand::MoveTo(destination));
    step(&mut app, 60);
    assert!(
        app.world()
            .get::<UnitPose>(player)
            .unwrap()
            .position()
            .distance(home)
            > 1.5
    );
    let enemy = hero(
        &mut app,
        1,
        Team::Enemy,
        home + Vec2::X * 4.5,
        LoadoutSelection::CharacterDefault,
    );
    let changes = &mut app.world_mut().resource_mut::<AttributeChanges>();
    let modifier = changes.allocate_source().unwrap();
    changes.upsert(
        enemy,
        modifier,
        AttributeModifier {
            flat: AttributeDelta {
                attack_damage: 10_000.0,
                attack_range: 10.0,
                ..default()
            },
            ..default()
        },
    );
    command(&mut app, enemy, UnitCommand::AttackTarget(player));
    for _ in 0..90 {
        step(&mut app, 1);
        if !app.world().get::<UnitState>(player).unwrap().alive() {
            break;
        }
    }
    assert!(!app.world().get::<UnitState>(player).unwrap().alive());
    let death = pending(&app)
        .into_iter()
        .find(|cue| cue.source == player.to_bits().to_string() && cue.cue == "jarvan.death")
        .expect("real lethal attack notification");
    command(&mut app, enemy, UnitCommand::Stop);
    step(&mut app, 181);
    assert!(app.world().get::<UnitState>(player).unwrap().alive());
    let respawn = pending(&app)
        .into_iter()
        .find(|cue| cue.cue == "shared.respawn")
        .unwrap();
    assert_ne!(death.life, respawn.life);
    assert!(death.position.distance(respawn.position) > 1.5);
    assert!(respawn.position.distance(home) < 0.01);
    let tick = app.world().resource::<SimulationTick>().0;
    let presented = app
        .world_mut()
        .resource_mut::<AudioRuntime>()
        .drain(tick, false, home);
    assert!(
        presented.iter().all(|cue| cue.cue != "jarvan.death"),
        "a long render stall must not replay a death that has already respawned"
    );
    assert!(presented.iter().any(|cue| cue.cue == "shared.respawn"));
}

#[test]
fn actual_hidden_enemy_cast_is_filtered_before_leaving_rust() {
    let (mut app, player) = arena();
    let hidden = hero(
        &mut app,
        1,
        Team::Enemy,
        Vec2::new(20.0, -20.0),
        LoadoutSelection::CharacterDefault,
    );
    command(
        &mut app,
        hidden,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(!team_can_see(app.world(), Team::Ally, hidden));
    assert!(pending(&app).is_empty());
    assert!(
        app.world().get::<UnitMechanics>(hidden).unwrap().shield() > 0.0,
        "the hidden cast really committed; silence must be observation filtering"
    );

    let position = app.world().get::<UnitPose>(player).unwrap().position() + Vec2::X * 4.0;
    let visible = hero(
        &mut app,
        1,
        Team::Enemy,
        position,
        LoadoutSelection::CharacterDefault,
    );
    command(
        &mut app,
        visible,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(team_can_see(app.world(), Team::Ally, visible));
    assert_eq!(
        pending(&app)
            .iter()
            .filter(|cue| cue.source == visible.to_bits().to_string())
            .count(),
        1
    );
}

#[test]
fn pause_and_a_new_match_discard_pending_old_audio() {
    let (mut app, player) = arena();
    command(
        &mut app,
        player,
        UnitCommand::CastAbility {
            slot: 1,
            target: AbilityTarget::SelfCast,
        },
    );
    step(&mut app, 1);
    assert!(!pending(&app).is_empty());
    app.world_mut().resource_mut::<SimulationControl>().paused = true;
    publish(app.world_mut());
    assert!(pending(&app).is_empty());

    app.world_mut().resource_mut::<SimulationControl>().paused = false;
    step(&mut app, 20);
    assert!(
        pending(&app).is_empty(),
        "resume cannot replay consumed events"
    );
    let old_session = app.world().resource::<AudioRuntime>().session;
    gameplay::reset_match(app.world_mut());
    let next = gameplay::start_match(app.world_mut(), CharacterId(2)).unwrap();
    app.insert_resource(ClientSession::Active {
        player: next.player,
        selection_revision: 2,
        match_id: next.id,
    });
    assert_ne!(Some(next.id), old_session.map(|s| s.match_id));
    publish(app.world_mut());
    assert_eq!(
        app.world()
            .resource::<AudioRuntime>()
            .session
            .unwrap()
            .player,
        next.player
    );
    assert!(pending(&app).is_empty());
    assert_eq!(app.world().resource::<MatchState>().id(), Some(next.id));
}

fn sample(priority: u8, tick: u64, personal: bool, position: Vec2) -> AudioCue {
    AudioCue {
        sequence: tick,
        tick,
        source: "7".into(),
        life: "0".into(),
        cue: "jarvan.attack".into(),
        variant: 0,
        gain: 0.7,
        pan: 0.0,
        priority,
        position,
        personal,
    }
}

#[test]
fn long_display_catchup_is_bounded_and_prefers_recent_life_notifications() {
    let mut runtime = AudioRuntime::default();
    for tick in 0..80 {
        runtime.push(sample(1, tick, false, Vec2::ZERO));
    }
    runtime.push(sample(3, 80, true, Vec2::new(100.0, 100.0)));
    assert_eq!(runtime.pending.len(), MAX_PENDING);
    let cues = runtime.drain(80, false, Vec2::ZERO);
    assert!(cues.iter().all(|cue| cue.tick >= 65));
    assert!(
        cues.iter().any(|cue| cue.priority == 3 && cue.personal),
        "the player's death/respawn cue is audible away from the camera"
    );
    assert!(runtime.pending.is_empty());
}

#[test]
fn distance_attenuation_is_cosmetic_and_variants_do_not_repeat_back_to_back() {
    let mut runtime = AudioRuntime::default();
    runtime.push(sample(1, 1, false, Vec2::ZERO));
    runtime.push(sample(1, 1, false, Vec2::X * 10.0));
    runtime.push(sample(1, 1, false, Vec2::X * HEARING_DISTANCE));
    let cues = runtime.drain(1, false, Vec2::ZERO);
    assert_eq!(cues.len(), 2);
    assert!(cues[0].gain > cues[1].gain && cues[1].gain > 0.0);
    assert!(cues[1].pan > 0.0);
    let variants: Vec<_> = (0..16)
        .map(|_| runtime.variant("jarvan.attack", 2))
        .collect();
    assert!(variants.iter().all(|index| *index < 2));
    assert!(variants.windows(2).all(|pair| pair[0] != pair[1]));
}
