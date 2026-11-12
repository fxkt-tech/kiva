//! The offline bake of every melee attack's blade path, and the check that
//! the committed data still matches the animation.
//!
//! `make bake-hits` replays each attack through the live display driver at
//! the simulation's tick rate, records where the weapon is at every tick and
//! writes `simulation/combat/hit_tracks.rs`. The rules then sweep that path;
//! they never see a skeleton. Changing a clip, an attack clip row, a weapon
//! model or an attack's duration changes the bake, and the check below fails
//! until the data is baked again.
use super::{
    animation::AttackReplay,
    avatar::{Appearance, AvatarAssets},
    skeleton::AnimationLibrary,
    source_animation::SourceAnimationLibrary,
    stickman::{Attachment, attachment_transform},
    weapon_models::WeaponModel,
};
use crate::simulation::{HitTrack, TrackId, WeaponMode, WeaponPair};
use bevy::prelude::*;
use std::fmt::Write;

/// The blade can hit while its tip moves at least this fast, in m/s.
const ACTIVE_SPEED: f32 = 25.0;
/// Active ticks this close together belong to one window.
const MERGE_GAP: u16 = 1;
/// A shorter burst of speed is a twitch, not a swing.
const MIN_TICKS: u16 = 2;
/// Added to the weapon's own thickness, for the difference between
/// appearances and the display's interpolation.
const MARGIN: f32 = 0.08;
/// The share of the weapon nearest the grip whose guard and ornaments do
/// not count toward its thickness.
const HILT: f32 = 0.25;
/// Frames an attack is held at its start so that the transition into it has
/// finished before its swings are measured.
const SETTLE: u32 = 14;
/// The shown and the settled blade count as the same pose within this distance.
const SAME_POSE: f32 = 0.01;
/// The player is baked with the default appearance, enemies with their own.
const PLAYER: Appearance = Appearance::Lihuowang;
const ENEMY: Appearance = Appearance::Choulingke;

/// A weapon in its grip frame: the grip is the origin.
struct Blade {
    tip: Vec3,
    radius: f32,
}

fn blade(vertices: &[Vec3]) -> Blade {
    let top = vertices.iter().map(|v| v.y).fold(0.0, f32::max);
    let end: Vec<_> = vertices.iter().filter(|v| v.y > top - 0.02).collect();
    let tip = end.iter().fold(Vec3::ZERO, |sum, v| sum + **v) / end.len() as f32;
    let radius = vertices
        .iter()
        .filter_map(|v| {
            let along = v.dot(tip) / tip.length_squared();
            (along >= HILT).then(|| v.distance(tip * along.min(1.0)))
        })
        .fold(0.0, f32::max);
    Blade { tip, radius }
}

struct Baked {
    track: TrackId,
    duration: f32,
    samples: Vec<[f32; 6]>,
    windows: Vec<(u16, u16)>,
    blade_radius: f32,
    reach: f32,
}

/// One actor's attacks in the order they follow each other.
struct Chain {
    tracks: Vec<TrackId>,
    pair: WeaponPair,
    enemy: bool,
    /// Idle frames between two attacks of the chain.
    gap: u32,
}

fn chains() -> Vec<Chain> {
    let mut chains: Vec<Chain> = Vec::new();
    for track in TrackId::all() {
        let (pair, enemy, gap, joins) = match track {
            TrackId::PlayerMelee { pair, .. } => (pair, false, 0, true),
            TrackId::PlayerSkill(kind) => (
                WeaponPair::ALL
                    .into_iter()
                    .find(|pair| pair.melee_skill() == kind)
                    .expect("a blade skill belongs to a weapon pair"),
                false,
                0,
                false,
            ),
            // The Boss returns to idle for two steps between its strikes.
            TrackId::Boss { .. } => (WeaponPair::default(), true, 2, true),
            TrackId::Enemy => (WeaponPair::default(), true, 0, false),
        };
        let same = |a: &TrackId| std::mem::discriminant(a) == std::mem::discriminant(&track);
        match chains.last_mut() {
            Some(chain)
                if joins
                    && chain.pair == pair
                    && chain.enemy == enemy
                    && same(&chain.tracks[0]) =>
            {
                chain.tracks.push(track)
            }
            _ => chains.push(Chain {
                tracks: vec![track],
                pair,
                enemy,
                gap,
            }),
        }
    }
    chains
}

