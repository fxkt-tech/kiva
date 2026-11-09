//! Display roots consume authoritative snapshots; the environment draws shared level geometry.

use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{Body, Combatant, HitStop, PhysicsPose, SimulationActive},
};

use super::{ActorView, PlayerView, PropView, controls::ViewYaw};

pub(super) fn setup(world: &mut World) {
    super::environment::setup(world);
    spawn_player(world);
}

pub(super) fn sync_player(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    hit_stop: Res<HitStop>,
    view: Res<ViewYaw>,
    players: Query<(&Body, &Combatant), With<Player>>,
    mut views: Query<(&ActorView, &mut Transform), With<PlayerView>>,
) {
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    for (view_root, mut transform) in &mut views {
        let Ok((body, combat)) = players.get(view_root.owner) else {
            continue;
        };
        transform.translation = body.interpolated_world(alpha);
        let yaw = combat
            .sniper_sequence()
            .and_then(|sequence| {
                let direction = (sequence.aim_position() - transform.translation).xz();
                (direction.length_squared() > 0.0001).then(|| (-direction.x).atan2(-direction.y))
            })
            .unwrap_or(view.yaw());
        transform.rotation = Quat::from_rotation_y(yaw);
    }
}

pub(super) fn sync_props(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    hit_stop: Res<HitStop>,
    props: Query<&PhysicsPose>,
    mut views: Query<(&PropView, &mut Transform)>,
) {
    let alpha = super::display_alpha(&time, &active, &hit_stop);
    for (view, mut transform) in &mut views {
        if let Ok(pose) = props.get(view.owner) {
            transform.translation = pose.interpolated_position(alpha);
            transform.rotation = pose.interpolated_rotation(alpha);
        }
    }
}

fn spawn_player(world: &mut World) {
    let mut players = world.query_filtered::<(Entity, &Body), With<Player>>();
    let (player, body) = players.single(world).expect("one demo player");
    let position = body.world_position();
    let view = world
        .spawn((
            ActorView { owner: player },
            PlayerView,
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();
    let (body_material, eyes_material) = {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        (
            materials.add(Color::srgb(0.10, 0.56, 0.56)),
            materials.add(Color::srgb(1.0, 0.64, 0.23)),
        )
    };
    super::stickman::spawn(world, view, &body_material, &eyes_material);
}
