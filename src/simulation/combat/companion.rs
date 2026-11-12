//! The logical companion: no body, health or AI. She performs the ranged
//! skills at a position the player's action recorded when it started.
use super::hit::CombatWorld;
use super::{Combatant, Health};
use crate::simulation::{Body, Companion, CompanionState};
use avian3d::prelude::SpatialQueryFilter;
use bevy::prelude::*;

pub(crate) fn ensure_companions(world: &mut World) {
    let missing: Vec<_> = world
        .query::<(Entity, &Combatant)>()
        .iter(world)
        .filter(|(_, c)| {
            c.companion
                .is_none_or(|e| world.get::<Companion>(e).is_none())
        })
        .map(|(e, _)| e)
        .collect();
    for owner in missing {
        let entity = world
            .spawn((Companion { owner }, CompanionState::default()))
            .id();
        world.get_mut::<Combatant>(owner).unwrap().companion = Some(entity);
    }
    let orphaned: Vec<_> = world
        .query::<(Entity, &Companion)>()
        .iter(world)
        .filter(|(_, c)| world.get::<Combatant>(c.owner).is_none())
        .map(|(e, _)| e)
        .collect();
    for e in orphaned {
        world.despawn(e);
    }
}

/// Where the companion stands and fires from: beside and behind the player,
/// on real ground, with a clear line from the player to her and to the muzzle.
pub(super) fn companion_pose(
    world: &CombatWorld,
    body: &Body,
    facing: Vec2,
    owner: Entity,
) -> (Vec3, Vec3) {
    let origin = body.world_position();
    let forward = Vec3::new(facing.x, 0.0, facing.y);
    let side = Vec3::new(-facing.y, 0.0, facing.x);
    let desired = origin + side * 0.95 - forward * 0.45;
    for fraction in [1.0, 0.65, 0.3, 0.0] {
        let feet = origin.lerp(desired, fraction);
        let filter = SpatialQueryFilter::from_excluded_entities([owner]);
        let Some(hit) = world.spatial.cast_ray_predicate(
            feet + Vec3::Y * 1.5,
            Dir3::NEG_Y,
            3.0,
            true,
            &filter,
            &|e| !world.sensors.contains(e),
        ) else {
            continue;
        };
        if hit.normal.y < 0.65 {
            continue;
        }
        let feet = feet + Vec3::Y * (1.5 - hit.distance);
        let muzzle = feet + Vec3::Y * 1.2 + forward * 0.55;
        if world.visible(body.aim_position(), feet + Vec3::Y * 0.9, owner, owner)
            && world.visible(body.aim_position(), muzzle, owner, owner)
        {
            return (feet, muzzle);
        }
    }
    (origin, body.aim_position())
}

/// Publish the acting companion's snapshot for presentation.
pub(in crate::simulation) fn publish_companions(
    players: Query<(&Combatant, &Health)>,
    mut companions: Query<(&Companion, &mut CompanionState)>,
) {
    for (companion, mut state) in &mut companions {
        state.previous_position = state.position;
        let Ok((combat, health)) = players.get(companion.owner) else {
            state.visible = false;
            continue;
        };
        let Some(action) = combat.active_action() else {
            state.visible = false;
            continue;
        };
        state.visible = health.is_alive()
            && !combat.action.as_ref().unwrap().cancelled
            && action.kind.spec().by_companion();
        if state.action_id != action.action_id {
            state.previous_position = action.companion_position;
        }
        state.position = action.companion_position;
        state.source_position = action.source_position;
        state.aim_position = action.aim_position;
        state.facing = (action.aim_position - action.source_position)
            .xz()
            .normalize_or(Vec2::NEG_Y);
        state.pair = action.pair;
        state.action_id = action.action_id;
        state.kind = action.kind;
        state.elapsed = action.elapsed;
        state.shot_count = action.shot_count;
    }
}
