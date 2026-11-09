//! Authored skeletal animation driven by read-only simulation snapshots and actor-tagged events.
mod poses;

use bevy::prelude::*;

use super::{
    ActorView,
    skeleton::{AnimationLibrary, blend},
};
use crate::simulation::{
    Body, CombatEffect, CombatFeedback, Combatant, Enemy, Facing, Health, MotionState,
    SimulationActive, WeaponMode, WeaponSet,
};
use poses::Gesture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PoseState {
    Idle,
    Walk,
    Run,
    Air(u8),
    Land,
    Dodge,
    Crouch,
    Slide,
    Block,
    Attack(Gesture),
    Spin,
    Down,
}

struct Playback {
    kind: Gesture,
    elapsed: f32,
}

#[derive(Component)]
pub(super) struct RigPose {
    local: Vec<Transform>,
    pub(super) model: Vec<Transform>,
    from: Vec<Transform>,
    state: PoseState,
    clock: f32,
    gait_time: f32,
    transition: f32,
    air_time: f32,
    was_airborne: bool,
    jumps_used: u8,
    crouch_weight: f32,
    event_sequence: u64,
    playback: Option<Playback>,
    restart: bool,
}

impl RigPose {
    pub fn new(library: &AnimationLibrary) -> Self {
        let local = poses::idle(library, 0.0, WeaponMode::Melee);
        Self {
            model: library.model_pose(&local),
            from: local.clone(),
            local,
            state: PoseState::Idle,
            clock: 0.0,
            gait_time: 0.0,
            transition: 1.0,
            air_time: 0.0,
            was_airborne: false,
            jumps_used: 0,
            crouch_weight: 0.0,
            event_sequence: 0,
            playback: None,
            restart: false,
        }
    }

    fn observe(
        &mut self,
        library: &AnimationLibrary,
        effect: CombatEffect,
        set: WeaponSet,
        leaping: bool,
    ) {
        let kind = match effect {
            CombatEffect::Melee => {
                if set == WeaponSet::AxeCannon {
                    Gesture::Axe
                } else {
                    Gesture::Slash
                }
            }
            CombatEffect::Shot => Gesture::Shot,
            CombatEffect::Hammer => Gesture::Hammer,
            CombatEffect::Cannon if leaping => Gesture::Cannon,
            CombatEffect::Cannon => Gesture::Shot,
            CombatEffect::Hit => Gesture::Hit,
            CombatEffect::Block => Gesture::Guard,
            CombatEffect::EnemyWindup => Gesture::Windup,
            CombatEffect::Respawn => {
                *self = Self::new(library);
                return;
            }
            CombatEffect::Defeat | CombatEffect::Spin | CombatEffect::Dodge => return,
        };
        self.playback = Some(Playback { kind, elapsed: 0.0 });
        self.restart = true;
    }

