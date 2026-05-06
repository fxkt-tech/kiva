use rand::seq::SliceRandom;

use crate::game::domain::{PlayerAiConfig, PlayerProfile};

#[derive(Debug, Clone)]
pub struct PlayerPool {
    profiles: Vec<PlayerProfile>,
}

impl Default for PlayerPool {
    fn default() -> Self {
        Self {
            profiles: default_profiles(),
        }
    }
}

impl PlayerPool {
    pub fn new(profiles: Vec<PlayerProfile>) -> Self {
        Self { profiles }
    }

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

fn default_profiles() -> Vec<PlayerProfile> {
    vec![
        profile("林澈", "冷静分析，偏好归纳投票逻辑", "林", "gpt-4.1-mini"),
        profile("许愿", "外向发言，喜欢主动带节奏", "许", "gpt-4.1-mini"),
        profile("周砚", "谨慎保守，倾向先听后站边", "周", "gpt-4.1"),
        profile("沈鹿", "直觉型判断，重视发言状态", "沈", "gpt-4.1-mini"),
        profile("顾白", "强势盘问，擅长找矛盾点", "顾", "gpt-4.1"),
        profile("唐棠", "温和协作，偏好整理公共信息", "唐", "gpt-4.1-mini"),
        profile("秦野", "冒险激进，愿意早期压迫投票", "秦", "gpt-4.1-mini"),
        profile("夏弥", "细节控，关注夜晚结果与票型", "夏", "gpt-4.1"),
        profile("陆青", "低调观察，发言短但抓重点", "陆", "gpt-4.1-mini"),
        profile("姜禾", "怀疑主义，默认验证每个声明", "姜", "gpt-4.1-mini"),
        profile("宋屿", "逻辑流玩家，重视因果链", "宋", "gpt-4.1"),
        profile("程星", "情绪感知强，善于判断发言动机", "程", "gpt-4.1-mini"),
    ]
}

fn profile(
    name: &'static str,
    personality_preference: &'static str,
    avatar: &'static str,
    model: &'static str,
) -> PlayerProfile {
    PlayerProfile {
        name: name.to_string(),
        personality_preference: personality_preference.to_string(),
        avatar: avatar.to_string(),
        ai: PlayerAiConfig {
            model: model.to_string(),
            api_key: "KIVA_AI_API_KEY".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        },
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
