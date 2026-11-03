use super::{
    combat::{
        Action, Fighter,
        core::{AttackAxis, AttackKind, CHARGE_READY, DODGE_DURATION},
    },
    input::Controls,
    player::{Attachment, BoneVisual, attachment_transform},
    skeleton::{AnimationLibrary, blend},
};
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PoseState {
    Idle,
    Run,
    Charge(AttackAxis),
    Parry,
    Crouch,
    Slide,
    Attack(AttackKind),
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
    let index = if attack.kind.axis() == AttackAxis::Vertical {
        2
    } else {
        match attack.kind {
            AttackKind::Light(_, 1) => 1,
            _ => 0,
        }
    };
    // Source markers split preparation, contact and follow-through. Time-warp
    // each phase to the gameplay's actual damage window, including training.
    let (name, contact, follow) = match index {
        0 => ("Sword_Regular_A", 0.20, 0.40),
        1 => ("Sword_Regular_B", 0.23, 0.50),
        _ => ("Sword_Regular_C", 0.60, 0.95),
    };
    let sample = |name: &str, seconds: f32, looping: bool| {
        library.sample(
            name,
            if attack.kind == AttackKind::Crouch(AttackAxis::Vertical) {
                (1.15 - seconds).max(0.0)
            } else {
                seconds
            },
            looping,
        )
    };
    let t = attack.elapsed;
    if t < spec.windup {
        sample(name, contact * (t / spec.windup), false)
    } else if t < spec.windup + spec.active {
        sample(
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
            let source = sample(name, follow, false);
            let target = sample(rec, library.duration(rec) * recovery, false);
            source
                .into_iter()
                .zip(target)
                .map(|(a, b)| blend(a, b, (recovery * 8.0).min(1.0)))
                .collect()
        } else {
            sample(
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
    mut visuals: Query<(&BoneVisual, &mut Transform, Option<&mut Visibility>)>,
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
            Action::Attack(a) => PoseState::Attack(a.kind),
            Action::Charge(charge) => PoseState::Charge(charge.axis),
            Action::Parry(_) => PoseState::Parry,
            Action::Slide { .. } => PoseState::Slide,
            Action::Dodge { .. } => PoseState::Dodge,
            Action::Stagger(_) => PoseState::Stagger,
            Action::Down(_) => PoseState::Down,
            Action::Idle if fighter.airborne => PoseState::Air(fighter.jumps.used()),
            Action::Idle if landed || (rig.state == PoseState::Land && rig.clock < 0.16) => {
                PoseState::Land
            }
            Action::Idle if fighter.crouched => PoseState::Crouch,
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
            Action::Charge(charge) => library.sample(
                if charge.axis == AttackAxis::Horizontal {
                    "Sword_Regular_A"
                } else {
                    "Sword_Regular_C"
                },
                (charge.elapsed / CHARGE_READY).min(1.0)
                    * if charge.axis == AttackAxis::Horizontal {
                        0.18
                    } else {
                        0.5
                    },
                false,
            ),
            Action::Parry(_) => library.sample("Sword_Block", 0.2 + rig.clock.min(0.2), false),
            Action::Slide { .. } => library.sample("Sword_Idle", 0.0, false),
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
            Action::Idle if fighter.moving => library.sample(
                if fighter.sprinting {
                    "Jog_Fwd_Loop"
                } else {
                    "Walk_Loop"
                },
                rig.clock * if fighter.sprinting { 1.6 } else { 1.2 },
                true,
            ),
            Action::Idle => library.sample("Sword_Idle", rig.clock, true),
        };
        if matches!(state, PoseState::Charge(_) | PoseState::Parry)
            && (fighter.moving || fighter.airborne)
        {
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
        if fighter.crouched || state == PoseState::Slide {
            target[library.bone("pelvis")].translation.y -= 0.3;
            for side in ["l", "r"] {
                target[library.bone(&format!("thigh_{side}"))].rotation *=
                    Quat::from_rotation_x(-0.55);
                target[library.bone(&format!("calf_{side}"))].rotation *=
                    Quat::from_rotation_x(1.05);
            }
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
    for (visual, mut transform, visibility) in &mut visuals {
        if let Ok((fighter, rig)) = fighters.get(visual.owner) {
            if matches!(visual.attachment, Attachment::Sword { .. })
                && let Some(mut visibility) = visibility
            {
                *visibility = if fighter.disarmed > 0.0 {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            }
            *transform = attachment_transform(&visual.attachment, &rig.model);
        }
    }
}
