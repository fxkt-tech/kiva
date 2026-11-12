//! Concrete first-version weapon pairs. Grip mode belongs to attachment geometry.
use super::MeleeStage;

/// Kiva's authoritative ordinary-attack numbers; presentation only reads
/// them. Where and when the blade hits comes from the stage's baked path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeProfile {
    pub damage: f32,
    pub duration: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponPair {
    #[default]
    HammerMusket,
    ChainAxeBow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponMode {
    #[default]
    Melee,
    Ranged,
}
impl WeaponPair {
    pub const ALL: [Self; 2] = [Self::HammerMusket, Self::ChainAxeBow];

    /// Position in [`Self::ALL`], the order the pairs cycle in.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|pair| *pair == self)
            .expect("every pair is listed")
    }
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn melee_profile(self, stage: MeleeStage) -> MeleeProfile {
        // The three stages deliberately retain the existing ordinary-attack
        // numbers. Stage three has no extra damage, armor or guard break.
        let profiles = match self {
            Self::HammerMusket => {
                [MeleeProfile {
                    damage: 24.0,
                    duration: 0.60,
                }; 3]
            }
            Self::ChainAxeBow => {
                [MeleeProfile {
                    damage: 18.0,
                    duration: 0.40,
                }; 3]
            }
        };
        profiles[match stage {
            MeleeStage::One => 0,
            MeleeStage::Two => 1,
            MeleeStage::Three => 2,
        }]
    }
}
