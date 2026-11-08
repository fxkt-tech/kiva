//! Client-only identities and authored clip names; no combat rules live here.
use crate::{gameplay::UnitRole, simulation::*};

pub(super) struct Profile {
    pub path: &'static str,
    pub name: &'static str,
    pub skin: &'static str,
    pub scale: f32,
    pub idle: &'static str,
    pub run: &'static str,
    pub attack: &'static str,
    pub death: &'static str,
    pub skills: &'static [(AbilityId, &'static str)],
}

pub(super) const PROFILES: [Profile; 6] = [
    Profile {
        path: "models/league/jarvan-base.glb",
        name: "Jarvan IV",
        skin: "Original",
        scale: 1.1,
        idle: "Idle1",
        run: "Run",
        attack: "Attack1",
        death: "Death",
        skills: &[
            (AbilityId(11), "Spell1"),
            (AbilityId(12), "Spell2"),
            (AbilityId(13), "Spell3"),
            (AbilityId(14), "Spell4"),
        ],
    },
    Profile {
        path: "models/league/ashe-project.glb",
        name: "Ashe",
        skin: "PROJECT",
        scale: 1.2,
        idle: "Ashe_Skin08_Idle1.anm",
        run: "Run_Normal",
        attack: "Ashe_Skin08_Attack1.anm",
        death: "Death",
        skills: &[
            (AbilityId(21), "Ashe_Skin08_Spell1.anm"),
            (AbilityId(22), "Ashe_Skin08_Spell2.anm"),
            (AbilityId(23), "Spell3"),
            (AbilityId(24), "Ashe_Skin08_Spell4.anm"),
        ],
    },
    minion(
        "models/league/ha_orderminionmelee.glb",
        "Melee minion",
        "Order",
        1.0,
    ),
    minion(
        "models/league/ha_chaosminionmelee.glb",
        "Melee minion",
        "Chaos",
        1.0,
    ),
    minion(
        "models/league/ha_orderminionranged.glb",
        "Ranged minion",
        "Order",
        1.1,
    ),
    minion(
        "models/league/ha_chaosminionranged.glb",
        "Ranged minion",
        "Chaos",
        1.1,
    ),
];

const fn minion(path: &'static str, name: &'static str, skin: &'static str, scale: f32) -> Profile {
    Profile {
        path,
        name,
        skin,
        scale,
        idle: "Idle1",
        run: "Run",
        attack: "Attack1",
        death: "Death",
        skills: &[],
    }
}

impl Profile {
    pub fn skill(&self, ability: AbilityId) -> Option<&'static str> {
        self.skills
            .iter()
            .find_map(|(id, clip)| (*id == ability).then_some(*clip))
    }

    pub fn clips(&self) -> impl Iterator<Item = &'static str> + '_ {
        [self.idle, self.run, self.attack, self.death]
            .into_iter()
            .chain(self.skills.iter().map(|(_, clip)| *clip))
    }
}

pub(super) fn profile(
    role: Option<UnitRole>,
    identity: Option<CharacterId>,
    team: Team,
) -> Option<usize> {
    match role {
        Some(UnitRole::Hero) => match identity {
            Some(CharacterId(1)) => Some(0),
            Some(CharacterId(2)) => Some(1),
            _ => None,
        },
        Some(UnitRole::MeleeMinion) => Some(if team == Team::Ally { 2 } else { 3 }),
        Some(UnitRole::RangedMinion) => Some(if team == Team::Ally { 4 } else { 5 }),
        _ => None,
    }
}

pub(in crate::client) fn ability_icon(id: AbilityId) -> Option<&'static str> {
    Some(match id.0 {
        11 => "assets/icons/league/jarvan-q.png",
        12 => "assets/icons/league/jarvan-w.png",
        13 => "assets/icons/league/jarvan-e.png",
        14 => "assets/icons/league/jarvan-r.png",
        21 => "assets/icons/league/ashe-q.png",
        22 => "assets/icons/league/ashe-w.png",
        23 => "assets/icons/league/ashe-e.png",
        24 => "assets/icons/league/ashe-r.png",
        _ => return None,
    })
}

pub(in crate::client) fn portrait(id: CharacterId) -> Option<&'static str> {
    match id.0 {
        1 => Some("assets/icons/league/jarvan-base.jpg"),
        2 => Some("assets/icons/league/ashe-project.jpg"),
        _ => None,
    }
}
