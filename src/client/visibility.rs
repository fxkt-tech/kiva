//! One authoritative observation drives models, labels, HUD and field effects.
use super::session::ClientSession;
use crate::simulation::*;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(super) struct VisibleWorld {
    restricted: bool,
    entities: HashSet<Entity>,
}
impl VisibleWorld {
    pub fn contains(&self, entity: Entity) -> bool {
        !self.restricted || self.entities.contains(&entity)
    }
}

pub(super) fn refresh(world: &mut World) {
    let team = world
        .get_resource::<ClientSession>()
        .and_then(ClientSession::player)
        .and_then(|player| world.get::<Unit>(player))
        .map(Unit::team);
    let Some(team) = team else {
        world.insert_resource(VisibleWorld::default());
        return;
    };
    let ids: Vec<_> = world
        .query_filtered::<Entity, Or<(
            With<Unit>,
            With<Projectile>,
            With<BattlefieldObject>,
            With<AreaEffect>,
        )>>()
        .iter(world)
        .collect();
    let entities = ids
        .into_iter()
        .filter(|id| {
            if let Some(projectile) = world.get::<Projectile>(*id) {
                return projectile.team() == team
                    || point_visible(world, team, projectile.position());
            }
            if let Some(area) = world.get::<AreaEffect>(*id) {
                return area.team() == team || point_visible(world, team, area.position());
            }
            team_can_see(world, team, *id)
        })
        .collect();
    world.insert_resource(VisibleWorld {
        restricted: true,
        entities,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            content,
            presentation::{self, PresentationFrame},
            scene::{self, UnitVisual},
            session,
        },
        gameplay::*,
    };

    #[test]
    fn scout_vision_controls_real_unit_roots_and_expires_without_leaking_hidden_enemies() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin, GameplayPlugin))
            .init_resource::<VisibleWorld>()
            .init_resource::<PresentationFrame>()
            .add_systems(
                Update,
                (refresh, presentation::prepare, scene::sync_units).chain(),
            );
        content::setup(app.world_mut());
        app.world_mut()
            .resource_mut::<crate::client::content::SelectionSettings>()
            .default_character = CharacterId(2);
        session::setup(app.world_mut());
        session::start_native(app.world_mut());
        let player = app.world().resource::<ClientSession>().player().unwrap();
        let enemy = app
            .world()
            .resource::<MatchState>()
            .base(Team::Enemy)
            .unwrap();
        let destination = app.world().get::<UnitPose>(enemy).unwrap().position();
        for id in [player, enemy] {
            app.world_mut().entity_mut(id).insert((
                UnitVisual,
                Transform::default(),
                Visibility::Inherited,
            ));
        }
        app.world_mut().run_schedule(Update);
        assert!(!app.world().resource::<VisibleWorld>().contains(enemy));
        assert_eq!(
            *app.world().get::<Visibility>(enemy).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().get::<Visibility>(player).unwrap(),
            Visibility::Inherited
        );
        app.world_mut().resource_mut::<CommandQueue>().submit(
            player,
            UnitCommand::CastAbility {
                slot: 2,
                target: AbilityTarget::Point(destination),
            },
        );
        for _ in 0..400 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().run_schedule(Update);
        assert!(app.world().resource::<VisibleWorld>().contains(enemy));
        assert_eq!(
            *app.world().get::<Visibility>(enemy).unwrap(),
            Visibility::Inherited
        );
        app.world_mut().resource_mut::<SimulationControl>().paused = true;
        for _ in 0..500 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().run_schedule(Update);
        assert!(
            app.world().resource::<VisibleWorld>().contains(enemy),
            "pause must preserve actual scout vision"
        );
        app.world_mut().resource_mut::<SimulationControl>().paused = false;
        for _ in 0..600 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().run_schedule(Update);
        assert!(!app.world().resource::<VisibleWorld>().contains(enemy));
        assert_eq!(
            *app.world().get::<Visibility>(enemy).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn standalone_presentation_without_player_observation_keeps_full_visibility() {
        let mut world = World::new();
        refresh(&mut world);
        let id = world.spawn_empty().id();
        assert!(world.resource::<VisibleWorld>().contains(id));
    }
}
