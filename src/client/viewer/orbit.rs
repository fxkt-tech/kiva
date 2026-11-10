//! Inspection-only orbit controls, independent of animation and game input capture.
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub(super) struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fit_requested: bool,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            target: Vec3::new(0., 0.95, 0.),
            yaw: std::f32::consts::PI - 0.4,
            pitch: 0.18,
            distance: 3.8,
            fit_requested: true,
        }
    }
}
impl Orbit {
    pub fn transform(&self) -> Transform {
        let direction = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        );
        Transform::from_translation(self.target + direction * self.distance)
            .looking_at(self.target, Vec3::Y)
    }
    pub fn rotate(&mut self, delta: Vec2) {
        self.yaw = (self.yaw - delta.x * 0.006).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + delta.y * 0.006).clamp(-1.54, 1.54);
    }
    pub fn pan(&mut self, delta: Vec2, height: f32) {
        let camera = self.transform();
        let scale = 2. * self.distance * (25f32.to_radians()).tan() / height.max(1.);
        self.target += camera.rotation * Vec3::new(-delta.x, delta.y, 0.) * scale;
    }
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (delta * 0.001).exp()).clamp(0.12, 80.);
    }
    pub fn view(&mut self, name: &str) {
        use std::f32::consts::{FRAC_PI_2, PI};
        (self.yaw, self.pitch) = match name {
            "front" => (PI, 0.),
            "back" => (0., 0.),
            "left" => (-FRAC_PI_2, 0.),
            "right" => (FRAC_PI_2, 0.),
            "top" => (PI, 1.54),
            _ => (PI - 0.4, 0.18),
        };
    }
    pub fn fit(&mut self, min: Vec3, max: Vec3, aspect: f32) {
        if !min.is_finite() || !max.is_finite() || min.cmpgt(max).any() {
            return;
        }
        let vertical = 50f32.to_radians();
        let horizontal = 2. * ((vertical * 0.5).tan() * aspect.max(0.1)).atan();
        let half_angle = vertical.min(horizontal) * 0.5;
        self.target = (min + max) * 0.5;
        self.distance =
            (((max - min).length() * 0.5).max(0.15) / half_angle.sin() * 1.12).clamp(0.12, 80.);
        self.fit_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_contains_all_corners_at_portrait_and_wide_aspects_without_changing_direction() {
        let min = Vec3::new(-0.8, -0.2, -0.5);
        let max = Vec3::new(0.6, 2.7, 1.1);
        for aspect in [0.5, 1., 2.4] {
            let mut orbit = Orbit::default();
            let yaw = orbit.yaw;
            let pitch = orbit.pitch;
            orbit.fit(min, max, aspect);
            assert_eq!(orbit.yaw, yaw);
            assert_eq!(orbit.pitch, pitch);
            let inverse = orbit.transform().compute_affine().inverse();
            for x in [min.x, max.x] {
                for y in [min.y, max.y] {
                    for z in [min.z, max.z] {
                        let p = inverse.transform_point3(Vec3::new(x, y, z));
                        assert!(p.z < 0.);
                        let half = (-p.z) * (25f32.to_radians()).tan();
                        assert!(p.y.abs() < half && p.x.abs() < half * aspect);
                    }
                }
            }
        }
    }
    #[test]
    fn camera_controls_remain_finite_and_clamped_at_extreme_inputs() {
        let mut orbit = Orbit::default();
        for _ in 0..10 {
            orbit.rotate(Vec2::new(2000., 2000.));
            orbit.zoom(-2000.);
            orbit.pan(Vec2::new(50., 20.), 600.);
        }
        assert!(orbit.transform().to_matrix().is_finite());
        assert!(orbit.pitch <= 1.54);
        assert!(orbit.distance >= 0.12);
        orbit.view("top");
        assert!(orbit.transform().to_matrix().is_finite());
    }
}