/// The weapon's grip and tip, in the actor's frame, at every tick of each
/// attack of `chain`. `settled` first holds each attack at its start until
/// the transition into it is over.
fn replay(
    chain: &Chain,
    assets: &AvatarAssets,
    blade: &Blade,
    library: &AnimationLibrary,
    sources: &SourceAnimationLibrary,
    settled: bool,
) -> Vec<Vec<[Vec3; 2]>> {
    let attachment = Attachment::Weapon {
        bone: library.bone("hand_r"),
        mode: WeaponMode::Melee,
    };
    let mut replay = AttackReplay::new(library, chain.pair, chain.enemy);
    replay.idle(library, sources, 40);
    chain
        .tracks
        .iter()
        .map(|&track| {
            let duration = track.duration();
            replay.begin();
            for _ in 0..if settled { SETTLE } else { 0 } {
                replay.show(library, sources, track, 0.0, duration);
            }
            let last = HitTrack::tick(duration);
            let path = (0..=last)
                .map(|tick| {
                    // The player's action retires in the step it completes,
                    // before that step is displayed.
                    let rig = if tick == last && !chain.enemy {
                        replay.idle(library, sources, 1);
                        replay.pose()
                    } else {
                        replay.show(library, sources, track, HitTrack::time(tick), duration)
                    };
                    let weapon = attachment_transform(&attachment, &assets.attachment_pose(rig));
                    [weapon.translation, weapon.transform_point(blade.tip)]
                })
                .collect();
            replay.idle(library, sources, chain.gap);
            path
        })
        .collect()
}

/// The first tick at which the displayed blade is the attack's own motion:
/// the transition from whatever came before has finished. Until then the
/// display depends on the previous pose, which no bake can know. It is found
/// by replaying the attack a second time with the transition already over
/// (`settled`) and waiting for the two to agree.
fn own_motion(shown: &[[Vec3; 2]], settled: &[[Vec3; 2]]) -> usize {
    let same = |tick: usize| {
        shown[tick][0].distance(settled[tick][0]) <= SAME_POSE
            && shown[tick][1].distance(settled[tick][1]) <= SAME_POSE
    };
    (0..shown.len() - 2)
        .find(|&tick| (tick..tick + 3).all(same))
        .expect("an attack shows its own motion")
}

/// Inclusive tick ranges within `ticks` in which the tip of `path` moves
/// fast enough to hit.
fn windows(path: &[[Vec3; 2]], ticks: std::ops::RangeInclusive<usize>) -> Vec<(u16, u16)> {
    let speed = |tick: usize| path[tick - 1][1].distance(path[tick][1]) * 60.0;
    let (first, last) = (*ticks.start(), *ticks.end());
    let mut windows: Vec<(u16, u16)> = Vec::new();
    for tick in ticks.clone() {
        if speed(tick) < ACTIVE_SPEED {
            continue;
        }
        let tick = tick as u16;
        match windows.last_mut() {
            Some(window) if tick - window.1 <= MERGE_GAP + 1 => window.1 = tick,
            _ => windows.push((tick, tick)),
        }
    }
    windows.retain(|window| window.1 - window.0 + 1 >= MIN_TICKS);
    if windows.is_empty() {
        // A gentle attack still hits around its fastest moment.
        let peak = ticks
            .max_by(|a, b| speed(*a).total_cmp(&speed(*b)))
            .expect("an attack lasts more than one tick") as u16;
        windows.push((
            peak.saturating_sub(1).max(first as u16),
            (peak + 1).min(last as u16),
        ));
    }
    windows
}

