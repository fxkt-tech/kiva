use super::{
    combat::{
        Action, Fighter,
        core::{AttackKind, DODGE_DURATION, ease},
    },
    input::Controls,
    player::{Joint, JointKind},
};
use bevy::prelude::*;

pub(super) fn animate_fighters(
    time: Res<Time>,
    controls: Res<Controls>,
    fighters: Query<&Fighter>,
    mut joints: Query<(&Joint, &mut Transform)>,
) {
    if !controls.active && !controls.reset {
        return;
    }
    for (joint, mut transform) in &mut joints {
        let Ok(fighter) = fighters.get(joint.owner) else {
            continue;
        };
        if fighter.hit_stop > 0.0 && !controls.reset {
            continue;
        }
        let mut pose = joint.rest;
        let gait = if fighter.moving {
            fighter.walk_phase.sin() * 0.48
        } else {
            0.0
        };
        match joint.kind {
            JointKind::Body => {
                pose.translation.y += if fighter.moving {
                    (fighter.walk_phase * 2.0).cos().abs() * 0.035
                } else {
                    (time.elapsed_secs() * 2.0).sin() * 0.008
                };
            }
            JointKind::LeftLeg => pose.rotation = Quat::from_rotation_x(gait),
            JointKind::RightLeg => pose.rotation = Quat::from_rotation_x(-gait),
            JointKind::LeftArm => pose.rotation = Quat::from_rotation_x(-gait * 0.6),
            JointKind::RightArm => pose.rotation = Quat::from_rotation_x(0.15 + gait * 0.3),
            JointKind::Wrist => pose.rotation = Quat::from_rotation_y(-0.12),
        }
        if fighter.airborne {
            match joint.kind {
                JointKind::LeftLeg => pose.rotation = Quat::from_rotation_x(-0.60),
                JointKind::RightLeg => pose.rotation = Quat::from_rotation_x(0.45),
                JointKind::Body => pose.rotation = Quat::from_rotation_x(0.12),
                JointKind::LeftArm => pose.rotation = Quat::from_rotation_z(0.35),
                _ => {}
            }
        }
        match &fighter.action {
            Action::Dodge { elapsed, direction } => {
                let weight =
                    (std::f32::consts::PI * (*elapsed / DODGE_DURATION).clamp(0.0, 1.0)).sin();
                let side = direction.dot(Quat::from_rotation_y(fighter.facing) * Vec3::X);
                match joint.kind {
                    JointKind::Body => {
                        pose.translation.y -= weight * 0.12;
                        pose.rotation = Quat::from_euler(
                            EulerRot::XYZ,
                            0.25 * weight,
                            0.0,
                            -0.45 * side * weight,
                        );
                    }
                    JointKind::LeftLeg => pose.rotation = Quat::from_rotation_x(-0.7 * weight),
                    JointKind::RightLeg => pose.rotation = Quat::from_rotation_x(0.8 * weight),
                    JointKind::RightArm => pose.rotation = Quat::from_rotation_x(0.9 * weight),
                    _ => {}
                }
            }
            Action::Attack(attack) => {
                let spec = attack.spec();
                let wind = ease(attack.elapsed / spec.windup);
                let strike = ease((attack.elapsed - spec.windup) / spec.active);
                let recover =
                    ((attack.elapsed - spec.windup - spec.active) / spec.recovery).clamp(0.0, 1.0);
                let weight = 1.0 - recover * recover;
                let reverse = matches!(attack.kind, AttackKind::Light(1));
                let heavy = matches!(attack.kind, AttackKind::Light(2));
                let sweep = if reverse {
                    1.15 * wind - 2.25 * strike
                } else {
                    -1.15 * wind + 2.25 * strike
                };
                match joint.kind {
                    JointKind::RightArm => {
                        pose.rotation = if heavy {
                            Quat::from_euler(
                                EulerRot::YXZ,
                                0.1,
                                (1.65 * wind - 1.7 * strike) * weight,
                                -0.18 * weight,
                            )
                        } else {
                            Quat::from_euler(
                                EulerRot::YXZ,
                                sweep * weight,
                                0.45 * weight,
                                -0.35 * weight,
                            )
                        };
                    }
                    JointKind::LeftArm => {
                        pose.rotation = Quat::from_euler(
                            EulerRot::XYZ,
                            0.55 * wind * weight,
                            0.0,
                            0.25 * weight,
                        )
                    }
                    JointKind::Wrist => {
                        pose.rotation = Quat::from_euler(
                            EulerRot::XYZ,
                            if heavy {
                                -0.2 * strike * weight
                            } else {
                                0.1 * weight
                            },
                            -sweep * 0.35 * weight,
                            if reverse {
                                -0.18 * weight
                            } else {
                                0.18 * weight
                            },
                        )
                    }
                    JointKind::LeftLeg => {
                        pose.rotation = Quat::from_rotation_x(-0.25 * wind * weight)
                    }
                    JointKind::RightLeg => {
                        pose.rotation = Quat::from_rotation_x(0.30 * strike * weight)
                    }
                    JointKind::Body => {
                        pose.rotation = Quat::from_euler(
                            EulerRot::XYZ,
                            if heavy { 0.16 * strike * weight } else { 0.0 },
                            sweep * 0.22 * weight,
                            0.0,
                        );
                        pose.translation.y -= 0.04 * wind * weight;
                    }
                }
            }
            Action::Guard => match joint.kind {
                JointKind::RightArm => {
                    pose.rotation = Quat::from_euler(EulerRot::YXZ, -0.65, 1.05, -0.20)
                }
                JointKind::LeftArm => {
                    pose.rotation = Quat::from_euler(EulerRot::XYZ, 1.15, 0.35, 0.35)
                }
                JointKind::Body => {
                    pose.translation.y -= 0.055;
                    pose.rotation = Quat::from_rotation_x(0.08);
                }
                _ => {}
            },
            Action::Stagger(remaining) => {
                if matches!(joint.kind, JointKind::Body) {
                    pose.rotation =
                        Quat::from_rotation_x(-0.27 * (remaining / 0.24).clamp(0.0, 1.0));
                }
            }
            Action::Down(_) => {
                if matches!(joint.kind, JointKind::Body) {
                    pose.translation.y = 0.22;
                    pose.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                }
            }
            Action::Idle => {}
        }
        *transform = pose;
    }
}
