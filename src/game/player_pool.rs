use rand::seq::SliceRandom;

use crate::game::domain::PlayerProfile;
use crate::game::player_config;

#[derive(Debug, Clone)]
pub struct PlayerPool {
    profiles: Vec<PlayerProfile>,
}

impl Default for PlayerPool {
    fn default() -> Self {
        Self {
            profiles: player_config::default_profiles(),
        }
    }
}

impl PlayerPool {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(profiles: Vec<PlayerProfile>) -> Self {
        Self { profiles }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn draw_nine<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Option<Vec<PlayerProfile>> {
        const PLAYER_COUNT: usize = 9;

        if self.profiles.len() < PLAYER_COUNT {
            return None;
        }

        let mut profiles = self.profiles.clone();
        profiles.shuffle(rng);
        profiles.truncate(PLAYER_COUNT);
        Some(profiles)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn default_pool_has_more_than_nine_players() {
        assert!(PlayerPool::default().len() > 9);
    }

    #[test]
    fn default_ai_configs_are_unconfigured_but_have_system_prompts() {
        let pool = PlayerPool::default();
        let selected = pool.draw_nine(&mut rand::rng()).unwrap();

        for profile in selected {
            assert!(profile.ai.base_url.is_empty());
            assert!(profile.ai.api_key.is_empty());
            assert!(profile.ai.model.is_empty());
            assert!(!profile.ai.system_prompt.trim().is_empty());
        }
    }

    #[test]
    fn draw_nine_selects_nine_unique_players() {
        let pool = PlayerPool::default();
        let selected = pool.draw_nine(&mut rand::rng()).unwrap();
        let unique_names = selected
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(selected.len(), 9);
        assert_eq!(unique_names.len(), 9);
    }

    #[test]
    fn draw_nine_returns_none_when_pool_is_too_small() {
        let pool = PlayerPool::new(Vec::new());

        assert!(pool.draw_nine(&mut rand::rng()).is_none());
    }
}