fn bake() -> Vec<Baked> {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<AnimationLibrary>();
    let library = AnimationLibrary::default();
    let sources = SourceAnimationLibrary::bundled();
    let player = AvatarAssets::loaded(&mut world, PLAYER);
    let enemy = AvatarAssets::loaded(&mut world, ENEMY);
    let mut baked = Vec::new();
    for chain in chains() {
        let (assets, weapon) = if chain.enemy {
            let vertices = enemy.blade_vertices(world.resource::<Assets<Mesh>>());
            (&enemy, blade(&vertices))
        } else {
            let model = WeaponModel::shared(&mut world, chain.pair, WeaponMode::Melee);
            let vertices = model.vertices(world.resource::<Assets<Mesh>>());
            (&player, blade(&vertices))
        };
        // What the display shows is what the rules sweep. The second replay
        // only tells when the transition into each attack is over.
        let shown = replay(&chain, assets, &weapon, &library, &sources, false);
        let settled = replay(&chain, assets, &weapon, &library, &sources, true);
        for ((&track, shown), settled) in chain.tracks.iter().zip(shown).zip(settled) {
            // A sweep runs from the previous tick's sample, so the blade can
            // hit from the tick after its own motion has begun.
            let first = own_motion(&shown, &settled) + 1;
            let windows = windows(&shown, first..=shown.len() - 1);
            let blade_radius = weapon.radius + MARGIN;
            let reach = windows
                .iter()
                .flat_map(|&(start, end)| start - 1..=end)
                .flat_map(|tick| shown[usize::from(tick)])
                .map(|point| point.xz().length())
                .fold(0.0, f32::max)
                + blade_radius;
            baked.push(Baked {
                track,
                duration: track.duration(),
                samples: shown
                    .iter()
                    .map(|[grip, tip]| [grip.x, grip.y, grip.z, tip.x, tip.y, tip.z].map(round))
                    .collect(),
                windows,
                blade_radius: round(blade_radius),
                reach: round(reach),
            });
        }
    }
    baked
}

/// Four decimals: a tenth of a millimetre, and a stable text form.
fn round(value: f32) -> f32 {
    let rounded = (value * 1e4).round() / 1e4;
    if rounded == 0.0 { 0.0 } else { rounded }
}

/// The pattern which names `track` in the generated match.
fn pattern(track: TrackId) -> String {
    match track {
        TrackId::PlayerMelee { pair, stage } => format!(
            "TrackId::PlayerMelee {{ pair: WeaponPair::{pair:?}, stage: MeleeStage::{stage:?} }}"
        ),
        TrackId::PlayerSkill(kind) => format!("TrackId::PlayerSkill(ActionKind::{kind:?})"),
        TrackId::Boss { stage } => format!("TrackId::Boss {{ stage: MeleeStage::{stage:?} }}"),
        TrackId::Enemy => "TrackId::Enemy".into(),
    }
}

fn render(baked: &[Baked]) -> String {
    let mut text = String::from(
        "//! Every melee attack's blade path, baked from its animation.\n\
         //!\n\
         //! GENERATED by `make bake-hits` (client/hit_bake_tests.rs). Do not edit:\n\
         //! change the clip, the attack clip row, the weapon or the duration and\n\
         //! bake again. Samples are grip xyz then tip xyz in the attacker's frame\n\
         //! (feet at the origin, facing -Z), one per fixed step.\n\
         use super::sweep::HitTrack;\n\
         use crate::simulation::{ActionKind, MeleeStage, TrackId, WeaponPair};\n\n\
         pub(super) fn of(track: TrackId) -> Option<&'static HitTrack> {\n    Some(match track {\n",
    );
    for (index, track) in baked.iter().enumerate() {
        writeln!(text, "        {} => &TRACK_{index},", pattern(track.track)).unwrap();
    }
    text.push_str("        TrackId::PlayerSkill(_) => return None,\n    })\n}\n");
    for (index, track) in baked.iter().enumerate() {
        writeln!(text, "\n/// {:?}", track.track).unwrap();
        writeln!(text, "static TRACK_{index}: HitTrack = HitTrack {{").unwrap();
        writeln!(text, "    duration: {:?},", track.duration).unwrap();
        writeln!(text, "    blade_radius: {:?},", track.blade_radius).unwrap();
        writeln!(text, "    reach: {:?},", track.reach).unwrap();
        writeln!(text, "    windows: &{:?},", track.windows).unwrap();
        text.push_str("    samples: &[\n");
        for sample in &track.samples {
            writeln!(text, "        {sample:?},").unwrap();
        }
        text.push_str("    ],\n};\n");
    }
    text
}