    fn advance(&mut self, library: &AnimationLibrary, input: Inputs, dt: f32) {
        let landed = self.was_airborne && !input.airborne;
        self.was_airborne = input.airborne;
        let jumped = input.jumps_used > self.jumps_used;
        self.jumps_used = input.jumps_used;
        self.air_time = if input.airborne {
            if jumped { dt } else { self.air_time + dt }
        } else {
            0.0
        };
        if let Some(playback) = &mut self.playback {
            playback.elapsed += dt;
            if playback.elapsed >= playback.kind.duration() {
                self.playback = None;
            }
        }
        let state = if !input.alive {
            PoseState::Down
        } else if self
            .playback
            .as_ref()
            .is_some_and(|playback| playback.kind == Gesture::Hit)
        {
            PoseState::Attack(Gesture::Hit)
        } else if input.dodging {
            PoseState::Dodge
        } else if input.spinning {
            PoseState::Spin
        } else if input.sliding {
            PoseState::Slide
        } else if input.blocking {
            PoseState::Block
        } else if let Some(playback) = &self.playback {
            PoseState::Attack(playback.kind)
        } else if input.airborne {
            PoseState::Air(input.jumps_used)
        } else if landed || (self.state == PoseState::Land && self.clock < 0.16) {
            PoseState::Land
        } else if input.crouched {
            PoseState::Crouch
        } else if input.speed > 0.01 {
            if input.sprinting {
                PoseState::Run
            } else {
                PoseState::Walk
            }
        } else {
            PoseState::Idle
        };
        let restart = std::mem::take(&mut self.restart);
        if state != self.state || restart {
            self.from = self.local.clone();
            self.state = state;
            self.clock = 0.0;
            self.gait_time = 0.0;
            self.transition = 0.0;
        }
        self.clock += dt;
        self.transition += dt;
        self.gait_time += dt
            * match state {
                PoseState::Run => 1.6 * input.speed / 6.0,
                PoseState::Walk => 1.2 * input.speed / 3.8,
                PoseState::Crouch => 1.2 * input.speed / 3.8,
                PoseState::Block => 0.85 * input.speed / 3.8,
                _ => 0.0,
            };
        let mut target = match state {
            PoseState::Idle => poses::idle(library, self.clock, input.mode),
            PoseState::Walk | PoseState::Run => library.sample(
                if state == PoseState::Run {
                    "Jog_Fwd_Loop"
                } else {
                    "Walk_Loop"
                },
                self.gait_time,
                true,
            ),
            PoseState::Air(jumps) => airborne_pose(library, self.air_time, jumps),
            PoseState::Land => library.sample("Jump_Land", 0.15 + self.clock * 4.0, false),
            PoseState::Dodge => {
                let mut pose = library.sample(
                    "Roll",
                    input.dodge_elapsed / 0.42 * library.duration("Roll"),
                    false,
                );
                pose[0].rotation = Quat::from_rotation_y(input.dodge_yaw) * pose[0].rotation;
                pose
            }
            PoseState::Crouch => {
                if input.speed > 0.01 {
                    let mut pose = library.sample("Walk_Loop", self.gait_time, true);
                    poses::hold_guard(library, &mut pose, input.mode);
                    pose
                } else {
                    poses::idle(library, self.clock, input.mode)
                }
            }
            PoseState::Slide => {
                let mut pose = library.sample("Sword_Idle", 0.0, false);
                poses::hold_guard(library, &mut pose, input.mode);
                pose[0].rotation = Quat::from_rotation_y(input.slide_yaw) * pose[0].rotation;
                pose
            }
            PoseState::Block => library.sample("Sword_Block", 0.2 + self.clock.min(0.2), false),
            PoseState::Attack(kind) => poses::gesture(
                library,
                kind,
                self.playback
                    .as_ref()
                    .map_or(0.0, |playback| playback.elapsed),
                input.mode,
            ),
            PoseState::Spin => poses::gesture(
                library,
                Gesture::Axe,
                (self.clock % Gesture::Axe.duration()).min(0.28),
                WeaponMode::Melee,
            ),
            PoseState::Down => library.sample(
                "Death01",
                self.clock / 0.8 * library.duration("Death01"),
                false,
            ),
        };
        if matches!(state, PoseState::Air(_)) {
            poses::hold_guard(library, &mut target, input.mode);
        }
        if input.airborne
            && !matches!(
                state,
                PoseState::Air(_) | PoseState::Dodge | PoseState::Down
            )
        {
            replace_lower_body(
                library,
                &mut target,
                &airborne_pose(library, self.air_time, input.jumps_used),
            );
        } else if state == PoseState::Block && input.speed > 0.01 {
            replace_lower_body(
                library,
                &mut target,
                &library.sample("Walk_Loop", self.gait_time, true),
            );
        }
        let seconds = match state {
            PoseState::Attack(Gesture::Hit) => 0.035,
            PoseState::Attack(_) => 0.055,
            _ => 0.12,
        };
        let weight = poses::smooth(self.transition / seconds);
        let mut posed: Vec<_> = self
            .from
            .iter()
            .copied()
            .zip(target)
            .map(|(a, b)| blend(a, b, weight))
            .collect();
        if input.airborne && state != PoseState::Down {
            // Physics supplies the world jump height. Authored pelvis motion may
            // bend the body, but must not lift the whole character a second time.
            pin_airborne_pelvis(library, &mut posed);
        }
        if state == PoseState::Spin {
            posed[0].rotation =
                Quat::from_rotation_y(self.clock * std::f32::consts::TAU * 2.0) * posed[0].rotation;
        }
        // Keep the transition source before planted-foot IK; storing the lowered
        // pose here would apply the pelvis offset again on the next transition.
        self.local = posed.clone();
        self.crouch_weight = (self.crouch_weight
            + if input.crouched || input.sliding {
                dt / 0.12
            } else {
                -dt / 0.12
            })
        .clamp(0.0, 1.0);
        if self.crouch_weight > 0.0 {
            poses::crouch_pose(library, &mut posed, poses::smooth(self.crouch_weight));
        }
        self.model = library.model_pose(&posed);
    }
}

