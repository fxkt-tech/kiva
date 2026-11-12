//! Developer overlay (F4): the blade sweeps, target capsules and contacts the
//! melee rules actually tested, drawn over the scene. It reads the
//! simulation's probe history and decides nothing.
use bevy::prelude::*;

use crate::simulation::{HitProbes, ProbeShape};

/// F4 switches the simulation's probe recording, and with it this overlay.
pub(super) fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut probes: ResMut<HitProbes>,
    mut config: ResMut<GizmoConfigStore>,
) {
    if keys.just_pressed(KeyCode::F4) {
        probes.enabled = !probes.enabled;
        // Sweeps happen inside bodies; draw them over the scene.
        config.config_mut::<DefaultGizmoConfigGroup>().0.depth_bias = -1.0;
    }
}

pub(super) fn draw(probes: Res<HitProbes>, mut gizmos: Gizmos) {
    if !probes.enabled {
        return;
    }
    let now = probes.tick();
    for probe in probes.entries() {
        let age = now.saturating_sub(probe.tick) as f32 / HitProbes::LIFETIME as f32;
        let alpha = (1.0 - age).clamp(0.0, 1.0);
        match probe.shape {
            ProbeShape::Blade { blade, radius } => {
                let color = Color::srgba(1.0, 0.75, 0.2, alpha);
                gizmos.line(blade[0], blade[1], color);
                gizmos.sphere(Isometry3d::from_translation(blade[1]), radius, color);
            }
            ProbeShape::Hurtbox(hurtbox) => {
                let color = Color::srgba(0.3, 0.9, 0.85, alpha * 0.6);
                let [bottom, top] = hurtbox.segment();
                let radius = hurtbox.radius();
                let flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
                for centre in [bottom, top] {
                    gizmos.circle(Isometry3d::new(centre, flat), radius, color);
                }
                for side in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
                    gizmos.line(bottom + side * radius, top + side * radius, color);
                }
            }
            ProbeShape::Contact(point) => {
                let color = Color::srgba(1.0, 0.2, 0.2, alpha);
                gizmos.sphere(Isometry3d::from_translation(point), 0.07, color);
            }
        }
    }
}
