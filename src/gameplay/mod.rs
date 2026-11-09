//! The demo's level data and player assembly, independent of rendering and input.

pub mod terrain;

use bevy::prelude::*;

pub use terrain::TerrainGeometry;

use crate::simulation::{
    ActionIntent, Body, CombatIntent, Combatant, Enemy, Facing, Health, MotionState, MoveIntent,
    SpawnPoint, WeaponSet, spawn_box, spawn_terrain,
};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct Level {
    pub half_size: f32,
    pub props: Vec<Prop>,
    pub ground: TerrainGeometry,
    pub ramp: TerrainGeometry,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Prop {
    pub center: Vec2,
    pub size: Vec3,
    pub kind: PropKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Crate,
    Pillar,
    Stone,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Boundary {
    pub center: Vec3,
    pub size: Vec3,
}

impl Level {
    fn courtyard() -> Self {
        use PropKind::{Crate, Pillar, Stone};

        let prop = |x, z, size, kind| Prop {
            center: Vec2::new(x, z),
            size,
            kind,
        };

        Self {
            half_size: 25.0,
            ground: TerrainGeometry::ground(25.0),
            ramp: TerrainGeometry::ramp(),
            props: vec![
                prop(-5.0, -5.0, Vec3::new(1.6, 1.4, 1.6), Crate),
                prop(-3.0, -7.0, Vec3::new(1.2, 1.0, 1.2), Crate),
                prop(5.0, -4.0, Vec3::new(1.8, 1.5, 1.6), Crate),
                prop(6.0, -6.2, Vec3::new(1.2, 1.0, 1.2), Crate),
                prop(8.0, 3.0, Vec3::new(1.5, 1.2, 1.5), Crate),
                prop(9.0, 5.0, Vec3::new(1.2, 1.6, 1.2), Crate),
                prop(-8.0, 7.0, Vec3::new(1.7, 1.4, 1.7), Crate),
                prop(-10.0, 6.0, Vec3::new(1.2, 0.9, 1.2), Crate),
                prop(-10.0, -12.0, Vec3::new(1.8, 3.2, 1.8), Pillar),
                prop(10.0, -12.0, Vec3::new(1.8, 3.2, 1.8), Pillar),
                prop(-15.0, 0.0, Vec3::new(2.0, 2.6, 2.0), Pillar),
                prop(15.0, 0.0, Vec3::new(2.0, 2.6, 2.0), Pillar),
                prop(-9.0, 14.0, Vec3::new(1.8, 3.2, 1.8), Pillar),
                prop(9.0, 14.0, Vec3::new(1.8, 3.2, 1.8), Pillar),
                prop(-5.0, 11.0, Vec3::new(2.6, 0.8, 1.8), Stone),
                prop(3.0, 12.0, Vec3::new(2.0, 0.65, 2.4), Stone),
                prop(-18.0, -17.0, Vec3::new(2.8, 1.1, 2.0), Stone),
                prop(17.0, -17.0, Vec3::new(2.0, 0.9, 2.8), Stone),
                prop(17.0, 15.0, Vec3::new(2.6, 0.8, 1.8), Stone),
                prop(-17.0, 16.0, Vec3::new(2.0, 1.1, 2.6), Stone),
                prop(-2.0, -13.0, Vec3::new(5.0, 1.0, 1.2), Stone),
            ],
        }
    }

    pub fn boundaries(&self) -> [Boundary; 4] {
        let edge = self.half_size + 0.25;
        let length = self.half_size * 2.0 + 1.0;
        [
            Boundary {
                center: Vec3::new(-edge, 0.35, 0.0),
                size: Vec3::new(0.5, 0.7, length),
            },
            Boundary {
                center: Vec3::new(edge, 0.35, 0.0),
                size: Vec3::new(0.5, 0.7, length),
            },
            Boundary {
                center: Vec3::new(0.0, 0.35, -edge),
                size: Vec3::new(length, 0.7, 0.5),
            },
            Boundary {
                center: Vec3::new(0.0, 0.35, edge),
                size: Vec3::new(length, 0.7, 0.5),
            },
        ]
    }
}

pub fn setup(world: &mut World) {
    let level = Level::courtyard();
    spawn_terrain(
        world,
        level.ground.vertices.clone(),
        level.ground.triangles.clone(),
    );
    spawn_terrain(
        world,
        level.ramp.vertices.clone(),
        level.ramp.triangles.clone(),
    );
    for prop in &level.props {
        let center = Vec3::new(prop.center.x, prop.size.y * 0.5, prop.center.y);
        let entity = spawn_box(world, center, prop.size, prop.kind == PropKind::Crate);
        world.entity_mut(entity).insert(*prop);
    }
    for boundary in level.boundaries() {
        let entity = spawn_box(world, boundary.center, boundary.size, false);
        world.entity_mut(entity).insert(boundary);
    }
    world.insert_resource(level);
    world.spawn((
        Player,
        Body::new(Vec2::ZERO, 6.0, 0.35),
        MoveIntent::default(),
        ActionIntent::default(),
        MotionState::default(),
        Facing::default(),
        Combatant::new(WeaponSet::HammerSniper),
        CombatIntent::default(),
        Health::new(100.0),
        SpawnPoint(Vec2::ZERO),
    ));
    for position in [
        Vec2::new(-1.5, -4.0),
        Vec2::new(0.0, -5.5),
        Vec2::new(1.5, -4.5),
        Vec2::new(1.0, -8.0),
    ] {
        world.spawn((
            Enemy::default(),
            Body::new(position, 2.0, 0.3),
            MoveIntent::default(),
            ActionIntent::default(),
            MotionState::default(),
            Facing::default(),
            Health::new(120.0),
            SpawnPoint(position),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{PhysicsPose, SimulationPlugin};
    use avian3d::prelude::RigidBody;
    use std::time::Duration;

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn terrain_actor(position: Vec3) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        let level = Level::courtyard();
        spawn_terrain(
            app.world_mut(),
            level.ground.vertices,
            level.ground.triangles,
        );
        spawn_terrain(app.world_mut(), level.ramp.vertices, level.ramp.triangles);
        let actor = app
            .world_mut()
            .spawn((
                Body::at(position, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
            ))
            .id();
        tick(&mut app, 12);
        (app, actor)
    }

    fn position(app: &App, actor: Entity) -> Vec3 {
        app.world().get::<Body>(actor).unwrap().world_position()
    }

    #[test]
    fn terrain_zones_preserve_existing_props_and_open_spawns() {
        let level = Level::courtyard();
        let zones = [
            (
                Vec2::new(-6.0, 1.0),
                Vec2::new(-3.0, terrain::ramp_summit() + 3.0),
            ),
            (Vec2::new(11.0, 4.0), Vec2::new(19.5, 13.0)),
        ];
        assert_eq!(level.props.len(), 21);
        for prop in &level.props {
            assert!(prop.size.is_finite() && prop.size.min_element() > 0.0);
            let half_size = Vec2::new(prop.size.x, prop.size.z) * 0.5;
            let extent = prop.center.abs() + half_size;
            assert!(extent.max_element() < level.half_size);
            assert!((prop.center.abs() - half_size).max_element() > 2.0);
            for (min, max) in zones {
                assert!(
                    (prop.center + half_size).cmple(min).any()
                        || (prop.center - half_size).cmpge(max).any(),
                    "prop {prop:?} overlaps a terrain test zone"
                );
            }
        }
        for point in [
            Vec2::ZERO,
            Vec2::new(-1.5, -4.0),
            Vec2::new(0.0, -5.5),
            Vec2::new(1.5, -4.5),
            Vec2::new(1.0, -8.0),
        ] {
            for (min, max) in zones {
                assert!(point.cmplt(min - 0.35).any() || point.cmpgt(max + 0.35).any());
            }
        }
        for vertex in &level.ground.vertices {
            if vertex.x.abs() <= 2.0 && vertex.z.abs() <= 2.0 {
                assert_eq!(vertex.y, 0.0);
            }
        }
    }

    #[test]
    fn level_assembles_dynamic_crates_static_props_and_finite_rim() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        setup(app.world_mut());
        let world = app.world_mut();
        let mut props = world.query::<(&Prop, &PhysicsPose, &RigidBody)>();
        let props: Vec<_> = props.iter(world).collect();
        assert_eq!(props.len(), 21);
        for (prop, pose, body) in props {
            assert_eq!(*body == RigidBody::Dynamic, prop.kind == PropKind::Crate);
            let center = Vec3::new(prop.center.x, prop.size.y * 0.5, prop.center.y);
            assert!(pose.position().distance(center) < 0.0001);
        }
        let mut boundaries = world.query::<(&Boundary, &PhysicsPose, &RigidBody)>();
        let boundaries: Vec<_> = boundaries.iter(world).collect();
        assert_eq!(boundaries.len(), 4);
        for (boundary, pose, body) in boundaries {
            assert_eq!(*body, RigidBody::Static);
            assert_eq!(boundary.size.y, 0.7);
            assert_eq!(pose.position(), boundary.center);
        }
    }

    #[test]
    fn combat_actors_spawn_with_complete_state_and_settle_on_shared_ground() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        setup(app.world_mut());
        let world = app.world_mut();
        let mut actors = world.query::<(&Body, &Health, &SpawnPoint)>();
        let actors: Vec<_> = actors.iter(world).collect();
        assert_eq!(actors.len(), 5);
        for (body, health, spawn) in actors {
            assert_eq!(body.position(), spawn.0);
            assert!(health.is_alive());
        }
        tick(&mut app, 12);
        let world = app.world_mut();
        let mut actors = world.query::<(&Body, &MotionState)>();
        for (body, motion) in actors.iter(world) {
            assert!(body.world_position().y.abs() < 0.03);
            assert!(motion.is_grounded());
        }
    }

    #[test]
    fn platform_landing_refills_two_jumps_without_snapping_to_world_zero() {
        let (mut app, actor) = terrain_actor(Vec3::new(-4.5, 1.5, 6.5));
        for _ in 0..2 {
            app.world_mut()
                .get_mut::<ActionIntent>(actor)
                .unwrap()
                .request_jump();
            tick(&mut app, 8);
        }
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.jumps_used(), 2);
        assert!(position(&app, actor).y > 2.0);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 120);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_grounded());
        assert_eq!(motion.jumps_used(), 0);
        assert!((position(&app, actor).y - 1.5).abs() < 0.06);
    }
}