#[derive(Clone, Copy)]
struct Inputs {
    alive: bool,
    airborne: bool,
    jumps_used: u8,
    speed: f32,
    sprinting: bool,
    dodging: bool,
    dodge_elapsed: f32,
    dodge_yaw: f32,
    crouched: bool,
    sliding: bool,
    slide_yaw: f32,
    blocking: bool,
    spinning: bool,
    mode: WeaponMode,
}

fn airborne_pose(library: &AnimationLibrary, clock: f32, jumps: u8) -> Vec<Transform> {
    if jumps >= 2 {
        return library.sample("NinjaJump_Idle_Loop", clock, true);
    }
    let start = library.sample("Jump_Start", 0.30 + clock.min(0.14) * 3.5, false);
    let air = library.sample("Jump_Loop", (clock - 0.14).max(0.0), true);
    let weight = ((clock - 0.14) / 0.10).clamp(0.0, 1.0);
    start
        .into_iter()
        .zip(air)
        .map(|(a, b)| blend(a, b, weight))
        .collect()
}

fn replace_lower_body(library: &AnimationLibrary, local: &mut [Transform], lower: &[Transform]) {
    let pelvis = library.bone("pelvis");
    local[pelvis] = lower[pelvis];
    let first_leg = library.bone("thigh_l");
    local[first_leg..].copy_from_slice(&lower[first_leg..]);
}

fn pin_airborne_pelvis(library: &AnimationLibrary, local: &mut [Transform]) {
    let pelvis = library.bone("pelvis");
    let parent = library.bones[pelvis].parent.expect("pelvis has a parent");
    let model = library.model_pose(local);
    let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
    let offset = Vec3::Y * (standing[pelvis].translation.y - model[pelvis].translation.y);
    local[pelvis].translation += model[parent]
        .compute_affine()
        .inverse()
        .transform_vector3(offset);
}

type AnimatedActors = (
    &'static Body,
    &'static MotionState,
    &'static Health,
    &'static Facing,
    Option<&'static Combatant>,
    Option<&'static Enemy>,
);

