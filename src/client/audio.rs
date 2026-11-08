//! Read-only battle facts become bounded transient presentation cues.
//! Capture after every complete simulation step, before another life or cleanup.
use super::{camera::CameraRig, session::ClientSession};
use crate::{
    gameplay::{GameplaySet, MatchId},
    simulation::*,
};
use bevy::{ecs::message::MessageCursor, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_PENDING: usize = 48;
const MAX_AGE_TICKS: u64 = 15;
const HEARING_DISTANCE: f32 = 30.0;

pub(super) struct CombatAudioPlugin;

impl Plugin for CombatAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioRuntime>()
            .add_systems(
                FixedUpdate,
                capture
                    .after(SimulationSet::Step)
                    .before(GameplaySet::Outcome),
            )
            .add_systems(PostUpdate, publish.after(super::camera::update));
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CueDefinition {
    files: Vec<String>,
    volume: f32,
    priority: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroCues {
    attack: String,
    empowered_attack: Option<String>,
    death: String,
    respawn: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AudioCatalog {
    cues: BTreeMap<String, CueDefinition>,
    #[serde(skip_serializing)]
    heroes: BTreeMap<u16, HeroCues>,
    #[serde(skip_serializing)]
    abilities: BTreeMap<u16, String>,
}

impl AudioCatalog {
    fn builtin() -> Self {
        let catalog: Self = serde_json::from_str(include_str!("../../assets/audio/catalog.json"))
            .expect("valid checked-in audio catalog");
        assert!(catalog.cues.values().all(|cue| {
            !cue.files.is_empty()
                && cue.files.iter().all(|file| {
                    file.ends_with(".ogg") && !file.contains(['/', '\\']) && !file.contains("..")
                })
                && cue.volume.is_finite()
                && (0.0..=1.0).contains(&cue.volume)
                && cue.priority <= 3
        }));
        assert!(
            catalog
                .abilities
                .values()
                .all(|id| catalog.cues.contains_key(id))
        );
        assert!(catalog.heroes.values().all(|hero| {
            [&hero.attack, &hero.death, &hero.respawn]
                .into_iter()
                .all(|id| catalog.cues.contains_key(id))
                && hero
                    .empowered_attack
                    .as_ref()
                    .is_none_or(|id| catalog.cues.contains_key(id))
        }));
        catalog
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AudioSession {
    player: Entity,
    match_id: MatchId,
}

impl AudioSession {
    fn current(world: &World) -> Option<Self> {
        match world.get_resource::<ClientSession>()? {
            ClientSession::Active {
                player, match_id, ..
            } if world.get::<Unit>(*player).is_some() => Some(Self {
                player: *player,
                match_id: *match_id,
            }),
            _ => None,
        }
    }

    fn token(self) -> String {
        format!("{}:{}", self.match_id, self.player.to_bits())
    }
}

#[derive(Clone, Debug, Serialize)]
struct AudioCue {
    sequence: u64,
    tick: u64,
    source: String,
    life: String,
    cue: String,
    variant: usize,
    gain: f32,
    pan: f32,
    priority: u8,
    #[serde(skip_serializing)]
    position: Vec2,
    #[serde(skip_serializing)]
    personal: bool,
}

#[derive(Serialize)]
struct AudioFrame<'a> {
    frame: u64,
    session: Option<String>,
    paused: bool,
    tick: u64,
    catalog: Option<&'a AudioCatalog>,
    cues: Vec<AudioCue>,
}

#[derive(Resource)]
struct AudioRuntime {
    catalog: AudioCatalog,
    reader: MessageCursor<SimulationEvent>,
    session: Option<AudioSession>,
    pending: Vec<AudioCue>,
    sequence: u64,
    random: u64,
    variants: BTreeMap<String, usize>,
    published_catalog: bool,
    frame: u64,
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self {
            catalog: AudioCatalog::builtin(),
            reader: default(),
            session: None,
            pending: Vec::new(),
            sequence: 0,
            random: 0x4b49_5641_a8cd_9187,
            variants: default(),
            published_catalog: false,
            frame: 0,
        }
    }
}

enum CueTrigger {
    Attack,
    Ability(AbilityId),
    Death,
    Respawn,
}

impl AudioRuntime {
    fn sync_session(&mut self, current: Option<AudioSession>) {
        if self.session != current {
            self.session = current;
            self.pending.clear();
            self.variants.clear();
            self.sequence = 0;
        }
    }

    fn variant(&mut self, key: &str, count: usize) -> usize {
        // Cosmetic PRNG only; it cannot affect simulation or consume its state.
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        let mut index = (self.random % count as u64) as usize;
        if count > 1 && self.variants.get(key) == Some(&index) {
            index = (index + 1) % count;
        }
        self.variants.insert(key.to_owned(), index);
        index
    }

    fn push(&mut self, cue: AudioCue) {
        if self.pending.len() == MAX_PENDING {
            let (index, weakest) = self
                .pending
                .iter()
                .enumerate()
                .min_by_key(|(_, cue)| cue.priority)
                .expect("bounded nonempty queue");
            if weakest.priority > cue.priority {
                return;
            }
            self.pending.remove(index);
        }
        self.pending.push(cue);
    }

    fn drain(&mut self, tick: u64, paused: bool, listener: Vec2) -> Vec<AudioCue> {
        if paused {
            self.pending.clear();
            return Vec::new();
        }
        self.pending
            .drain(..)
            .filter_map(|mut cue| {
                if tick.saturating_sub(cue.tick) > MAX_AGE_TICKS {
                    return None;
                }
                if !cue.personal {
                    let distance = cue.position.distance(listener);
                    if distance >= HEARING_DISTANCE {
                        return None;
                    }
                    cue.gain *= (1.0 - distance / HEARING_DISTANCE).powi(2);
                    // Camera is rotated over the x/z plane. Its visual right axis
                    // is (2,-1), so a source on screen right sounds on the right.
                    cue.pan = ((cue.position - listener).dot(Vec2::new(2.0, -1.0).normalize())
                        / 14.0)
                        .clamp(-0.8, 0.8);
                }
                Some(cue)
            })
            .collect()
    }
}

fn capture(world: &mut World) {
    let current = AudioSession::current(world);
    let tick = world.resource::<SimulationTick>().0;
    let paused = world.resource::<SimulationControl>().paused;
    world.resource_scope(|world, mut runtime: Mut<AudioRuntime>| {
        runtime.sync_session(current);
        let events: Vec<_> = runtime
            .reader
            .read(world.resource::<Messages<SimulationEvent>>())
            .cloned()
            .collect();
        let Some(session) = current.filter(|_| !paused) else {
            runtime.pending.clear();
            return;
        };
        let team = world
            .get::<Unit>(session.player)
            .expect("current player")
            .team();
        for event in events {
            let (source, trigger) = match event {
                SimulationEvent::AttackReleased { source, .. } => (source, CueTrigger::Attack),
                SimulationEvent::AbilityCommitted {
                    unit,
                    ability,
                    recast: false,
                    ..
                } => (unit, CueTrigger::Ability(ability)),
                SimulationEvent::Died { unit } => (unit, CueTrigger::Death),
                SimulationEvent::Respawned { unit } => (unit, CueTrigger::Respawn),
                _ => continue,
            };
            let Some(identity) = world.get::<CharacterIdentity>(source) else {
                continue;
            };
            let Some(pose) = world.get::<UnitPose>(source) else {
                continue;
            };
            let Some(state) = world.get::<UnitState>(source) else {
                continue;
            };
            if !team_can_see(world, team, source) {
                continue;
            }
            let Some(hero) = runtime.catalog.heroes.get(&identity.id().0) else {
                continue;
            };
            let personal = source == session.player
                && matches!(trigger, CueTrigger::Death | CueTrigger::Respawn);
            let key = match trigger {
                CueTrigger::Attack => hero
                    .empowered_attack
                    .as_ref()
                    .filter(|_| {
                        world
                            .get::<UnitMechanics>(source)
                            .is_some_and(UnitMechanics::empowered)
                    })
                    .or(Some(&hero.attack)),
                CueTrigger::Ability(id) => runtime.catalog.abilities.get(&id.0),
                CueTrigger::Death => Some(&hero.death),
                CueTrigger::Respawn => Some(&hero.respawn),
            };
            let Some(key) = key.cloned() else { continue };
            let definition = &runtime.catalog.cues[&key];
            let (volume, priority, count) = (
                definition.volume,
                definition.priority,
                definition.files.len(),
            );
            let variant = runtime.variant(&key, count);
            runtime.sequence = runtime.sequence.wrapping_add(1);
            let sequence = runtime.sequence;
            runtime.push(AudioCue {
                sequence,
                tick,
                source: source.to_bits().to_string(),
                life: state.life_ref(source).epoch().to_string(),
                cue: key,
                variant,
                gain: volume,
                pan: 0.0,
                priority,
                position: pose.position(),
                personal,
            });
        }
    });
}

fn publish(world: &mut World) {
    let session = AudioSession::current(world);
    let paused = session.is_none() || world.resource::<SimulationControl>().paused;
    let tick = world.resource::<SimulationTick>().0;
    let listener = world
        .get_resource::<CameraRig>()
        .map_or(Vec2::ZERO, |rig| rig.center);
    world.resource_scope(|_, mut runtime: Mut<AudioRuntime>| {
        runtime.sync_session(session);
        runtime.frame = runtime
            .frame
            .checked_add(1)
            .expect("audio display frame space exhausted");
        let cues = runtime.drain(tick, paused, listener);
        let catalog = (!runtime.published_catalog).then_some(&runtime.catalog);
        let packet = AudioFrame {
            frame: runtime.frame,
            session: session.map(AudioSession::token),
            paused,
            tick,
            catalog,
            cues,
        };
        #[cfg(target_arch = "wasm32")]
        dispatch(&packet);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = packet;
        runtime.published_catalog = true;
    });
}

#[cfg(target_arch = "wasm32")]
fn dispatch(packet: &AudioFrame<'_>) {
    if let Some(window) = web_sys::window()
        && let Ok(json) = serde_json::to_string(packet)
    {
        let init = web_sys::CustomEventInit::new();
        init.set_detail(&wasm_bindgen::JsValue::from_str(&json));
        if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("kiva-audio", &init) {
            let _ = window.dispatch_event(&event);
        }
    }
}

#[cfg(test)]
mod tests;
