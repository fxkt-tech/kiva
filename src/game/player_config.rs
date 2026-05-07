use crate::game::domain::{PlayerAiConfig, PlayerProfile};

pub fn default_profiles() -> Vec<PlayerProfile> {
    vec![
        profile("林澈", "冷静分析，偏好归纳投票逻辑", "林"),
        profile("许愿", "外向发言，喜欢主动带节奏", "许"),
        profile("周砚", "谨慎保守，倾向先听后站边", "周"),
        profile("沈鹿", "直觉型判断，重视发言状态", "沈"),
        profile("顾白", "强势盘问，擅长找矛盾点", "顾"),
        profile("唐棠", "温和协作，偏好整理公共信息", "唐"),
        profile("秦野", "冒险激进，愿意早期压迫投票", "秦"),
        profile("夏弥", "细节控，关注夜晚结果与票型", "夏"),
        profile("陆青", "低调观察，发言短但抓重点", "陆"),
        profile("姜禾", "怀疑主义，默认验证每个声明", "姜"),
        profile("宋屿", "逻辑流玩家，重视因果链", "宋"),
        profile("程星", "情绪感知强，善于判断发言动机", "程"),
    ]
}

pub fn apply_runtime_ai_config(profiles: &mut [PlayerProfile]) {
    let base_url = std::env::var("KIVA_LLM_BASE_URL").unwrap_or_default();
    let api_key = std::env::var("KIVA_LLM_API_KEY").unwrap_or_default();
    let model = std::env::var("KIVA_LLM_MODEL").unwrap_or_default();

    if base_url.trim().is_empty() || api_key.trim().is_empty() || model.trim().is_empty() {
        return;
    }

    for profile in profiles {
        profile.ai.base_url = base_url.clone();
        profile.ai.api_key = api_key.clone();
        profile.ai.model = model.clone();
    }
}

fn profile(
    name: &'static str,
    personality_preference: &'static str,
    avatar: &'static str,
) -> PlayerProfile {
    PlayerProfile {
        name: name.to_string(),
        personality_preference: personality_preference.to_string(),
        avatar: avatar.to_string(),
        ai: PlayerAiConfig {
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
            system_prompt: format!(
                "你是{name}，一名 AI 狼人杀玩家。你的性格：{personality_preference}。\
                你必须遵守当前身份的可见信息边界，隐藏自己的真实身份和私有信息，\
                除非公开跳身份能明显提高胜率。"
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn runtime_ai_config_applies_complete_global_env_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("KIVA_LLM_BASE_URL", "https://example.test/v1");
            std::env::set_var("KIVA_LLM_API_KEY", "test-key");
            std::env::set_var("KIVA_LLM_MODEL", "test-model");
        }
        let mut profiles = default_profiles();

        apply_runtime_ai_config(&mut profiles);

        assert!(profiles.iter().all(|profile| {
            profile.ai.base_url == "https://example.test/v1"
                && profile.ai.api_key == "test-key"
                && profile.ai.model == "test-model"
        }));
        unsafe {
            std::env::remove_var("KIVA_LLM_BASE_URL");
            std::env::remove_var("KIVA_LLM_API_KEY");
            std::env::remove_var("KIVA_LLM_MODEL");
        }
    }

    #[test]
    fn runtime_ai_config_ignores_incomplete_env_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("KIVA_LLM_BASE_URL", "https://example.test/v1");
            std::env::remove_var("KIVA_LLM_API_KEY");
            std::env::set_var("KIVA_LLM_MODEL", "test-model");
        }
        let mut profiles = default_profiles();

        apply_runtime_ai_config(&mut profiles);

        assert!(
            profiles
                .iter()
                .all(|profile| profile.ai.base_url.is_empty())
        );
        assert!(profiles.iter().all(|profile| profile.ai.api_key.is_empty()));
        assert!(profiles.iter().all(|profile| profile.ai.model.is_empty()));
        unsafe {
            std::env::remove_var("KIVA_LLM_BASE_URL");
            std::env::remove_var("KIVA_LLM_MODEL");
        }
    }
}
