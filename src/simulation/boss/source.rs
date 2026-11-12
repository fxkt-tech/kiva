//! The source encounter's decisions, transcribed node by node from
//! `reparsed/ai` of the boss_01 package (kept outside this repository, in
//! `~/go/src/github.com/fxkt-tech/mysrc/boss_01`): the phase trees `51000025_BOSS01_fight_P1` and
//! `51000031_BOSS01_fight_P2`, the dispatch tree `51000025_BOSS01_fight_Action`
//! and the combo trees `51000025_BOSS01_P1_ComboCondition` /
//! `51000031_BOSS01_P2_ComboCondition`. Action numbers, weights, distances,
//! timers and cooldowns are the source's own. What the source decides with
//! systems Kiva lacks (achievement buffs, obstacle probes, lock and dodge
//! events) is left out and named where it would be.
use super::BossPhase;

/// Action -> source skill slot, from the dispatch tree's `ApplySkillAction`
/// nodes (cases 4..60).
pub(super) const SLOTS: [u32; 57] = [
    90021629, 90021435, 90021632, 90021633, 90021626, 90021627, 90021593, 90021594, 90021595,
    90021596, 90021597, 90021598, 90021599, 90021600, 90021601, 90021602, 90021603, 90021604,
    90021605, 90021606, 90021607, 90021608, 90021609, 90021610, 90021611, 90021612, 90021613,
    90021614, 90021615, 90021616, 90021625, 90021631, 90021634, 90021635, 90021636, 90021637,
    90021628, 90021639, 90021640, 90021638, 90021622, 90021641, 90021623, 90021646, 90021647,
    90021648, 90021649, 90021650, 90021651, 90021652, 90021653, 90021654, 90021655, 90021656,
    90021657, 90021658, 90021645,
];

/// Seconds an action's weight stays zero after it is dispatched, from the
/// dispatch nodes with `action_flow_modify=true`; zero elsewhere.
const COOLDOWNS: [f32; 57] = [
    0., 0., 0., 0., 0., 0., 40., 55., 50., 65., 60., 0., 50., 60., 60., 55., 70., 65., 0., 80.,
    65., 55., 0., 75., 0., 75., 75., 55., 55., 0., 0., 35., 45., 60., 65., 60., 55., 60., 45., 55.,
    85., 45., 85., 0., 65., 50., 65., 65., 50., 55., 55., 60., 45., 50., 40., 55., 0.,
];

/// Kiva adaptation, the one change to a source number here: cooldowns run
/// at a quarter of their source length. At full length the near pools empty
/// after about four strikes and the Boss circles for a third of the fight;
/// the source fight has player movement, hit reactions and events which keep
/// it busy and Kiva does not. `1.0` restores the source pacing.
pub(super) const COOLDOWN_SCALE: f32 = 0.25;

pub(super) fn slot(action: u16) -> Option<u32> {
    (4..=60)
        .contains(&action)
        .then(|| SLOTS[usize::from(action - 4)])
}

fn cooldown(action: u16) -> f32 {
    if (4..=60).contains(&action) {
        COOLDOWNS[usize::from(action - 4)]
    } else {
        0.0
    }
}

/// Whether dispatching `action` adds to `attack_count`. Actions 12 and 45
/// skip it once phase one has marked its target (buff 450097, below 65%).
fn counts_as_attack(action: u16, marked: bool) -> bool {
    match action {
        10 | 11 | 27 | 29 | 30 | 50 | 59 => true,
        12 | 45 => !marked,
        _ => false,
    }
}

/// `attack_count` below which the phase tree still attacks; at the limit it
/// circles the target and starts counting again.
pub(super) fn attack_limit(phase: BossPhase) -> u8 {
    match phase {
        BossPhase::One => 3,
        BossPhase::Two => 5,
    }
}

/// The `CheckTimerDecorator` nodes: a timed branch runs at most once per
/// period, and the first time at once (`first_no_exec=false`).
#[derive(Clone, Copy)]
enum Timer {
    /// P2 normal choice: soul draw (29/30).
    Xihun,
    /// P2 below 75% health: the finisher (60), shared with combo tag 652.
    Ca,
    /// P2 combo tag 652.
    Suohun,
    /// P1 combo tag 640.
    Touji,
}

