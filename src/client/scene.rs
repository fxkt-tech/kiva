//! Display roots consume authoritative snapshots; the environment draws shared level geometry.

use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{Body, Combatant, HitStop, SimulationActive},
};

use super::{ActorView, PlayerView, controls::ViewYaw};

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
            .sequence()
            .and_then(|sequence| {
                let direction = (sequence.aim_position() - transform.translation).xz();
                (direction.length_squared() > 0.0001).then(|| (-direction.x).atan2(-direction.y))
            })
            .unwrap_or(view.yaw());
        transform.rotation = Quat::from_rotation_y(yaw);
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
    let appearance = world
        .get_resource::<super::appearance::AppearanceSelection>()
        .map_or(super::avatar::Appearance::Lihuowang, |selection| {
            selection.current()
        });
    super::avatar::spawn_with_appearance(world, view, appearance);
}
