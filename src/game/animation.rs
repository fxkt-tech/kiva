use super::{
    combat::{
        Action, Fighter,
        core::{AttackKind, DODGE_DURATION},
    },
    input::Controls,
    player::{BoneVisual, attachment_transform},
    skeleton::{AnimationLibrary, blend},
};
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PoseState {
    Idle,
    Run,
    Guard,
    Attack(u8),
    Air(u8),
    Land,
    Dodge,
    Stagger,
    Down,
}
#[derive(Component)]
pub(super) struct RigPose {
    local: Vec<Transform>,
    pub model: Vec<Transform>,
    from: Vec<Transform>,
    state: PoseState,
    clock: f32,
    transition: f32,
    was_airborne: bool,
    dodge_yaw: f32,
}
impl RigPose {
    pub fn new(library: &AnimationLibrary) -> Self {
        let local = library.sample("Sword_Idle", 0.0, true);
        Self {
            model: library.model_pose(&local),
            from: local.clone(),
            local,
            state: PoseState::Idle,
            clock: 0.0,
            transition: 1.0,
            was_airborne: false,
            dodge_yaw: 0.0,
        }
    }
}
fn attack_pose(library: &AnimationLibrary, attack: &super::combat::core::Attack) -> Vec<Transform> {
    let spec = attack.spec();
    let index = match attack.kind {
        AttackKind::Light(i) => i,
        AttackKind::Training => 2,
    };
    // Source markers split preparation, contact and follow-through. Time-warp
    // each phase to the gameplay's actual damage window, including training.
    let (name, contact, follow) = match index {
        0 => ("Sword_Regular_A", 0.20, 0.40),
        1 => ("Sword_Regular_B", 0.23, 0.50),
        _ => ("Sword_Regular_C", 0.60, 0.95),
    };
    let t = attack.elapsed;
    if t < spec.windup {
        library.sample(name, contact * (t / spec.windup), false)
    } else if t < spec.windup + spec.active {
        library.sample(
            name,
            contact + (follow - contact) * ((t - spec.windup) / spec.active),
            false,
        )
    } else {
        let recovery = ((t - spec.windup - spec.active) / spec.recovery).clamp(0.0, 1.0);
        if index < 2 {
            let rec = if index == 0 {
                "Sword_Regular_A_Rec"
            } else {
                "Sword_Regular_B_Rec"
            };
            let source = library.sample(name, follow, false);
            let target = library.sample(rec, library.duration(rec) * recovery, false);
            source
                .into_iter()
                .zip(target)
                .map(|(a, b)| blend(a, b, (recovery * 8.0).min(1.0)))
                .collect()
        } else {
            library.sample(
                name,
                follow + (library.duration(name) - follow) * recovery,
                false,
            )
        }
    }
}
pub(super) fn animate_fighters(
    time: Res<Time>,
    controls: Res<Controls>,
    library: Res<AnimationLibrary>,
    mut fighters: Query<(&Fighter, &mut RigPose)>,
    mut visuals: Query<(&BoneVisual, &mut Transform)>,
) {
    for (fighter, mut rig) in &mut fighters {
        if controls.reset {
            *rig = RigPose::new(&library);
            continue;
        }
        if !controls.active || fighter.hit_stop > 0.0 {
            continue;
        }
        let dt = time.delta_secs();
        let landed = rig.was_airborne && !fighter.airborne;
        rig.was_airborne = fighter.airborne;
        let state = match &fighter.action {
            Action::Attack(a) => PoseState::Attack(match a.kind {
                AttackKind::Light(i) => i as u8,
                AttackKind::Training => 3,
            }),
            Action::Guard => PoseState::Guard,
            Action::Dodge { .. } => PoseState::Dodge,
            Action::Stagger(_) => PoseState::Stagger,
            Action::Down(_) => PoseState::Down,
            Action::Idle if fighter.airborne => PoseState::Air(fighter.jumps.used()),
            Action::Idle if landed || (rig.state == PoseState::Land && rig.clock < 0.16) => {
                PoseState::Land
            }
            Action::Idle if fighter.moving => PoseState::Run,
            Action::Idle => PoseState::Idle,
        };
        if rig.state != state {
            rig.from = rig.local.clone();
            rig.state = state;
            rig.clock = 0.0;
            rig.transition = 0.0;
            if let Action::Dodge { direction, .. } = &fighter.action {
                rig.dodge_yaw = (-direction.x).atan2(-direction.z) - fighter.facing;
            }
        }
        rig.clock += dt;
        rig.transition += dt;
        let mut target = match &fighter.action {
            Action::Attack(attack) => attack_pose(&library, attack),
            Action::Guard => {
                library.sample("Sword_Block", 0.40 + (rig.clock * 1.7).sin() * 0.025, false)
            }
            Action::Dodge { elapsed, .. } => library.sample(
                "Roll",
                elapsed / DODGE_DURATION * library.duration("Roll"),
                false,
            ),
            Action::Stagger(remaining) => library.sample(
                "Hit_Chest",
                (0.24 - remaining) / 0.24 * library.duration("Hit_Chest"),
                false,
            ),
            Action::Down(remaining) => library.sample(
                "Death01",
                (2.0 - remaining) / 0.8 * library.duration("Death01"),
                false,
            ),
            Action::Idle if fighter.airborne => {
                if fighter.jumps.used() >= 2 {
                    library.sample("NinjaJump_Idle_Loop", rig.clock, true)
                } else {
                    let start =
                        library.sample("Jump_Start", 0.30 + rig.clock.min(0.14) * 3.5, false);
                    let air = library.sample("Jump_Loop", (rig.clock - 0.14).max(0.0), true);
                    let weight = ((rig.clock - 0.14) / 0.10).clamp(0.0, 1.0);
                    start
                        .into_iter()
                        .zip(air)
                        .map(|(a, b)| blend(a, b, weight))
                        .collect()
                }
            }
            Action::Idle if state == PoseState::Land => {
                library.sample("Jump_Land", 0.15 + rig.clock * 4.0, false)
            }
            Action::Idle if fighter.moving => library.sample("Jog_Fwd_Loop", rig.clock * 1.2, true),
            Action::Idle => library.sample("Sword_Idle", rig.clock, true),
        };
        if state == PoseState::Guard && (fighter.moving || fighter.airborne) {
            // Keep the blocking arms, with an independently animated lower
            // body. Indices follow the verified parent-first import layout.
            let lower = if fighter.airborne {
                library.sample("Jump_Loop", rig.clock, true)
            } else {
                library.sample("Walk_Loop", rig.clock * 0.85, true)
            };
            target[1] = lower[1]; // pelvis
            let first_leg = library.bone("thigh_l");
            target[first_leg..].copy_from_slice(&lower[first_leg..]);
        }
        if state == PoseState::Dodge {
            target[0].rotation = Quat::from_rotation_y(rig.dodge_yaw) * target[0].rotation;
        }
        let transition_seconds = match state {
            PoseState::Attack(_) => 0.055,
            PoseState::Stagger => 0.035,
            _ => 0.12,
        };
        let w = (rig.transition / transition_seconds).clamp(0.0, 1.0);
        let w = w * w * (3.0 - 2.0 * w);
        rig.local = rig
            .from
            .iter()
            .copied()
            .zip(target)
            .map(|(a, b)| blend(a, b, w))
            .collect();
        rig.model = library.model_pose(&rig.local);
    }
    for (visual, mut transform) in &mut visuals {
        if let Ok((_, rig)) = fighters.get(visual.owner) {
            *transform = attachment_transform(&visual.attachment, &rig.model);
        }
    }
}