impl Timer {
    const PERIODS: [f32; 4] = [40.0, 70.0, 65.0, 90.0];
}

/// `PassNode` branches, which run only the first time they are reached.
#[derive(Clone, Copy)]
enum Once {
    Opening,
    P2Combo3,
    P2Combo13,
    P2Combo19,
}

/// `Probability` nodes with `different_with_last=true`.
#[derive(Clone, Copy)]
enum Alternate {
    Gedang,
    P1Combo28,
}

/// The encounter's blackboard: what the trees read and write between ticks.
pub(super) struct Mind {
    pub attack_count: u8,
    cooldowns: [f32; 61],
    /// Seconds since each timed branch last ran; `None` before the first.
    timers: [Option<f32>; 4],
    passed: [bool; 4],
    last: [Option<usize>; 2],
    /// P2's finisher is owed after a skill which cannot cancel into it now.
    is_ca: bool,
    /// The direction of the last circling move (`lastlockdirection`).
    pub last_lock_left: bool,
    random: u32,
}

impl Default for Mind {
    fn default() -> Self {
        Self {
            attack_count: 0,
            cooldowns: [0.0; 61],
            timers: [None; 4],
            passed: [false; 4],
            last: [None; 2],
            is_ca: false,
            last_lock_left: false,
            random: 0xc10c_51a1,
        }
    }
}

/// What a decision reads of the world.
#[derive(Clone, Copy)]
pub(super) struct Sight {
    pub phase: BossPhase,
    /// Feet to feet, in source units (metres; no hidden model scaling).
    pub distance: f32,
    /// Phase one's mark on its target at 65% health (buff 450097).
    pub marked: bool,
    /// The current phase's health share.
    pub health: f32,
}

/// What the phase tree does next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Plan {
    Act(u16),
    /// Circle the target for `seconds`, then do `then`. `turn`: first turn
    /// to face a target behind (the round-end circling checks, the opening
    /// circling does not).
    Circle {
        seconds: f32,
        then: Option<u16>,
        turn: bool,
    },
}

impl Plan {
    /// Whether this plan begins with the source's `JudgeTargetInRangeAction`
    /// and `TurningAction`: the dispatch tree's movement actions 1, 2 and 3
    /// and the phase trees' round-end circling. Skills do not.
    pub fn turns_first(self) -> bool {
        match self {
            Plan::Act(action) => (1..=3).contains(&action),
            Plan::Circle { turn, .. } => turn,
        }
    }
}

impl Mind {
    pub fn tick(&mut self, dt: f32) {
        for cooldown in &mut self.cooldowns {
            *cooldown = (*cooldown - dt).max(0.0);
        }
        for timer in self.timers.iter_mut().flatten() {
            *timer += dt;
        }
    }

