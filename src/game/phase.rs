/// 表示游戏当前所处的阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    /// 游戏开始阶段。
    Start,
    /// 指定轮次的夜晚阶段。
    Night(u32),
    /// 指定轮次的天亮阶段。
    Dawn(u32),
    /// 指定轮次的白天发言阶段。
    DaySpeech(u32),
    /// 指定轮次的投票阶段。
    Vote(u32),
    /// 指定轮次的猎人开枪阶段。
    HunterShot(u32),
    /// 游戏结束后的复盘阶段。
    Review,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_can_track_numbered_night() {
        assert_eq!(GamePhase::Night(2), GamePhase::Night(2));
    }
}
