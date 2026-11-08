//! Simulation progress and permission to issue human input are distinct.
use crate::client::web::PointerCapture;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(in crate::client) struct GameplayInput {
    pub blocked: bool,
}

pub(super) struct Access {
    pub paused: bool,
    pub gameplay: bool,
}

impl Access {
    pub fn new(
        active: bool,
        focused: bool,
        pause_requested: bool,
        capture: Option<PointerCapture>,
        settings_open: bool,
    ) -> Self {
        // Esc can release the actual lock before pointerlockchange is sent.
        // Losing input alone must not insert a pause/Stop into a running game.
        let paused = !active || !focused || pause_requested;
        let gameplay = !paused && !settings_open && capture.is_none_or(PointerCapture::confirmed);
        Self { paused, gameplay }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_and_unconfirmed_capture_block_input_without_pausing_the_world() {
        for capture in [
            PointerCapture {
                actual: false,
                event_confirmed: false,
            },
            PointerCapture {
                actual: true,
                event_confirmed: false,
            },
            PointerCapture {
                actual: true,
                event_confirmed: true,
            },
        ] {
            let access = Access::new(true, true, false, Some(capture), true);
            assert!(!access.paused);
            assert!(!access.gameplay);
        }
        let released_before_notification =
            Access::new(true, true, false, Some(PointerCapture::default()), false);
        assert!(!released_before_notification.paused);
        assert!(!released_before_notification.gameplay);
    }

    #[test]
    fn no_player_focus_loss_and_explicit_pause_still_freeze_settings() {
        for (active, focused, pause) in [
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ] {
            let access = Access::new(active, focused, pause, None, true);
            assert!(access.paused);
            assert!(!access.gameplay);
        }
        assert!(Access::new(true, true, false, None, false).gameplay);
        assert!(
            Access::new(
                true,
                true,
                false,
                Some(PointerCapture {
                    actual: true,
                    event_confirmed: true
                }),
                false
            )
            .gameplay
        );
    }
}