    pub fn roll(&mut self, maximum: u32) -> u32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        self.random % maximum
    }

    /// A uniform time in `[low, high]`, for the source's random durations.
    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.roll(1001) as f32 / 1000.0
    }

    /// Seconds until `action`'s weight returns.
    #[cfg(test)]
    pub fn cooldown(&self, action: u16) -> f32 {
        self.cooldowns[usize::from(action)]
    }

    pub fn is_cooling(&self, action: u16) -> bool {
        self.cooldowns[usize::from(action)] > 0.0
    }

    /// The dispatch tree's bookkeeping when `action` starts.
    pub fn dispatch(&mut self, action: u16, marked: bool) {
        let cooldown = cooldown(action) * COOLDOWN_SCALE;
        if cooldown > 0.0 {
            self.cooldowns[usize::from(action)] = cooldown;
        }
        if counts_as_attack(action, marked) {
            self.attack_count = self.attack_count.saturating_add(1);
        }
    }

    /// `ApplyActionByCondition`: a weighted choice among the actions whose
    /// weight is not zeroed by a cooldown; `None` when all are.
    fn pick(&mut self, choices: &[(u16, u32)]) -> Option<u16> {
        let total: u32 = choices
            .iter()
            .filter(|(action, _)| !self.is_cooling(*action))
            .map(|(_, weight)| weight)
            .sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.roll(total);
        for &(action, weight) in choices {
            if self.is_cooling(action) {
                continue;
            }
            if roll < weight {
                return Some(action);
            }
            roll -= weight;
        }
        unreachable!("weighted choice exhausted")
    }

    fn timer(&mut self, timer: Timer) -> bool {
        let index = timer as usize;
        let ready = self.timers[index].is_none_or(|elapsed| elapsed >= Timer::PERIODS[index]);
        if ready {
            self.timers[index] = Some(0.0);
        }
        ready
    }

    fn once(&mut self, once: Once) -> bool {
        !std::mem::replace(&mut self.passed[once as usize], true)
    }

    /// One of `count` equally likely children, never the one chosen last time.
    fn alternate(&mut self, node: Alternate, count: usize) -> usize {
        let last = self.last[node as usize];
        let mut child = self.roll(count as u32) as usize;
        if Some(child) == last {
            child = (child + 1 + self.roll(count as u32 - 1) as usize) % count;
        }
        self.last[node as usize] = Some(child);
        child
    }

    /// The phase tree's attacking branch (`attack_count` below the limit).
    pub fn choose(&mut self, sight: Sight) -> Plan {
        let d = sight.distance;
        match sight.phase {
            BossPhase::One => {
                if self.once(Once::Opening) {
                    return Plan::Circle {
                        seconds: 2.0,
                        then: self.pick(&[(45, 100)]),
                        turn: false,
                    };
                }
                let action = if d > 12.0 {
                    self.pick(&[(3, 100)])
                } else {
                    None
                }
                .or_else(|| {
                    (d > 6.0)
                        .then(|| self.pick(&[(3, 10), (14, 50), (16, 50), (45, 50)]))
                        .flatten()
                })
                .or_else(|| self.pick(&[(1, 1), (10, 50), (12, 50), (11, 50), (19, 1)]));
                Plan::Act(action.unwrap_or(1))
            }
            BossPhase::Two => {
                if self.once(Once::Opening)
                    && let Some(action) = self.pick(&[(21, 100)])
                {
                    return Plan::Act(action);
                }
                if sight.health <= 0.75 && self.timer(Timer::Ca) {
                    if d <= 6.0 {
                        return Plan::Act(60);
                    }
                    // Too far: owe it to the next skill which can cancel (tag 605).
                    self.is_ca = true;
                }
                // The 450100 branch (soul draw 27) needs an achievement buff
                // Kiva has no counterpart for.
                let action = if d > 12.0 {
                    self.pick(&[(3, 100)])
                } else if d > 6.0 {
                    self.soul_draw()
                        .or_else(|| self.pick(&[(48, 50), (49, 50), (24, 50), (3, 10), (21, 50)]))
                } else {
                    self.soul_draw()
                        .or_else(|| self.pick(&[(50, 50), (25, 50), (53, 50)]))
                };
                Plan::Act(action.unwrap_or(1))
            }
        }
    }

    fn soul_draw(&mut self) -> Option<u16> {
        if self.timer(Timer::Xihun) {
            self.pick(&[(29, 50), (30, 50)])
        } else {
            None
        }
    }

    /// `GedangCondition`: the target guarded against the Boss.
    pub fn counter_guard(&mut self) -> u16 {
        [6, 7][self.alternate(Alternate::Gedang, 2)]
    }

    /// The follow-up the combo trees choose when `action` reaches its
    /// cancel point, or `None` when it ends normally.
    pub fn combo(&mut self, action: u16, sight: Sight) -> Option<u16> {
        let tag = slot(action)? % 1000;
        let combo = match sight.phase {
            BossPhase::One => p1_tag(tag, sight.distance, self)?,
            BossPhase::Two => p2_tag(tag, sight.distance, self)?,
        };
        match sight.phase {
            BossPhase::One => self.p1_combo(combo, sight),
            BossPhase::Two => self.p2_combo(combo, sight.distance),
        }
    }

    /// `51000025_BOSS01_fight_P1`, `CBTSwitchCaseNode` on `BOSS01_Combo`.
    fn p1_combo(&mut self, combo: u8, sight: Sight) -> Option<u16> {
        let (d, marked) = (sight.distance, sight.marked);
        let near = |reach: f32| d <= reach;
        let wide = |mind: &mut Self| mind.pick(&[(16, 50), (45, 50)]);
        match combo {
            1 => (near(7.0) && marked)
                .then(|| self.pick(&[(46, 90), (44, 15), (40, 15), (41, 15)]))
                .flatten()
                .or_else(|| {
                    near(7.0)
                        .then(|| self.pick(&[(41, 70), (44, 15), (40, 15)]))
                        .flatten()
                })
                .or_else(|| wide(self)),
            2 => self.pick(&[(45, 50), (3, 10), (16, 50), (14, 50)]),
            3 | 16 => self.pick(&[(17, 100)]),
            4 => self.pick(&[(45, 50), (3, 10), (14, 50), (16, 50)]),
            // `behit_gold` (hit while gold-bodied); Kiva raises no such hit.
            5 => Some(34),
            6 => Some(9),
            7 => Some(8),
            8 => Some(7),
            9 => Some(6),
            10 => Some(4),
            11 => if marked {
                self.pick(&[(38, 50), (39, 50), (41, 25), (44, 25)])
            } else {
                None
            }
            .or_else(|| self.pick(&[(38, 50), (39, 50), (19, 25), (18, 25)])),
            12 => if marked {
                self.pick(&[(13, 50), (37, 50), (41, 25), (44, 25)])
            } else {
                None
            }
            .or_else(|| self.pick(&[(13, 50), (37, 50), (19, 25), (17, 25)])),
            13 => self.pick(&[(20, 100)]),
            14 => near(6.0)
                .then(|| self.pick(&[(43, 100)]))
                .flatten()
                .or(Some(16)),
            15 => self.pick(&[(44, 100)]),
            17 => (near(7.0) && marked)
                .then(|| self.pick(&[(46, 90), (44, 15), (41, 15)]))
                .flatten()
                .or_else(|| {
                    near(7.0)
                        .then(|| self.pick(&[(44, 50), (41, 50)]))
                        .flatten()
                })
                .or_else(|| wide(self)),
            18 => self.pick(&[(45, 30), (18, 70)]),
            19 => self.pick(&[(13, 100)]),
            20 => (d < 6.0)
                .then(|| self.pick(&[(15, 100)]))
                .flatten()
                .or_else(|| wide(self)),
            21 => self.pick(&[(20, 50), (18, 50)]),
            22 => if marked {
                self.pick(&[(46, 90), (17, 15), (41, 15)])
            } else {
                None
            }
            .or_else(|| self.pick(&[(41, 70), (17, 30)])),
            23 => if marked {
                self.pick(&[(46, 90), (40, 15), (41, 15)])
            } else {
                None
            }
            .or_else(|| self.pick(&[(44, 50), (41, 25)])),
            24 => if marked {
                self.pick(&[(46, 90), (40, 15), (41, 15)])
            } else {
                None
            }
            .or_else(|| self.pick(&[(41, 50), (40, 50)])),
            25 => wide(self),
            26 => self.pick(&[(20, 50), (39, 50)]),
            27 => self.pick(&[(39, 50), (20, 50)]),
            28 => {
                if self.alternate(Alternate::P1Combo28, 2) == 0 {
                    Some(20)
                } else {
                    self.pick(&[(18, 50), (45, 50)])
                }
            }
            29 => self.pick(&[(20, 50), (42, 50)]),
            30 => near(6.0)
                .then(|| self.pick(&[(20, 50), (17, 50)]))
                .flatten()
                .or_else(|| wide(self)),
            _ => None,
        }
    }

    /// `51000031_BOSS01_fight_P2`, `CBTSwitchCaseNode` on `BOSS01_Combo`.
    fn p2_combo(&mut self, combo: u8, d: f32) -> Option<u16> {
        let wide = |mind: &mut Self| mind.pick(&[(48, 50), (49, 50), (24, 50)]);
        let narrow = |mind: &mut Self| mind.pick(&[(48, 25), (49, 50), (24, 50)]);
        match combo {
            1 => Some(22),
            2 => Some(33),
            3 => {
                if self.once(Once::P2Combo3) {
                    Some(28)
                } else {
                    self.pick(&[(56, 50), (53, 50), (51, 15)])
                }
            }
            4 | 14 => narrow(self),
            5 | 16 => wide(self),
            6 => (d <= 6.0)
                .then(|| self.pick(&[(56, 50), (58, 50)]))
                .flatten()
                .or_else(|| narrow(self)),
            7 => Some(28),
            8 => Some(55),
            9 => self.pick(&[(23, 99), (54, 1)]),
            10 | 11 => self.pick(&[(57, 50), (54, 50)]),
            12 => self.pick(&[(52, 50), (31, 50)]),
            13 => {
                if self.once(Once::P2Combo13) {
                    Some(28)
                } else {
                    self.pick(&[(53, 50), (58, 50), (51, 15)])
                }
            }
            15 => Some(16),
            17 => self.pick(&[(59, 50), (48, 50)]),
            18 => self.pick(&[(55, 50), (31, 50)]),
            19 => {
                if self.once(Once::P2Combo19) {
                    Some(28)
                } else {
                    Some(59)
                }
            }
            20 => self.pick(&[(57, 100)]),
            21 => Some(60),
            22 => (d <= 6.0)
                .then(|| self.pick(&[(57, 100)]))
                .flatten()
                .or(Some(48)),
            _ => None,
        }
    }
}