pub(super) fn animate(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    library: Res<AnimationLibrary>,
    feedback: Res<CombatFeedback>,
    actors: Query<AnimatedActors>,
    mut views: Query<(&ActorView, &mut RigPose)>,
) {
    if !active.0 {
        return;
    }
    let latest_sequence = feedback.events().last().map_or(0, |event| event.sequence);
    for (view, mut rig) in &mut views {
        let Ok((body, motion, health, facing, combat, _)) = actors.get(view.owner) else {
            continue;
        };
        let set = combat.map_or(WeaponSet::HammerSniper, Combatant::weapon_set);
        let mode = combat.map_or(WeaponMode::Melee, Combatant::weapon_mode);
        let previous_sequence = rig.event_sequence;
        for event in feedback
            .events()
            .filter(|event| event.sequence > previous_sequence && event.actor == view.owner)
        {
            rig.observe(&library, event.kind, set, motion.is_leaping());
        }
        rig.event_sequence = latest_sequence;
        let direction = motion.dodge_direction();
        let dodge_yaw = (-direction.x).atan2(-direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let slide_direction = motion.slide_direction();
        let slide_yaw = (-slide_direction.x).atan2(-slide_direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let speed = body.step_displacement().length() / fixed.delta_secs().max(0.0001);
        rig.advance(
            &library,
            Inputs {
                alive: health.is_alive(),
                airborne: !motion.is_grounded(),
                jumps_used: motion.jumps_used(),
                speed,
                sprinting: motion.is_sprinting(),
                dodging: motion.is_dodging(),
                dodge_elapsed: motion.dodge_elapsed(),
                dodge_yaw,
                crouched: motion.is_crouched(),
                sliding: motion.is_sliding(),
                slide_yaw,
                blocking: motion.is_blocking(),
                spinning: combat.is_some_and(Combatant::is_spinning),
                mode,
            },
            time.delta_secs(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{
        ActionIntent, CollisionWorld, CombatIntent, MoveIntent, SimulationPlugin, SpawnPoint,
    };
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(100.0, vec![]))
            .init_resource::<AnimationLibrary>()
            .add_systems(Update, animate);
        app.finish();
        app
    }

    fn actor(app: &mut App) -> Entity {
        let rig = RigPose::new(app.world().resource::<AnimationLibrary>());
        let actor = app
            .world_mut()
            .spawn((
                Body::new(Vec2::ZERO, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
                Health::new(100.0),
                Combatant::new(WeaponSet::HammerSniper),
                CombatIntent::default(),
                SpawnPoint(Vec2::ZERO),
            ))
            .id();
        let view = app
            .world_mut()
            .spawn((ActorView { owner: actor }, rig))
            .id();
        app.world_mut().entity_mut(actor).insert(TestView(view));
        actor
    }

    #[derive(Component)]
    struct TestView(Entity);

    fn actor_rig(app: &App, actor: Entity) -> &RigPose {
        let view = app.world().get::<TestView>(actor).unwrap().0;
        app.world().get::<RigPose>(view).unwrap()
    }

    fn frame(app: &mut App, fixed: bool) {
        let dt = Duration::from_secs_f64(1.0 / 60.0);
        if fixed {
            app.world_mut().resource_mut::<Time<Fixed>>().advance_by(dt);
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().resource_mut::<Time>().advance_by(dt);
        app.world_mut().run_schedule(Update);
    }

    fn pose_difference(a: &[Transform], b: &[Transform]) -> f32 {
        a.iter()
            .zip(b)
            .map(|(a, b)| a.translation.distance(b.translation))
            .fold(0.0, f32::max)
    }

    fn finite(pose: &[Transform]) {
        for bone in pose {
            assert!(bone.translation.is_finite());
            assert!(bone.rotation.is_finite());
            assert!(bone.scale.is_finite());
            assert!((bone.rotation.length() - 1.0).abs() < 0.001);
        }
    }

    fn idle_input() -> Inputs {
        Inputs {
            alive: true,
            airborne: false,
            jumps_used: 0,
            speed: 0.0,
            sprinting: false,
            dodging: false,
            dodge_elapsed: 0.0,
            dodge_yaw: 0.0,
            crouched: false,
            sliding: false,
            slide_yaw: 0.0,
            blocking: false,
            spinning: false,
            mode: WeaponMode::Melee,
        }
    }

    #[test]
    fn second_jump_uses_act_pose_and_restarts_air_time_without_adding_root_height() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        for _ in 0..25 {
            frame(&mut app, true);
        }
        let first = actor_rig(&app, actor).model.clone();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.jumps_used(), 2);
        assert!(motion.height() > height);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Air(2));
        assert!(rig.air_time < 0.02);
        assert!(rig.clock < 0.02);
        for _ in 0..10 {
            frame(&mut app, true);
        }
        let library = app.world().resource::<AnimationLibrary>();
        let pelvis = library.bone("pelvis");
        let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
        let rig = actor_rig(&app, actor);
        assert!(pose_difference(&first, &rig.model) > 0.05);
        assert!((rig.model[pelvis].translation.y - standing[pelvis].translation.y).abs() < 0.001);
        assert_eq!(
            app.world().get::<Body>(actor).unwrap().position(),
            Vec2::ZERO
        );
        finite(&rig.model);
        let clock = rig.clock;
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Air(2));
        assert!(rig.clock > clock);
        for _ in 0..90 {
            frame(&mut app, true);
            if app.world().get::<MotionState>(actor).unwrap().is_grounded() {
                break;
            }
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Land);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().jumps_used(),
            0
        );
    }

    #[test]
    fn real_fixed_motion_drives_walk_run_jump_land_block_and_roll_poses() {
        let mut app = app();
        let actor = actor(&mut app);
        let idle = actor_rig(&app, actor).model.clone();
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Walk);
        assert!(pose_difference(&idle, &rig.model) > 0.05);
        let walking = rig.model.clone();
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_held(true, false);
        for _ in 0..18 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Run);
        assert!(pose_difference(&walking, &rig.model) > 0.05);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        assert_eq!(actor_rig(&app, actor).state, PoseState::Air(1));
        assert!(app.world().get::<MotionState>(actor).unwrap().height() > 0.0);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        for _ in 0..12 {
            frame(&mut app, false);
        }
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().height(),
            height
        );
        let library = app.world().resource::<AnimationLibrary>();
        let pelvis = library.bone("pelvis");
        let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
        let rig = actor_rig(&app, actor);
        assert!((rig.model[pelvis].translation.y - standing[pelvis].translation.y).abs() < 0.001);
        finite(&rig.model);
        for _ in 0..80 {
            frame(&mut app, true);
            if app.world().get::<MotionState>(actor).unwrap().is_grounded() {
                break;
            }
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Land);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_held(false, true);
        frame(&mut app, true);
        assert_eq!(actor_rig(&app, actor).state, PoseState::Block);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        for _ in 0..8 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Dodge);
        assert!(pose_difference(&idle, &rig.model) > 0.1);
        finite(&rig.model);
    }

    #[test]
    fn blocked_actor_idles_and_zero_fixed_render_frames_keep_a_stable_walk_state() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        frame(&mut app, true);
        for _ in 0..4 {
            frame(&mut app, false);
            assert_eq!(actor_rig(&app, actor).state, PoseState::Walk);
        }
        app.world_mut().insert_resource(CollisionWorld::new(
            100.0,
            vec![crate::simulation::SolidBox::new(
                Vec2::new(1.0, 0.0),
                Vec2::splat(0.5),
            )],
        ));
        for _ in 0..30 {
            frame(&mut app, true);
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Idle);
    }

    #[test]
    fn authored_weapon_gestures_change_bones_and_return_to_guard() {
        let library = AnimationLibrary::default();
        for (kind, mode) in [
            (Gesture::Slash, WeaponMode::Melee),
            (Gesture::Axe, WeaponMode::Melee),
            (Gesture::Hammer, WeaponMode::Melee),
            (Gesture::Shot, WeaponMode::Ranged),
            (Gesture::Cannon, WeaponMode::Ranged),
            (Gesture::Windup, WeaponMode::Melee),
            (Gesture::Hit, WeaponMode::Melee),
            (Gesture::Guard, WeaponMode::Melee),
        ] {
            let a = library.model_pose(&poses::gesture(&library, kind, 0.01, mode));
            let b =
                library.model_pose(&poses::gesture(&library, kind, kind.duration() * 0.6, mode));
            assert!(
                pose_difference(&a, &b) > 0.001,
                "{kind:?} has no body motion"
            );
            finite(&a);
            finite(&b);
        }
        let mut rig = RigPose::new(&library);
        let initial = rig.model.clone();
        rig.observe(
            &library,
            CombatEffect::Hammer,
            WeaponSet::HammerSniper,
            false,
        );
        for _ in 0..12 {
            rig.advance(&library, idle_input(), 1.0 / 60.0);
        }
        assert!(pose_difference(&initial, &rig.model) > 0.05);
        for _ in 0..60 {
            rig.advance(&library, idle_input(), 1.0 / 60.0);
        }
        assert_eq!(rig.state, PoseState::Idle);
        finite(&rig.model);
    }

    #[test]
    fn actor_tagged_feedback_animates_only_the_performer_even_at_the_same_position() {
        use avian3d::prelude::CollisionLayers;
        let mut app = app();
        let performer = actor(&mut app);
        let bystander = actor(&mut app);
        // Isolate event ownership from capsule depenetration while keeping
        // both actors in the real physics world and colliding with the floor.
        for actor in [performer, bystander] {
            app.world_mut()
                .entity_mut(actor)
                .insert(CollisionLayers::new(0b01, 0b10));
        }
        app.world_mut()
            .get_mut::<CombatIntent>(performer)
            .unwrap()
            .request_attack();
        frame(&mut app, true);
        assert_eq!(
            actor_rig(&app, performer).state,
            PoseState::Attack(Gesture::Slash)
        );
        assert_eq!(actor_rig(&app, bystander).state, PoseState::Idle);
        let clock = actor_rig(&app, performer).clock;
        frame(&mut app, false);
        assert!(actor_rig(&app, performer).clock > clock);
    }

    #[test]
    fn pause_freezes_pose_clocks_and_feedback_cursor_then_resumes_without_physics_writes() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        frame(&mut app, true);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        let rig = actor_rig(&app, actor);
        let before = rig.model.clone();
        let clock = rig.clock;
        let sequence = rig.event_sequence;
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        for _ in 0..30 {
            frame(&mut app, false);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(pose_difference(&before, &rig.model), 0.0);
        assert_eq!(rig.clock, clock);
        assert_eq!(rig.event_sequence, sequence);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        for _ in 0..12 {
            frame(&mut app, false);
        }
        assert!(pose_difference(&before, &actor_rig(&app, actor).model) > 0.01);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().height(),
            height
        );
    }

    #[test]
    fn death_lowers_the_authored_head_and_respawn_restores_the_rig() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let head = library.bone("Head");
        let standing = rig.model[head].translation.y;
        let mut down = idle_input();
        down.alive = false;
        for _ in 0..60 {
            rig.advance(&library, down, 1.0 / 60.0);
        }
        assert_eq!(rig.state, PoseState::Down);
        assert!(rig.model[head].translation.y < standing - 0.5);
        finite(&rig.model);
        rig.observe(
            &library,
            CombatEffect::Respawn,
            WeaponSet::HammerSniper,
            false,
        );
        rig.advance(&library, idle_input(), 1.0 / 60.0);
        assert_eq!(rig.state, PoseState::Idle);
        assert!((rig.model[head].translation.y - standing).abs() < 0.05);
    }

    #[test]
    fn spin_turns_the_whole_rig_and_cannon_effects_only_use_leap_gestures_when_airborne() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let mut spin = idle_input();
        spin.spinning = true;
        spin.airborne = true;
        for _ in 0..6 {
            rig.advance(&library, spin, 1.0 / 60.0);
        }
        let first = rig.model.clone();
        for _ in 0..6 {
            rig.advance(&library, spin, 1.0 / 60.0);
        }
        assert!(pose_difference(&first, &rig.model) > 0.1);
        finite(&rig.model);
        rig.observe(&library, CombatEffect::Cannon, WeaponSet::AxeCannon, false);
        assert_eq!(rig.playback.as_ref().unwrap().kind, Gesture::Shot);
        rig.observe(&library, CombatEffect::Cannon, WeaponSet::AxeCannon, true);
        assert_eq!(rig.playback.as_ref().unwrap().kind, Gesture::Cannon);
    }

    #[test]
    fn a_partial_movement_step_slows_gait_without_rewinding_an_existing_walk_cycle() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let mut input = idle_input();
        input.speed = 6.0;
        for _ in 0..120 {
            rig.advance(&library, input, 1.0 / 60.0);
        }
        let before = rig.model.clone();
        let gait = rig.gait_time;
        input.speed = 0.3;
        rig.advance(&library, input, 1.0 / 60.0);
        assert_eq!(rig.state, PoseState::Walk);
        assert!(rig.gait_time > gait);
        assert!(pose_difference(&before, &rig.model) < 0.05);
        finite(&rig.model);
    }

    #[test]
    fn fixed_crouch_keeps_its_foot_anchor_then_walks_and_smoothly_stands_without_foot_drift() {
        let mut app = app();
        let actor = actor(&mut app);
        let head = app.world().resource::<AnimationLibrary>().bone("Head");
        let standing_head = actor_rig(&app, actor).model[head].translation.y;
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Crouch);
        assert!(rig.crouch_weight > 0.0 && rig.crouch_weight < 1.0);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, 1.0);
        assert!(rig.model[head].translation.y < standing_head - 0.2);
        let library = app.world().resource::<AnimationLibrary>();
        let base = library.model_pose(&rig.local);
        for side in ["l", "r"] {
            let foot = library.bone(&format!("foot_{side}"));
            assert!(rig.model[foot].translation.distance(base[foot].translation) < 0.002);
        }
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
        assert!(app.world().get::<Body>(actor).unwrap().position().length() < 0.001);
        let crouched = rig.model.clone();
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        for _ in 0..30 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Crouch);
        assert!(pose_difference(&crouched, &rig.model) > 0.01);
        finite(&rig.model);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert!(rig.crouch_weight > 0.0 && rig.crouch_weight < 1.0);
        let weight = rig.crouch_weight;
        let paused = rig.model.clone();
        let position = app.world().get::<Body>(actor).unwrap().position();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        for _ in 0..30 {
            frame(&mut app, false);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, weight);
        assert_eq!(pose_difference(&paused, &rig.model), 0.0);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        app.world_mut().get_mut::<MoveIntent>(actor).unwrap().stop();
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, 0.0);
        assert_eq!(rig.state, PoseState::Idle);
        assert!(rig.model[head].translation.y > standing_head - 0.1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
    }

    #[test]
    fn real_sprint_to_slide_uses_a_crouched_pose_then_keeps_crouching_after_the_slide() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(actor).unwrap();
            intent.set_movement_held(true);
            intent.set_held(true, false);
            intent.request_dodge();
        }
        for _ in 0..18 {
            frame(&mut app, true);
        }
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Slide);
        assert!(rig.crouch_weight > 0.0);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Slide);
        assert_eq!(rig.crouch_weight, 1.0);
        finite(&rig.model);
        for _ in 0..30 {
            frame(&mut app, true);
        }
        assert!(app.world().get::<MotionState>(actor).unwrap().is_crouched());
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_sliding());
        assert_eq!(actor_rig(&app, actor).state, PoseState::Crouch);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
    }
}
