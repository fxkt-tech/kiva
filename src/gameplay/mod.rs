//! The demo's level data and player assembly, independent of rendering and input.

use bevy::prelude::*;

use crate::simulation::{
    ActionIntent, Body, CollisionWorld, CombatIntent, Combatant, Enemy, Facing, Health,
    MotionState, MoveIntent, SolidBox, SpawnPoint, WeaponSet,
};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct Level {
    pub half_size: f32,
    pub props: Vec<Prop>,
}

#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub center: Vec2,
    pub size: Vec3,
    pub kind: PropKind,
}

#[derive(Clone, Copy, Debug)]
pub enum PropKind {
    Crate,
    Pillar,
    Stone,
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

    fn collision_world(&self) -> CollisionWorld {
        let solids = self
            .props
            .iter()
            .map(|prop| SolidBox::new(prop.center, Vec2::new(prop.size.x, prop.size.z) * 0.5))
            .collect();
        CollisionWorld::new(self.half_size, solids)
    }
}

pub fn setup(world: &mut World) {
    let level = Level::courtyard();
    world.insert_resource(level.collision_world());
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

    #[test]
    fn courtyard_has_open_spawn_and_matching_prop_collisions() {
        let level = Level::courtyard();
        let collision_world = level.collision_world();

        assert!(collision_world.is_clear(Vec2::ZERO, 2.0));
        for prop in &level.props {
            assert!(prop.size.is_finite() && prop.size.min_element() > 0.0);
            let extent = prop.center.abs() + Vec2::new(prop.size.x, prop.size.z) * 0.5;
            assert!(extent.max_element() < level.half_size);
            assert!(!collision_world.is_clear(prop.center, 0.35));
        }
    }

    #[test]
    fn combat_actors_spawn_with_complete_state_on_clear_ground() {
        let mut world = World::new();
        setup(&mut world);
        let mut actors = world.query::<(&Body, &MotionState, &Health, &SpawnPoint)>();
        let actors: Vec<_> = actors.iter(&world).collect();
        assert_eq!(actors.len(), 5);
        for (body, motion, health, spawn) in actors {
            assert!(
                world
                    .resource::<CollisionWorld>()
                    .is_clear(body.position(), 0.35)
            );
            assert_eq!(body.position(), spawn.0);
            assert!(health.is_alive());
            assert!(motion.is_grounded());
        }
    }
}