/// `51000025_BOSS01_P1_ComboCondition`: the skill's tag (its slot's last
/// three digits) and the distance set `BOSS01_Combo`. Tags 601, 628 and 99
/// end without a combo; the lock / dodge / 80 / 81 tags come from events
/// Kiva does not raise.
fn p1_tag(tag: u32, d: f32, mind: &mut Mind) -> Option<u8> {
    let within = |reach: f32, combo: u8| (d <= reach).then_some(combo);
    match tag {
        593 => within(7.0, 21),
        594 => within(7.0, 22),
        595 => within(7.0, 24),
        596 => Some(24),
        599 => within(7.0, 23),
        602 => (d >= 5.0).then_some(25),
        626 => Some(if d <= 6.0 { 11 } else { 2 }),
        627 => Some(if d <= 6.0 { 12 } else { 4 }),
        629 => Some(20),
        635 => within(9.0, 17),
        636 => within(7.0, 3),
        637 => within(7.0, 1),
        638 => within(7.0, 26),
        639 => within(9.0, 28),
        640 => (d <= 12.0 && mind.timer(Timer::Touji)).then_some(15),
        641 => within(8.0, 14),
        _ => None,
    }
}

/// `51000031_BOSS01_P2_ComboCondition`, as [`p1_tag`]. Tags 615 and 653 end
/// without a combo.
fn p2_tag(tag: u32, d: f32, mind: &mut Mind) -> Option<u8> {
    let within = |reach: f32, combo: u8| (d <= reach).then_some(combo);
    match tag {
        604 => within(15.0, 1),
        605 => std::mem::take(&mut mind.is_ca).then_some(21),
        607 => within(8.0, 20),
        608 => within(8.0, 19),
        616 => within(7.0, 6),
        626 => Some(if d <= 6.0 { 3 } else { 4 }),
        627 => Some(if d <= 6.0 { 13 } else { 14 }),
        629 => Some(if d <= 6.0 { 2 } else { 5 }),
        647 => Some(9),
        648 => within(7.0, 8),
        650 => Some(if d <= 5.0 { 15 } else { 16 }),
        652 => {
            if mind.timer(Timer::Suohun) {
                Some(7)
            } else if mind.timer(Timer::Ca) {
                Some(21)
            } else {
                None
            }
        }
        655 => within(7.0, 10),
        656 => within(7.0, 12),
        657 => within(7.0, 18),
        658 => Some(22),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sight(phase: BossPhase, distance: f32) -> Sight {
        Sight {
            phase,
            distance,
            marked: false,
            health: 1.0,
        }
    }

    fn opened(phase: BossPhase) -> Mind {
        let mut mind = Mind::default();
        mind.choose(sight(phase, 3.0));
        mind
    }

    #[test]
    fn phase_one_opens_by_circling_into_45_and_phase_two_with_21() {
        let mut mind = Mind::default();
        assert_eq!(
            mind.choose(sight(BossPhase::One, 3.0)),
            Plan::Circle {
                seconds: 2.0,
                then: Some(45),
                turn: false,
            }
        );
        assert_ne!(
            mind.choose(sight(BossPhase::One, 3.0)),
            Plan::Circle {
                seconds: 2.0,
                then: Some(45),
                turn: false,
            }
        );
        assert_eq!(
            Mind::default().choose(sight(BossPhase::Two, 3.0)),
            Plan::Act(21)
        );
    }

    #[test]
    fn normal_choices_follow_the_source_distance_pools() {
        let mut mind = opened(BossPhase::One);
        assert_eq!(mind.choose(sight(BossPhase::One, 12.5)), Plan::Act(3));
        for _ in 0..500 {
            let Plan::Act(mid) = mind.choose(sight(BossPhase::One, 9.0)) else {
                panic!()
            };
            assert!([3, 14, 16, 45].contains(&mid), "{mid}");
            let Plan::Act(near) = mind.choose(sight(BossPhase::One, 6.0)) else {
                panic!()
            };
            assert!([1, 10, 11, 12, 19].contains(&near), "{near}");
        }
        let mut mind = opened(BossPhase::Two);
        // The soul draw runs once per 40 s, the first time at once.
        assert!(matches!(
            mind.choose(sight(BossPhase::Two, 5.0)),
            Plan::Act(29 | 30)
        ));
        for _ in 0..500 {
            let Plan::Act(near) = mind.choose(sight(BossPhase::Two, 5.0)) else {
                panic!()
            };
            assert!([50, 25, 53].contains(&near), "{near}");
        }
    }

    #[test]
    fn dispatch_counts_attacks_and_zeroes_weights_for_the_source_cooldown() {
        let mut mind = opened(BossPhase::One);
        mind.dispatch(10, false);
        mind.dispatch(16, false);
        assert_eq!(mind.attack_count, 1, "16 is not counted");
        assert_eq!(mind.cooldown(10), 40.0 * COOLDOWN_SCALE);
        // Once the target is marked, 45 no longer counts.
        mind.dispatch(45, true);
        assert_eq!(mind.attack_count, 1);
        for action in [10, 11, 12, 19] {
            mind.dispatch(action, false);
        }
        // Every near choice cools down: only the circling move remains.
        for _ in 0..50 {
            assert_eq!(mind.choose(sight(BossPhase::One, 3.0)), Plan::Act(1));
        }
        mind.tick(85.0 * COOLDOWN_SCALE);
        assert_eq!(mind.cooldown(10), 0.0);
        assert_eq!(attack_limit(BossPhase::One), 3);
        assert_eq!(attack_limit(BossPhase::Two), 5);
    }

    #[test]
    fn combo_tags_pick_follow_ups_by_distance() {
        let mut mind = opened(BossPhase::One);
        // 45 -> tag 641: within 8 m combo 14, which is 43 within 6 m, else 16.
        assert_eq!(mind.combo(45, sight(BossPhase::One, 5.0)), Some(43));
        assert_eq!(mind.combo(45, sight(BossPhase::One, 7.5)), Some(16));
        assert_eq!(mind.combo(45, sight(BossPhase::One, 9.0)), None);
        // 18 -> tag 601 never combos.
        assert_eq!(mind.combo(18, sight(BossPhase::One, 3.0)), None);
        let mut mind = opened(BossPhase::Two);
        // 21 -> tag 604: within 15 m straight into 22.
        assert_eq!(mind.combo(21, sight(BossPhase::Two, 14.0)), Some(22));
        // 25 -> tag 608 -> combo 19: 28 the first time only, then 59.
        assert_eq!(mind.combo(25, sight(BossPhase::Two, 3.0)), Some(28));
        assert_eq!(mind.combo(25, sight(BossPhase::Two, 3.0)), Some(59));
    }

    #[test]
    fn phase_two_finisher_runs_once_per_seventy_seconds_below_three_quarters() {
        let low = |distance| Sight {
            health: 0.7,
            ..sight(BossPhase::Two, distance)
        };
        let mut mind = opened(BossPhase::Two);
        assert_eq!(mind.choose(low(5.0)), Plan::Act(60));
        assert_ne!(mind.choose(low(5.0)), Plan::Act(60));
        mind.tick(70.0);
        // Too far: owed to the next skill which can cancel into it (22's 605).
        assert_ne!(mind.choose(low(9.0)), Plan::Act(60));
        assert_eq!(mind.combo(22, low(9.0)), Some(60));
        assert_eq!(mind.combo(22, low(9.0)), None);
    }

    #[test]
    fn the_guard_counter_never_repeats_itself() {
        let mut mind = Mind::default();
        let mut last = mind.counter_guard();
        for _ in 0..20 {
            let next = mind.counter_guard();
            assert!([6, 7].contains(&next));
            assert_ne!(next, last);
            last = next;
        }
    }
}
