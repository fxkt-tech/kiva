//! Test fixtures use the same public mode and simulation boundaries as clients.
use crate::{gameplay::*, simulation::*};
use bevy::prelude::*;

/// End a real match through a basic attack rather than writing match/health state.
pub(super) fn finish_match(app: &mut App, winner: Team) {
    let losing_team = if winner == Team::Ally {
        Team::Enemy
    } else {
        Team::Ally
    };
    let base = app
        .world()
        .resource::<MatchState>()
        .base(losing_team)
        .unwrap();
    let position = app.world().get::<UnitPose>(base).unwrap().position() + Vec2::X * 3.0;
    let striker = spawn_unit(
        app.world_mut(),
        UnitSpec {
            name: "Terminal attack fixture".into(),
            team: winner,
            position,
            stats: UnitStats {
                attack_delivery: BasicAttackDelivery::Direct,
                base_attributes: BaseAttributes {
                    attack_damage: 10000.0,
                    attack_speed: MAX_ATTACK_SPEED,
                    attack_range: 4.0,
                    ..default()
                },
                ..default()
            },
            abilities: [None; 4],
            respawn_delay: None,
        },
    )
    .unwrap();
    app.world_mut()
        .resource_mut::<CommandQueue>()
        .submit(striker, UnitCommand::AttackTarget(base));
    for _ in 0..2 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<MatchState>().phase(),
        MatchPhase::Finished(MatchOutcome::Winner(winner))
    );
}
