//! The demo's level data and player assembly, independent of rendering and input.

pub mod terrain;

use bevy::prelude::*;

pub use terrain::TerrainGeometry;

use crate::simulation::{
    ActionIntent, Body, Boss, CombatIntent, Combatant, Enemy, Facing, Health, MotionState,
    MoveIntent, SpawnPoint, WeaponPair, spawn_box, spawn_terrain,
};

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMode {
    #[default]
    Target,
    Combat,
}

impl EnemyMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Target => Self::Combat,
            Self::Combat => Self::Target,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Target => "靶子",
            Self::Combat => "实战",
        }
    }
}

/// Replace only the enemy roster. This also works while the simulation is paused.
pub fn set_enemy_mode(world: &mut World, mode: EnemyMode) {
    if world.get_resource::<EnemyMode>() == Some(&mode) {
        return;
    }
    let enemies: Vec<_> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    for enemy in enemies {
        world.despawn(enemy);
    }
    for (mut combat, mut intent) in world
        .query::<(&mut Combatant, &mut CombatIntent)>()
        .iter_mut(world)
    {
        combat.clear_targets();
        intent.clear();
    }
    for mut state in world
        .query::<&mut crate::simulation::CompanionState>()
        .iter_mut(world)
    {
        state.visible = false;
    }
    for mut defense in world
        .query::<&mut crate::simulation::DefenseState>()
        .iter_mut(world)
    {
        defense.clear();
    }
    world.insert_resource(mode);
    spawn_enemies(world, mode);
}

fn spawn_enemies(world: &mut World, mode: EnemyMode) {
    let positions: &[Vec2] = match mode {
        EnemyMode::Target => &[Vec2::new(0.0, -4.0)],
        EnemyMode::Combat => &[Vec2::new(0.0, -6.0)],
    };
    for &position in positions {
        let (enemy, health) = match mode {
            EnemyMode::Target => (Enemy::target_dummy(), 1_000_000.0),
            EnemyMode::Combat => (Enemy::default(), Boss::PHASE_HEALTH),
        };
        let mut facing = Facing::default();
        facing.set(-position);
        let mut entity = world.spawn((
            enemy,
            Body::new(
                position,
                if mode == EnemyMode::Combat { 5.2 } else { 2.0 },
                0.3,
            )
            .scaled(if mode == EnemyMode::Target { 1.45 } else { 1.0 }),
            MoveIntent::default(),
            ActionIntent::default(),
            MotionState::default(),
            facing,
            Health::new(health),
            SpawnPoint(position),
        ));
        if mode == EnemyMode::Combat {
            entity.insert(Boss::default());
        }
    }
}

#[derive(Resource)]
pub struct Level {
    pub half_size: f32,
    pub ground: TerrainGeometry,
    pub bamboo: Vec<Bamboo>,
    pub forest_trunks: TerrainGeometry,
    pub forest_rocks: TerrainGeometry,
}

/// The same leaning trunk geometry is used for contact and presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bamboo {
    pub base: Vec3,
    pub height: f32,
    pub radius: f32,
    pub lean: Vec2,
}

impl Bamboo {
    pub fn center_at(&self, fraction: f32) -> Vec3 {
        self.base + Vec3::new(self.lean.x, self.height, self.lean.y) * fraction
    }

