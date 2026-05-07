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
