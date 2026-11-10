//! Player appearance selection owns only render assets, never combat loadouts.
use bevy::prelude::*;

use super::{
    PlayerView,
    avatar::{self, Appearance, AvatarAppearance},
    combat::WeaponVisual,
};

#[derive(Resource, Default)]
pub(super) struct AppearanceSelection(Appearance);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaAppearance"], js_name = takeAppearance)]
    fn take_appearance() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaAppearance"], js_name = render)]
    fn render(appearance: u32);
}

pub(super) fn sample(
    mut selection: ResMut<AppearanceSelection>,
    #[cfg(not(target_arch = "wasm32"))] keys: Res<ButtonInput<KeyCode>>,
    #[cfg(not(target_arch = "wasm32"))] window: Single<&Window>,
    #[cfg(not(target_arch = "wasm32"))] active: Res<crate::simulation::SimulationActive>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match take_appearance() {
            1 => selection.0 = Appearance::Choulingke,
            2 => selection.0 = Appearance::Lihuowang,
            _ => {}
        }
        render(match selection.0 {
            Appearance::Choulingke => 1,
            Appearance::Lihuowang => 2,
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    if window.focused && active.0 && keys.just_pressed(KeyCode::F3) {
        selection.0 = match selection.0 {
            Appearance::Choulingke => Appearance::Lihuowang,
            Appearance::Lihuowang => Appearance::Choulingke,
        };
    }
}

/// Replace render-owned children in one update, retaining actor and RigPose.
/// The selection may arrive while paused or before any player view exists.
pub(super) fn reconcile(world: &mut World) {
    let Some(selection) = world.get_resource::<AppearanceSelection>() else {
        return;
    };
    let selected = selection.0;
    let players: Vec<_> = world
        .query_filtered::<(Entity, Option<&AvatarAppearance>), With<PlayerView>>()
        .iter(world)
        .filter_map(|(entity, current)| {
            current
                .is_none_or(|current| current.0 != selected)
                .then_some(entity)
        })
        .collect();
    for player in players {
        avatar::replace(world, player, selected);
        let weapons: Vec<_> = world
            .query::<(Entity, &WeaponVisual, &ChildOf)>()
            .iter(world)
            .filter_map(|(entity, weapon, parent)| {
                (parent.parent() == player && weapon.is_melee()).then_some(entity)
            })
            .collect();
        for weapon in weapons {
            avatar::replace_blade(world, weapon, selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            ActorView,
            animation::{RigPose, SourcePose},
            combat::{self, EnemyView},
            skeleton::AnimationLibrary,
            source_animation::SourceAnimationLibrary,
            stickman,
        },
        gameplay::Player,
        simulation::{Body, Combatant, Enemy, Health, MotionState, SimulationActive, WeaponSet},
    };

    fn scene() -> (World, Entity, Entity) {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        world.init_resource::<AppearanceSelection>();
        world.insert_resource(SimulationActive(false));
        let mut motion = MotionState::default();
        assert!(motion.launch_up(4.0));
        let actor = world
            .spawn((
                Player,
                Body::at(Vec3::new(2.0, 1.4, -3.0), 6.0, 0.35),
                motion,
                Combatant::new(WeaponSet::AxeCannon),
                Health::new(100.0),
            ))
            .id();
        let view = world
            .spawn((
                PlayerView,
                ActorView { owner: actor },
                Transform::from_xyz(2.0, 1.4, -3.0),
                Visibility::default(),
            ))
            .id();
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        stickman::spawn(&mut world, view, &material, &material);
        world.spawn((
            Enemy::target_dummy(),
            Body::new(Vec2::new(0.0, -4.0), 6.0, 0.35),
            Health::new(1_000_000.0),
        ));
        combat::setup(&mut world);
        (world, actor, view)
    }

    fn weapon_meshes(world: &mut World, melee: bool) -> Vec<Handle<Mesh>> {
        let roots: Vec<_> = world
            .query::<(Entity, &WeaponVisual)>()
            .iter(world)
            .filter_map(|(entity, weapon)| (weapon.is_melee() == melee).then_some(entity))
            .collect();
        roots
            .iter()
            .flat_map(|root| world.get::<Children>(*root).unwrap().iter())
            .filter_map(|child| world.get::<Mesh3d>(child).map(|mesh| mesh.0.clone()))
            .collect()
    }

    #[test]
    fn paired_appearance_switch_preserves_paused_airborne_actor_pose_and_enemy() {
        let (mut world, actor, view) = scene();
        let original_transform = *world.get::<Transform>(view).unwrap();
        let source_model = world
            .resource::<SourceAnimationLibrary>()
            .rest_model
            .clone();
        {
            let mut rig = world.get_mut::<RigPose>(view).unwrap();
            rig.model[0].translation = Vec3::new(0.2, 0.1, 0.0);
            rig.source = Some(SourcePose {
                model: source_model,
                weight: 0.6,
            });
        }
        let pose = world.get::<RigPose>(view).unwrap().model.clone();
        let source_pointer = world
            .get::<RigPose>(view)
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .model
            .as_ptr();
        let old_melee = weapon_meshes(&mut world, true);
        let ranged = weapon_meshes(&mut world, false);
        let enemy = world
            .query_filtered::<Entity, With<EnemyView>>()
            .single(&world)
            .unwrap();
        let enemy_children: Vec<_> = world.get::<Children>(enemy).unwrap().iter().collect();
        world.resource_mut::<AppearanceSelection>().0 = Appearance::Lihuowang;
        reconcile(&mut world);
        assert_eq!(
            world.get::<AvatarAppearance>(view).unwrap().0,
            Appearance::Lihuowang
        );
        assert_eq!(*world.get::<Transform>(view).unwrap(), original_transform);
        assert_eq!(
            world.get::<Body>(actor).unwrap().world_position(),
            Vec3::new(2.0, 1.4, -3.0)
        );
        assert_eq!(
            world.get::<MotionState>(actor).unwrap().vertical_velocity(),
            4.0
        );
        assert_eq!(
            world.get::<Combatant>(actor).unwrap().weapon_set(),
            WeaponSet::AxeCannon
        );
        assert_eq!(world.get::<Health>(actor).unwrap().current(), 100.0);
        let rig = world.get::<RigPose>(view).unwrap();
        assert_eq!(rig.model, pose);
        assert_eq!(rig.source.as_ref().unwrap().model.as_ptr(), source_pointer);
        assert_eq!(rig.source.as_ref().unwrap().weight, 0.6);
        assert_ne!(weapon_meshes(&mut world, true), old_melee);
        assert_eq!(weapon_meshes(&mut world, false), ranged);
        assert_eq!(
            world.get::<AvatarAppearance>(enemy).unwrap().0,
            Appearance::Choulingke
        );
        assert_eq!(
            world
                .get::<Children>(enemy)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            enemy_children
        );
        let counts = (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world.resource::<Assets<Image>>().len(),
            world.entities().len(),
        );
        for _ in 0..4 {
            world.resource_mut::<AppearanceSelection>().0 = Appearance::Choulingke;
            reconcile(&mut world);
            assert_eq!(weapon_meshes(&mut world, true), old_melee);
            world.resource_mut::<AppearanceSelection>().0 = Appearance::Lihuowang;
            reconcile(&mut world);
            assert_eq!(
                (
                    world.resource::<Assets<Mesh>>().len(),
                    world.resource::<Assets<StandardMaterial>>().len(),
                    world.resource::<Assets<Image>>().len(),
                    world.entities().len(),
                ),
                counts
            );
        }
    }

    #[test]
    fn selection_before_views_exist_is_applied_when_player_is_spawned() {
        let mut world = World::new();
        world.insert_resource(AppearanceSelection(Appearance::Lihuowang));
        reconcile(&mut world);
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        let player = world
            .spawn((PlayerView, Transform::default(), Visibility::default()))
            .id();
        avatar::spawn(&mut world, player);
        assert_eq!(
            world.get::<AvatarAppearance>(player).unwrap().0,
            Appearance::Choulingke
        );
        reconcile(&mut world);
        assert_eq!(
            world.get::<AvatarAppearance>(player).unwrap().0,
            Appearance::Lihuowang
        );
    }
}
