//! Input-frame pointer snapshot. The browser owns and draws its live coordinates.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct GamePointer {
    position: Option<Vec2>,
    captured: bool,
    size: Vec2,
}

impl GamePointer {
    pub fn sync(&mut self, captured: bool, size: Vec2, absolute: Option<Vec2>) -> bool {
        let just_captured = captured && !self.captured;
        self.size = size.max(Vec2::ONE);
        if captured && !self.captured {
            self.position = Some(self.size * 0.5);
        }
        if !captured {
            self.position = absolute;
        }
        self.captured = captured;
        if captured {
            self.clamp();
        }
        just_captured
    }

    pub fn position(&self) -> Option<Vec2> {
        self.position
    }
    pub fn absolute(&mut self, position: Option<Vec2>) {
        if !self.captured {
            self.position = position;
        }
    }
    pub fn observe(&mut self, point: Vec2) {
        if self.captured && point.is_finite() {
            self.position = Some(point);
            self.clamp();
        }
    }
    fn clamp(&mut self) {
        self.position = self
            .position
            .map(|point| point.clamp(Vec2::ZERO, self.size - Vec2::ONE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_starts_at_center_and_discards_motion_beyond_the_edge() {
        let mut pointer = GamePointer::default();
        pointer.sync(true, Vec2::new(1280.0, 800.0), None);
        assert_eq!(pointer.position(), Some(Vec2::new(640.0, 400.0)));
        pointer.observe(Vec2::new(10000.0, -10000.0));
        assert_eq!(pointer.position(), Some(Vec2::new(1279.0, 0.0)));
        pointer.observe(Vec2::new(1277.0, 2.0));
        assert_eq!(pointer.position(), Some(Vec2::new(1277.0, 2.0)));
    }

    #[test]
    fn locked_absolute_events_cannot_move_the_virtual_cursor() {
        let mut pointer = GamePointer::default();
        pointer.sync(true, Vec2::splat(800.0), None);
        pointer.absolute(Some(Vec2::splat(-100.0)));
        assert_eq!(pointer.position(), Some(Vec2::splat(400.0)));
    }

    #[test]
    fn resize_clamps_and_recapture_recenters() {
        let mut pointer = GamePointer::default();
        pointer.sync(true, Vec2::splat(1000.0), None);
        pointer.observe(Vec2::splat(1500.0));
        pointer.sync(true, Vec2::splat(600.0), None);
        assert_eq!(pointer.position(), Some(Vec2::splat(599.0)));
        pointer.sync(false, Vec2::splat(600.0), Some(Vec2::ONE));
        pointer.sync(true, Vec2::splat(600.0), None);
        assert_eq!(pointer.position(), Some(Vec2::splat(300.0)));
    }
}