    pub fn radius_at(&self, fraction: f32) -> f32 {
        self.radius * (1.0 - 0.35 * fraction)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Boundary {
    pub center: Vec3,
    pub size: Vec3,
}

impl Level {
    fn bamboo_forest() -> Self {
        let mut bamboo = Vec::new();
        for z in -20_i32..=20 {
            for x in -20_i32..=20 {
                let key = ((x + 20) * 41 + z + 20) as u32;
                let point = Vec2::new(x as f32, z as f32) * 1.18
                    + Vec2::new(variation(key, 1) - 0.5, variation(key, 2) - 0.5) * 0.62;
                if forest_clearance(point) || variation(key, 3) < 0.12 {
                    continue;
                }
                bamboo.push(Bamboo {
                    base: Vec3::new(point.x, terrain::hill_height(point), point.y),
                    height: 7.0 + variation(key, 4) * 5.0,
                    radius: 0.065 + variation(key, 5) * 0.07,
                    lean: Vec2::new(variation(key, 6) - 0.5, variation(key, 7) - 0.5) * 0.9,
                });
            }
        }
        let forest_trunks = TerrainGeometry::bamboo_trunks(&bamboo);
        Self {
            half_size: 25.0,
            ground: TerrainGeometry::ground(25.0),
            bamboo,
            forest_trunks,
            forest_rocks: TerrainGeometry::forest_rocks(),
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

/// Visual variation is deterministic across native/WASM and independent of time.
pub fn variation(index: u32, salt: u32) -> f32 {
    let mut value = index.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    (value & 0xffff) as f32 / 65535.0
}

pub fn path_distance(point: Vec2) -> f32 {
    let north_south = (point.x - 3.0 * (point.y * 0.14).sin()).abs();
    let east_west = (point.y - 2.7 * (point.x * 0.11).sin()).abs();
    north_south.min(east_west)
}

fn forest_clearance(point: Vec2) -> bool {
    let clearing = (point.x / 8.5).powi(2) + ((point.y + 4.0) / 9.5).powi(2) < 1.0;
    let hill = (9.2..=21.5).contains(&point.x) && (2.0..=15.0).contains(&point.y);
    clearing || hill || path_distance(point) < 1.9
}

pub fn setup(world: &mut World) {
    let level = Level::bamboo_forest();
    spawn_terrain(
        world,
        level.ground.vertices.clone(),
        level.ground.triangles.clone(),
    );
    spawn_terrain(
        world,
        level.forest_trunks.vertices.clone(),
        level.forest_trunks.triangles.clone(),
    );
    spawn_terrain(
        world,
        level.forest_rocks.vertices.clone(),
        level.forest_rocks.triangles.clone(),
    );
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
        Combatant::new(WeaponPair::HammerMusket),
        CombatIntent::default(),
        Health::new(1000.0),
        SpawnPoint(Vec2::ZERO),
    ));
    let mode = world
        .get_resource::<EnemyMode>()
        .copied()
        .unwrap_or_default();
    world.insert_resource(mode);
    spawn_enemies(world, mode);
    crate::simulation::combat_setup_companions(world);
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

    #[test]
    fn dense_forest_is_deterministic_and_keeps_clear_traversal_corridors() {
        let first = Level::bamboo_forest();
        let second = Level::bamboo_forest();
        assert_eq!(first.bamboo, second.bamboo);
        assert!(
            first.bamboo.len() > 600,
            "forest is too sparse: {}",
            first.bamboo.len()
        );
        for stem in &first.bamboo {
            assert!(stem.base.is_finite() && stem.lean.is_finite());
            assert!((7.0..=12.0).contains(&stem.height));
            assert!((0.065..=0.135).contains(&stem.radius));
            assert!(stem.base.xz().length() > 2.5);
            assert!(stem.base.xz().distance(Vec2::new(0.0, -4.0)) > 8.0);
        }
        for z in (-23..=23).step_by(2) {
            let z = z as f32;
            let point = Vec2::new(3.0 * (z * 0.14).sin(), z);
            assert!(
                first
                    .bamboo
                    .iter()
                    .all(|stem| stem.base.xz().distance(point) - stem.radius > 1.1)
            );
        }
        assert_eq!(first.forest_trunks.vertices.len(), first.bamboo.len() * 42);
        for geometry in [&first.forest_trunks, &first.forest_rocks] {
            assert!(!geometry.triangles.is_empty());
            for triangle in &geometry.triangles {
                let [a, b, c] = triangle.map(|index| geometry.vertices[index as usize]);
                assert!(a.is_finite() && b.is_finite() && c.is_finite());
                assert!((b - a).cross(c - a).length_squared() > 1e-12);
            }
        }
    }

    #[test]
    fn woodland_terrain_keeps_spawns_and_the_hill_route_clear_of_bamboo() {
        let level = Level::bamboo_forest();
        // The walkable approach, summit and steep flank of the hill.
        let (min, max) = (Vec2::new(11.0, 4.0), Vec2::new(19.5, 13.0));
        for stem in &level.bamboo {
            let center = stem.base.xz();
            let half_size = Vec2::splat(stem.radius + 0.35);
            assert!((center.abs() + half_size).max_element() < level.half_size);
            assert_eq!(stem.base.y, terrain::hill_height(center));
            assert!(
                (center + half_size).cmple(min).any() || (center - half_size).cmpge(max).any(),
                "bamboo {stem:?} obstructs the hill route"
            );
        }
        // The player spawn, both enemy spawns and the fighting lane between them.
        for point in [
            Vec2::ZERO,
            Vec2::new(-1.5, -4.0),
            Vec2::new(0.0, -4.0),
            Vec2::new(0.0, -6.0),
            Vec2::new(1.5, -4.5),
            Vec2::new(1.0, -8.0),
        ] {
            for stem in &level.bamboo {
                assert!(
                    stem.base.xz().distance(point) - stem.radius > 2.0,
                    "bamboo {stem:?} crowds {point}"
                );
            }
        }
        for vertex in &level.ground.vertices {
            if vertex.x.abs() <= 2.0 && vertex.z.abs() <= 2.0 {
                assert_eq!(vertex.y, 0.0);
            }
        }
    }

    #[test]
    fn woodland_assembles_shared_trunk_and_rock_geometry_with_a_finite_rim() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        setup(app.world_mut());
        let world = app.world_mut();
        let mut terrain = world.query_filtered::<(&PhysicsPose, &RigidBody), Without<Boundary>>();
        let terrain: Vec<_> = terrain.iter(world).collect();
        assert_eq!(terrain.len(), 3, "ground, bamboo and rocks are solid");
        for (pose, body) in terrain {
            assert_eq!(*body, RigidBody::Static);
            assert_eq!(pose.position(), Vec3::ZERO);
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
        assert_eq!(actors.len(), 2);
        for (body, health, spawn) in actors {
            assert_eq!(body.position(), spawn.0);
            assert!(health.is_alive());
        }
        assert_eq!(*world.resource::<EnemyMode>(), EnemyMode::Target);
        let (enemy, health, body) = world
            .query::<(&Enemy, &Health, &Body)>()
            .single(world)
            .unwrap();
        assert!(enemy.is_passive());
        assert_eq!(health.maximum(), 1_000_000.0);
        assert_eq!(body.scale(), 1.45);
        assert!((body.radius() - 0.435).abs() < 0.001);
        tick(&mut app, 12);
        let world = app.world_mut();
        let mut actors = world.query::<(&Body, &MotionState)>();
        for (body, motion) in actors.iter(world) {
            assert!(body.world_position().y.abs() < 0.03);
            assert!(motion.is_grounded());
        }
    }

    #[test]
    fn modes_replace_the_roster_while_paused_and_the_dummy_never_attacks_or_chases() {
        use crate::simulation::{CombatEffect, CombatFeedback, SimulationActive, teleport_actor};

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        app.insert_resource(EnemyMode::Combat);
        setup(app.world_mut());
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        let old_enemies: Vec<_> = world
            .query_filtered::<Entity, With<Enemy>>()
            .iter(world)
            .collect();
        world.resource_mut::<SimulationActive>().0 = false;
        set_enemy_mode(world, EnemyMode::Target);
        assert!(
            old_enemies
                .iter()
                .all(|&entity| world.get_entity(entity).is_err())
        );
        let dummy = world
            .query_filtered::<Entity, With<Enemy>>()
            .single(world)
            .unwrap();
        assert_eq!(world.get::<Health>(dummy).unwrap().maximum(), 1_000_000.0);
        assert!(world.get::<Boss>(dummy).is_none());
        let origin = world.get::<Body>(dummy).unwrap().position();
        world.resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 180);
        assert!(
            app.world()
                .get::<Body>(dummy)
                .unwrap()
                .position()
                .distance(origin)
                < 0.02
        );

        // Stand in attack range for several full attack cycles.
        teleport_actor(app.world_mut(), player, Vec3::new(0.0, 0.0, -2.6));
        tick(&mut app, 300);
        assert_eq!(app.world().get::<Health>(player).unwrap().current(), 1000.0);
        assert!(
            !app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| {
                    event.actor == dummy
                        && matches!(event.kind, CombatEffect::EnemyWindup | CombatEffect::Melee)
                })
        );
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 13);
        assert!(app.world().get::<Health>(dummy).unwrap().current() < 1_000_000.0);
        assert!(!app.world().get::<MotionState>(dummy).unwrap().is_grounded());
        let cooldown = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .melee_cooldown();
        assert!(cooldown > 0.0);

        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        set_enemy_mode(app.world_mut(), EnemyMode::Combat);
        let world = app.world_mut();
        assert!(world.get_entity(dummy).is_err());
        let actor = world.get::<Combatant>(player).unwrap();
        assert_eq!(actor.target(), None);
        assert_eq!(actor.combo_remaining(), 0.0);
        assert_eq!(actor.melee_cooldown(), cooldown);
        let mut enemies = world.query::<(&Enemy, &Health, &Body)>();
        assert_eq!(enemies.iter(world).count(), 1);
        for (enemy, health, body) in enemies.iter(world) {
            assert!(!enemy.is_passive());
            assert_eq!(health.maximum(), 650.0);
            assert_eq!(body.scale(), 1.0);
        }
        assert_eq!(world.query::<&Boss>().iter(world).count(), 1);
    }
}
