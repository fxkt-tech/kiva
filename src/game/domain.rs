#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Camp {
    Good,
    Werewolf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Werewolf,
    Seer,
    Witch,
    Hunter,
    Villager,
}

impl Role {
    pub fn camp(self) -> Camp {
        match self {
            Role::Werewolf => Camp::Werewolf,
            Role::Seer | Role::Witch | Role::Hunter | Role::Villager => Camp::Good,
        }
    }

    pub fn nine_player_deck() -> Vec<Role> {
        vec![
            Role::Werewolf,
            Role::Werewolf,
            Role::Werewolf,
            Role::Seer,
            Role::Witch,
            Role::Hunter,
            Role::Villager,
            Role::Villager,
            Role::Villager,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Role::Werewolf => "狼人",
            Role::Seer => "预言家",
            Role::Witch => "女巫",
            Role::Hunter => "猎人",
            Role::Villager => "村民",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerKind {
    Ai,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerProfile {
    pub name: String,
    pub personality_preference: String,
    pub avatar: String,
    pub ai: PlayerAiConfig,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub personality_preference: String,
    pub avatar: String,
    pub ai: PlayerAiConfig,
    pub role: Role,
    pub kind: PlayerKind,
    pub alive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_player_deck_has_expected_roles() {
        let roles = Role::nine_player_deck();
        assert_eq!(roles.len(), 9);
        assert_eq!(
            roles.iter().filter(|role| **role == Role::Werewolf).count(),
            3
        );
        assert_eq!(roles.iter().filter(|role| **role == Role::Seer).count(), 1);
        assert_eq!(roles.iter().filter(|role| **role == Role::Witch).count(), 1);
        assert_eq!(
            roles.iter().filter(|role| **role == Role::Hunter).count(),
            1
        );
        assert_eq!(
            roles.iter().filter(|role| **role == Role::Villager).count(),
            3
        );
    }
}