fn path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/simulation/combat/hit_tracks.rs")
}

/// `make bake-hits`.
#[test]
#[ignore = "writes the generated hit data; run through `make bake-hits`"]
fn bake_hit_tracks() {
    let baked = bake();
    for track in &baked {
        println!(
            "{:?}: windows {:?}, reach {}, radius {}",
            track.track, track.windows, track.reach, track.blade_radius
        );
    }
    std::fs::write(path(), render(&baked)).unwrap();
}

#[test]
fn hit_tracks_match_animation() {
    const STALE: &str = "the baked hit data no longer matches the animation; run `make bake-hits`";
    let close = |a: f32, b: f32| (a - b).abs() <= 0.01;
    for baked in bake() {
        let track = baked.track;
        let committed = track
            .hit_track()
            .unwrap_or_else(|| panic!("{track:?} is not baked: {STALE}"));
        assert_eq!(committed.duration, baked.duration, "{track:?}: {STALE}");
        assert_eq!(committed.windows, baked.windows, "{track:?}: {STALE}");
        assert!(
            close(committed.blade_radius, baked.blade_radius)
                && close(committed.reach, baked.reach),
            "{track:?}: {STALE}"
        );
        assert_eq!(
            committed.samples.len(),
            baked.samples.len(),
            "{track:?}: {STALE}"
        );
        for (tick, (committed, baked)) in committed.samples.iter().zip(&baked.samples).enumerate() {
            assert!(
                committed.iter().zip(baked).all(|(a, b)| close(*a, *b)),
                "{track:?} tick {tick}: {STALE}"
            );
        }
    }
}

mod live {
    //! The end the bake exists for: in a real fight, the weapon the display
    //! draws is where the rules sweep.
    use super::*;
    use crate::client::{ActorView, animation::RigPose};
    use crate::simulation::{
        ActionIntent, AttackClock, Body, Boss, CollisionWorld, CombatIntent, Combatant, Enemy,
        Facing, Health, MotionState, MoveIntent, SimulationPlugin, SpawnPoint,
    };
    use std::time::Duration;

