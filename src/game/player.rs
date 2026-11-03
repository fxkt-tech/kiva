//! Shared low-poly character rig, used by the swordsman and training dummy.
use super::combat::{Fighter, Side};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

#[derive(Clone, Copy)]
pub(super) enum JointKind {
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
    Wrist,
}
#[derive(Component)]
pub(super) struct Joint {
    pub owner: Entity,
    pub kind: JointKind,
    pub rest: Transform,
}
#[derive(Component)]
pub(super) struct Appearance {
    pub material: Handle<StandardMaterial>,
    pub color: Color,
}
#[derive(Component)]
pub(super) struct SwordTip {
    pub owner: Entity,
    pub previous: Option<Vec3>,
}

struct RigMeshes {
    cube: Handle<Mesh>,
    head: Handle<Mesh>,
    limb: Handle<Mesh>,
}
fn part(
    parent: &mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    scale: Vec3,
) {
    parent.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(scale),
    ));
}

pub(super) fn spawn_fighters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rig = RigMeshes {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        head: meshes.add(Sphere::new(0.20).mesh().uv(16, 12)),
        limb: meshes.add(Capsule3d::new(0.095, 0.30)),
    };
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(188, 205, 210),
        metallic: 0.75,
        perceptual_roughness: 0.3,
        ..default()
    });
    let gold = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(190, 152, 69),
        metallic: 0.55,
        ..default()
    });
    let dark = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(29, 34, 38),
        ..default()
    });
    let skin = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(181, 139, 105),
        ..default()
    });
    for side in [Side::Player, Side::Dummy] {
        let color = if side == Side::Player {
            Color::srgb_u8(33, 106, 119)
        } else {
            Color::srgb_u8(155, 102, 65)
        };
        let cloth = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.8,
            ..default()
        });
        let blade = if side == Side::Player {
            metal.clone()
        } else {
            gold.clone()
        };
        let root = commands
            .spawn((
                Name::new(if side == Side::Player {
                    "Swordsman"
                } else {
                    "Training dummy"
                }),
                Fighter::new(side),
                Appearance {
                    material: cloth.clone(),
                    color,
                },
                Transform::from_translation(side.home())
                    .with_rotation(Quat::from_rotation_y(side.yaw())),
                Visibility::default(),
            ))
            .id();
        commands.entity(root).with_children(|root_children| {
            let rest = Transform::from_xyz(0.0, 0.9, 0.0);
            root_children
                .spawn((
                    Joint {
                        owner: root,
                        kind: JointKind::Body,
                        rest,
                    },
                    rest,
                    Visibility::default(),
                ))
                .with_children(|body| {
                    part(
                        body,
                        rig.cube.clone(),
                        cloth.clone(),
                        Vec3::new(0.0, 0.23, 0.0),
                        Vec3::new(0.59, 0.66, 0.35),
                    );
                    part(
                        body,
                        rig.cube.clone(),
                        dark.clone(),
                        Vec3::new(0.0, -0.04, 0.0),
                        Vec3::new(0.63, 0.10, 0.38),
                    );
                    part(
                        body,
                        rig.cube.clone(),
                        dark.clone(),
                        Vec3::new(0.0, -0.16, 0.0),
                        Vec3::new(0.5, 0.24, 0.33),
                    );
                    part(
                        body,
                        rig.cube.clone(),
                        gold.clone(),
                        Vec3::new(0.0, 0.23, -0.183),
                        Vec3::new(0.055, 0.58, 0.025),
                    );
                    part(
                        body,
                        rig.head.clone(),
                        if side == Side::Player {
                            skin.clone()
                        } else {
                            cloth.clone()
                        },
                        Vec3::new(0.0, 0.80, 0.0),
                        Vec3::ONE,
                    );
                    part(
                        body,
                        rig.cube.clone(),
                        dark.clone(),
                        Vec3::new(0.0, 0.91, 0.0),
                        Vec3::new(0.37, 0.09, 0.36),
                    );
                    part(
                        body,
                        rig.cube.clone(),
                        dark.clone(),
                        Vec3::new(0.0, 0.79, -0.185),
                        Vec3::new(0.24, 0.045, 0.04),
                    );
                    // Layered robe, shoulder guards and a tied-back hair silhouette.
                    part(
                        body,
                        rig.cube.clone(),
                        cloth.clone(),
                        Vec3::new(0.0, -0.17, 0.0),
                        Vec3::new(0.66, 0.32, 0.41),
                    );
                    for x in [-0.31, 0.31] {
                        part(
                            body,
                            rig.head.clone(),
                            if side == Side::Player {
                                metal.clone()
                            } else {
                                gold.clone()
                            },
                            Vec3::new(x, 0.47, 0.0),
                            Vec3::new(0.95, 0.45, 0.85),
                        );
                    }
                    if side == Side::Player {
                        part(
                            body,
                            rig.head.clone(),
                            dark.clone(),
                            Vec3::new(0.0, 0.93, 0.02),
                            Vec3::new(1.02, 0.54, 1.02),
                        );
                        part(
                            body,
                            rig.limb.clone(),
                            dark.clone(),
                            Vec3::new(0.0, 0.66, 0.22),
                            Vec3::new(0.6, 0.85, 0.6),
                        );
                        for (x, angle) in [(-0.12, -0.23_f32), (0.12, 0.23_f32)] {
                            body.spawn((
                                Mesh3d(rig.cube.clone()),
                                MeshMaterial3d(dark.clone()),
                                Transform::from_xyz(x, 0.27, -0.19)
                                    .with_rotation(Quat::from_rotation_z(angle))
                                    .with_scale(Vec3::new(0.11, 0.62, 0.035)),
                            ));
                        }
                    }
                    for (kind, x) in [(JointKind::LeftLeg, -0.16), (JointKind::RightLeg, 0.16)] {
                        let rest = Transform::from_xyz(x, -0.28, 0.0);
                        body.spawn((
                            Joint {
                                owner: root,
                                kind,
                                rest,
                            },
                            rest,
                            Visibility::default(),
                        ))
                        .with_children(|leg| {
                            part(
                                leg,
                                rig.limb.clone(),
                                dark.clone(),
                                Vec3::new(0.0, -0.24, 0.0),
                                Vec3::new(1.1, 1.2, 1.1),
                            );
                            part(
                                leg,
                                rig.cube.clone(),
                                dark.clone(),
                                Vec3::new(0.0, -0.51, -0.045),
                                Vec3::new(0.22, 0.14, 0.34),
                            );
                        });
                    }
                    for (kind, x) in [(JointKind::LeftArm, -0.40), (JointKind::RightArm, 0.40)] {
                        let rest = Transform::from_xyz(x, 0.49, 0.0);
                        body.spawn((
                            Joint {
                                owner: root,
                                kind,
                                rest,
                            },
                            rest,
                            Visibility::default(),
                        ))
                        .with_children(|arm| {
                            part(
                                arm,
                                rig.limb.clone(),
                                cloth.clone(),
                                Vec3::new(0.0, -0.12, 0.0),
                                Vec3::new(1.05, 0.6, 1.05),
                            );
                            part(
                                arm,
                                rig.limb.clone(),
                                if side == Side::Player {
                                    skin.clone()
                                } else {
                                    cloth.clone()
                                },
                                Vec3::new(0.0, -0.34, 0.0),
                                Vec3::new(0.82, 0.6, 0.82),
                            );
                            part(
                                arm,
                                rig.head.clone(),
                                dark.clone(),
                                Vec3::new(0.0, -0.44, 0.0),
                                Vec3::splat(0.48),
                            );
                            if matches!(kind, JointKind::RightArm) {
                                arm.spawn((
                                    Joint {
                                        owner: root,
                                        kind: JointKind::Wrist,
                                        rest: Transform::from_xyz(0.0, -0.47, -0.03),
                                    },
                                    Transform::from_xyz(0.0, -0.47, -0.03),
                                    Visibility::default(),
                                ))
                                .with_children(|sword| {
                                    part(
                                        sword,
                                        rig.cube.clone(),
                                        dark.clone(),
                                        Vec3::new(0.0, 0.0, 0.06),
                                        Vec3::new(0.065, 0.065, 0.23),
                                    );
                                    part(
                                        sword,
                                        rig.cube.clone(),
                                        gold.clone(),
                                        Vec3::new(0.0, 0.0, -0.08),
                                        Vec3::new(0.29, 0.05, 0.065),
                                    );
                                    part(
                                        sword,
                                        rig.cube.clone(),
                                        blade.clone(),
                                        Vec3::new(0.0, 0.0, -0.55),
                                        Vec3::new(0.07, 0.028, 0.91),
                                    );
                                    sword.spawn((
                                        SwordTip {
                                            owner: root,
                                            previous: None,
                                        },
                                        Transform::from_xyz(0.0, 0.0, -1.0),
                                        Visibility::default(),
                                    ));
                                });
                            }
                        });
                    }
                });
        });
    }
}
