//! Immutable character content resolves into the existing unit construction path.
use super::{
    AbilityCatalog, AbilityId, AbilityLoadout, EffectiveAttributes, Team, UnitSpec, UnitStats,
    abilities::validate_loadout, attributes, spawn_unit,
};
use bevy::prelude::*;
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharacterId(pub u16);

#[derive(Clone, Debug)]
pub struct CharacterDefinition {
    pub name: String,
    pub description: String,
    pub stats: UnitStats,
    pub default_loadout: AbilityLoadout,
}
impl CharacterDefinition {
    /// Initial effective attributes, including technical attack-speed limits.
    /// This projection does not create a unit or allocate modifier sources.
    pub fn initial_attributes(&self) -> Option<EffectiveAttributes> {
        self.stats
            .valid()
            .then(|| attributes::initial_attributes(self.stats.base_attributes))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadoutSelection {
    #[default]
    CharacterDefault,
    /// Replaces all four slots, including empty slots and repeated ability IDs.
    Explicit(AbilityLoadout),
}

#[derive(Clone, Debug)]
pub struct CharacterSpawnSpec {
    pub character: CharacterId,
    pub team: Team,
    pub position: Vec2,
    pub loadout: LoadoutSelection,
    pub respawn_delay: Option<f32>,
}

/// Character provenance only. Actual loadout and state remain on the unit.
#[derive(Component, Debug)]
pub struct CharacterIdentity {
    character_id: CharacterId,
}
impl CharacterIdentity {
    pub fn id(&self) -> CharacterId {
        self.character_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharacterError {
    DuplicateId(CharacterId),
    InvalidDefinition,
    UnknownCharacter(CharacterId),
    UnknownAbility { slot: usize, ability: AbilityId },
    InvalidSpawn { reason: &'static str },
}
impl fmt::Display for CharacterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(formatter, "duplicate character id {}", id.0),
            Self::InvalidDefinition => formatter.write_str("invalid character definition"),
            Self::UnknownCharacter(id) => write!(formatter, "unknown character id {}", id.0),
            Self::UnknownAbility { slot, ability } => {
                write!(formatter, "unknown ability {} in slot {slot}", ability.0)
            }
            Self::InvalidSpawn { reason } => formatter.write_str(reason),
        }
    }
}
impl std::error::Error for CharacterError {}

#[derive(Resource, Default)]
pub struct CharacterCatalog(BTreeMap<CharacterId, CharacterDefinition>);
impl CharacterCatalog {
    pub fn register(
        &mut self,
        id: CharacterId,
        definition: CharacterDefinition,
        abilities: &AbilityCatalog,
    ) -> Result<(), CharacterError> {
        if self.0.contains_key(&id) {
            return Err(CharacterError::DuplicateId(id));
        }
        if definition.name.trim().is_empty() || !definition.stats.valid() {
            return Err(CharacterError::InvalidDefinition);
        }
        validate_loadout(&definition.default_loadout, abilities)
            .map_err(|(slot, ability)| CharacterError::UnknownAbility { slot, ability })?;
        self.0.insert(id, definition);
        Ok(())
    }

    pub fn get(&self, id: CharacterId) -> Option<&CharacterDefinition> {
        self.0.get(&id)
    }

    /// Read-only entries in ascending character-ID order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (CharacterId, &CharacterDefinition)> + '_ {
        self.0.iter().map(|(&id, definition)| (id, definition))
    }
}

/// Resolves immutable content and a complete loadout before creating any state.
pub fn spawn_character(
    world: &mut World,
    spec: CharacterSpawnSpec,
) -> Result<Entity, CharacterError> {
    let definition = world
        .resource::<CharacterCatalog>()
        .get(spec.character)
        .ok_or(CharacterError::UnknownCharacter(spec.character))?;
    let abilities = match spec.loadout {
        LoadoutSelection::CharacterDefault => definition.default_loadout,
        LoadoutSelection::Explicit(loadout) => loadout,
    };
    validate_loadout(&abilities, world.resource::<AbilityCatalog>())
        .map_err(|(slot, ability)| CharacterError::UnknownAbility { slot, ability })?;
    let unit_spec = UnitSpec {
        name: definition.name.clone(),
        team: spec.team,
        position: spec.position,
        stats: definition.stats.clone(),
        abilities,
        respawn_delay: spec.respawn_delay,
    };
    let entity =
        spawn_unit(world, unit_spec).map_err(|reason| CharacterError::InvalidSpawn { reason })?;
    world.entity_mut(entity).insert(CharacterIdentity {
        character_id: spec.character,
    });
    Ok(entity)
}