    /// The display may differ from the bake by this much where the blade can hit.
    const TOLERANCE: f32 = 0.03;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(100.0, vec![]))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>()
            .init_resource::<SourceAnimationLibrary>()
            .add_systems(Update, crate::client::animation::animate);
        app.finish();
        app
    }

    fn frame(app: &mut App) {
        let dt = Duration::from_secs_f64(1.0 / 60.0);
        app.world_mut().resource_mut::<Time<Fixed>>().advance_by(dt);
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().resource_mut::<Time>().advance_by(dt);
        app.world_mut().run_schedule(Update);
    }

    fn actor(app: &mut App, position: Vec2, appearance: Appearance) -> (Entity, Entity) {
        let owner = app
            .world_mut()
            .spawn((
                Body::new(position, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
                Health::new(100_000.0),
                SpawnPoint(position),
            ))
            .id();
        let view = app.world_mut().spawn(ActorView { owner }).id();
        crate::client::avatar::spawn_with_appearance(app.world_mut(), view, appearance);
        (owner, view)
    }

    /// How far the displayed blade is from the baked one at `clock`, and
    /// whether the blade can hit at that tick.
    fn deviation(
        app: &App,
        view: Entity,
        assets: &AvatarAssets,
        weapon: &Blade,
        clock: AttackClock,
    ) -> (f32, bool) {
        let library = app.world().resource::<AnimationLibrary>();
        let attachment = Attachment::Weapon {
            bone: library.bone("hand_r"),
            mode: WeaponMode::Melee,
        };
        let rig = app.world().get::<RigPose>(view).unwrap();
        let shown = attachment_transform(&attachment, &assets.attachment_pose(rig));
        let track = clock.track.hit_track().unwrap();
        let tick = HitTrack::tick(clock.elapsed);
        let baked = track.samples[usize::from(tick).min(track.samples.len() - 1)];
        let grip = Vec3::new(baked[0], baked[1], baked[2]);
        let tip = Vec3::new(baked[3], baked[4], baked[5]);
        (
            shown
                .translation
                .distance(grip)
                .max(shown.transform_point(weapon.tip).distance(tip)),
            track.is_active(tick),
        )
    }

    #[test]
    fn the_players_displayed_weapon_is_where_the_rules_sweep() {
        for pair in WeaponPair::ALL {
            let mut app = app();
            let (player, view) = actor(&mut app, Vec2::ZERO, PLAYER);
            app.world_mut()
                .entity_mut(player)
                .insert((Combatant::new(pair), CombatIntent::default()));
            let assets = AvatarAssets::loaded(app.world_mut(), PLAYER);
            let model = WeaponModel::shared(app.world_mut(), pair, WeaponMode::Melee);
            let weapon = blade(&model.vertices(app.world().resource::<Assets<Mesh>>()));
            // A passive target out of the blade's reach, for skills which need a lock.
            let (dummy, _) = actor(&mut app, Vec2::new(0.0, -3.0), ENEMY);
            app.world_mut()
                .entity_mut(dummy)
                .insert(Enemy::target_dummy());
            for _ in 0..40 {
                frame(&mut app);
            }
            // The whole three-stage chain, then the pair's blade skill.
            let mut checked = 0;
            let mut tracks = Vec::new();
            for step in 0..260 {
                let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
                if step < 110 {
                    intent.request_attack();
                } else if step == 180 {
                    intent.request_melee_skill();
                }
                frame(&mut app);
                let Some(clock) = app.world().get::<Combatant>(player).unwrap().attack_clock()
                else {
                    continue;
                };
                if !tracks.contains(&clock.track) {
                    tracks.push(clock.track);
                }
                let (deviation, active) = deviation(&app, view, &assets, &weapon, clock);
                if active {
                    checked += 1;
                    assert!(
                        deviation <= TOLERANCE,
                        "{:?} at {:.3}s: the displayed blade is {deviation} m from the baked one",
                        clock.track,
                        clock.elapsed
                    );
                }
            }
            assert_eq!(tracks.len(), 4, "{pair:?} played {tracks:?}");
            assert!(checked > 20);
        }
    }

    #[test]
    fn the_boss_displayed_blade_is_where_the_rules_sweep() {
        let mut app = app();
        let (player, _) = actor(&mut app, Vec2::ZERO, PLAYER);
        app.world_mut().entity_mut(player).insert((
            Combatant::new(WeaponPair::default()),
            CombatIntent::default(),
        ));
        let (boss, view) = actor(&mut app, Vec2::new(0.0, -4.0), ENEMY);
        app.world_mut()
            .entity_mut(boss)
            .insert((Boss::default(), Enemy::default()));
        let assets = AvatarAssets::loaded(app.world_mut(), ENEMY);
        let weapon = blade(&assets.blade_vertices(app.world().resource::<Assets<Mesh>>()));
        let mut stages = Vec::new();
        let mut checked = 0;
        for _ in 0..420 {
            frame(&mut app);
            let Some(clock) = app.world().get::<Boss>(boss).unwrap().attack_clock() else {
                continue;
            };
            if !stages.contains(&clock.track) {
                stages.push(clock.track);
            }
            let (deviation, active) = deviation(&app, view, &assets, &weapon, clock);
            if active {
                checked += 1;
                assert!(
                    deviation <= TOLERANCE,
                    "{:?} at {:.3}s: the displayed blade is {deviation} m from the baked one",
                    clock.track,
                    clock.elapsed
                );
            }
        }
        assert_eq!(stages.len(), 3, "the Boss played {stages:?}");
        assert!(checked > 20);
    }
}
