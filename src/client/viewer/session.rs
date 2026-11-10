//! The inspection clock and validated UI commands. The browser owns no playback time.
use bevy::prelude::*;

use super::super::{animation::PreviewClip, avatar::Appearance};
use crate::simulation::{WeaponMode, WeaponPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Weapon {
    Hammer,
    Axe,
    Musket,
    Bow,
    Blade,
    None,
}
impl Weapon {
    pub fn id(self) -> &'static str {
        match self {
            Self::Hammer => "fulangmao",
            Self::Axe => "zhexingcuishan",
            Self::Musket => "yinbingluo",
            Self::Bow => "yizhiying",
            Self::Blade => "choulingke_blade",
            Self::None => "none",
        }
    }
    pub fn parse(id: &str) -> Option<Self> {
        Some(match id {
            "fulangmao" => Self::Hammer,
            "zhexingcuishan" => Self::Axe,
            "yinbingluo" => Self::Musket,
            "yizhiying" => Self::Bow,
            "choulingke_blade" => Self::Blade,
            "none" => Self::None,
            _ => return None,
        })
    }
    pub fn pair(self) -> WeaponPair {
        match self {
            Self::Axe | Self::Bow => WeaponPair::ChainAxeBow,
            _ => WeaponPair::HammerMusket,
        }
    }
    pub fn mode(self) -> WeaponMode {
        if matches!(self, Self::Musket | Self::Bow) {
            WeaponMode::Ranged
        } else {
            WeaponMode::Melee
        }
    }
}

pub(super) fn appearance_id(value: Appearance) -> &'static str {
    match value {
        Appearance::Choulingke => "choulingke",
        Appearance::Lihuowang => "lihuowang",
        Appearance::Bailingmiao => "bailingmiao",
    }
}
pub(super) fn parse_appearance(id: &str) -> Option<Appearance> {
    match id {
        "choulingke" => Some(Appearance::Choulingke),
        "lihuowang" => Some(Appearance::Lihuowang),
        "bailingmiao" => Some(Appearance::Bailingmiao),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Game,
    Source,
    Reference,
}
impl Mode {
    pub fn id(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::Source => "source",
            Self::Reference => "reference",
        }
    }
    pub fn for_action(id: &str) -> Self {
        if id == "reference" {
            Self::Reference
        } else if id.starts_with("game:") {
            Self::Game
        } else {
            Self::Source
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct DebugOptions {
    pub grid: bool,
    pub axes: bool,
    pub bones: bool,
    pub grips: bool,
    pub bounds: bool,
}
impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            grid: true,
            axes: true,
            bones: false,
            grips: false,
            bounds: false,
        }
    }
}

#[derive(Resource)]
pub(super) struct Session {
    pub appearance: Appearance,
    pub weapon: Weapon,
    pub mode: Mode,
    pub action: String,
    pub time: f64,
    pub duration: f32,
    pub playing: bool,
    pub speed: f32,
    pub looping: bool,
    pub revision: u64,
    pub debug: DebugOptions,
    pub status: String,
    pub catalog: Vec<PreviewClip>,
    game_action: String,
    source_action: String,
}
impl Session {
    pub fn new(catalog: Vec<PreviewClip>) -> Self {
        let duration = catalog
            .iter()
            .find(|c| c.id == "game:idle")
            .map_or(3.0, |c| c.duration);
        let source_action = catalog
            .iter()
            .find(|c| Mode::for_action(&c.id) == Mode::Source)
            .map_or_else(|| "reference".into(), |c| c.id.clone());
        Self {
            appearance: Appearance::Lihuowang,
            weapon: Weapon::Hammer,
            mode: Mode::Game,
            action: "game:idle".into(),
            time: 0.0,
            duration,
            playing: false,
            speed: 1.0,
            looping: true,
            revision: 1,
            debug: DebugOptions::default(),
            status: String::new(),
            catalog,
            game_action: "game:idle".into(),
            source_action,
        }
    }
    pub fn preset(&self) -> &'static str {
        if self.mode != Mode::Game {
            return "custom";
        }
        match (self.appearance, self.weapon) {
            (Appearance::Lihuowang | Appearance::Choulingke, Weapon::Hammer)
                if !self.action.contains("companion") =>
            {
                "brother_hammer"
            }
            (Appearance::Lihuowang | Appearance::Choulingke, Weapon::Axe)
                if !self.action.contains("companion") =>
            {
                "brother_axe"
            }
            (Appearance::Bailingmiao, Weapon::Musket) if self.action.contains("companion") => {
                "sister_musket"
            }
            (Appearance::Bailingmiao, Weapon::Bow) if self.action.contains("companion") => {
                "sister_bow"
            }
            (Appearance::Choulingke, Weapon::Blade) if !self.action.contains("companion") => {
                "enemy_blade"
            }
            _ => "custom",
        }
    }
    pub fn frame(&self) -> u32 {
        (self.time * 60.).round() as u32
    }
    pub fn warning(&self) -> &'static str {
        match self.mode {
            Mode::Reference => "当前资产参考姿态；AV03参考不等于绑定姿态。",
            Mode::Source => "片段对照：保留现有根运动剥除／重定向，不叠加游戏持握修正；手持隐藏。",
            Mode::Game if self.preset() == "custom" => "自定义搭配诊断，不代表游戏装备或动作规则。",
            _ => "单角色60Hz显示检查；不验证伤害、位移或完整技能时序。",
        }
    }
    fn set_action(&mut self, id: &str, reset: bool) -> bool {
        let Some(clip) = self.catalog.iter().find(|c| c.id == id) else {
            return false;
        };
        self.action = clip.id.clone();
        self.duration = clip.duration.max(0.0);
        self.mode = Mode::for_action(&self.action);
        match self.mode {
            Mode::Game => self.game_action = self.action.clone(),
            Mode::Source => self.source_action = self.action.clone(),
            Mode::Reference => self.playing = false,
        }
        if reset {
            self.time = 0.;
        } else {
            self.time = self.time.min(self.duration as f64);
        }
        true
    }
    pub fn apply(&mut self, line: &str) -> Effect {
        let fields = line.split('\t').collect::<Vec<_>>();
        let mut effect = Effect::default();
        let valid = match fields.as_slice() {
            ["appearance", id] => parse_appearance(id).is_some_and(|value| {
                if value != self.appearance {
                    self.appearance = value;
                    effect.selection = true;
                }
                true
            }),
            ["weapon", id] => Weapon::parse(id).is_some_and(|value| {
                if value != self.weapon {
                    self.weapon = value;
                    effect.selection = true;
                }
                true
            }),
            ["preset", id] => {
                let preset = match *id {
                    "brother_hammer" => Some((Appearance::Lihuowang, Weapon::Hammer, "game:idle")),
                    "brother_axe" => Some((Appearance::Lihuowang, Weapon::Axe, "game:idle")),
                    "sister_musket" => Some((
                        Appearance::Bailingmiao,
                        Weapon::Musket,
                        "game:companion_hold",
                    )),
                    "sister_bow" => {
                        Some((Appearance::Bailingmiao, Weapon::Bow, "game:companion_hold"))
                    }
                    "enemy_blade" => Some((Appearance::Choulingke, Weapon::Blade, "game:idle")),
                    _ => None,
                };
                preset.is_some_and(|(appearance, weapon, action)| {
                    self.appearance = appearance;
                    self.weapon = weapon;
                    effect.selection = true;
                    self.set_action(action, false)
                })
            }
            ["mode", id] => {
                let action = match *id {
                    "game" => Some(self.game_action.clone()),
                    "source" => Some(self.source_action.clone()),
                    "reference" => Some("reference".into()),
                    _ => None,
                };
                action.is_some_and(|id| self.set_action(&id, true))
            }
            ["action", id] => self.set_action(id, true),
            ["play", value] => boolean(value).is_some_and(|playing| {
                self.playing = playing && self.duration > 0.;
                if playing && self.time >= self.duration as f64 {
                    self.time = 0.;
                }
                true
            }),
            ["seek", value] => number(value).is_some_and(|time| {
                self.time = (time as f64).clamp(0., self.duration as f64);
                if self.mode == Mode::Game {
                    self.time = (self.time * 60.).round() / 60.;
                }
                self.playing = false;
                true
            }),
            ["step", value] if matches!(*value, "-1" | "1") => {
                let delta = if *value == "-1" { -1. } else { 1. };
                self.time = ((self.time * 60.).round() + delta)
                    .clamp(0., (self.duration as f64 * 60.).round())
                    / 60.;
                self.playing = false;
                true
            }
            ["speed", value] => number(value).is_some_and(|speed| {
                if [0.25, 0.5, 1., 1.5, 2.].contains(&speed) {
                    self.speed = speed;
                    true
                } else {
                    false
                }
            }),
            ["loop", value] => boolean(value).is_some_and(|value| {
                self.looping = value;
                true
            }),
            ["debug", name, value] => boolean(value).is_some_and(|value| {
                match *name {
                    "grid" => self.debug.grid = value,
                    "axes" => self.debug.axes = value,
                    "bones" => self.debug.bones = value,
                    "grips" => self.debug.grips = value,
                    "bounds" => self.debug.bounds = value,
                    _ => return false,
                }
                true
            }),
            ["part", id, value] => boolean(value).is_some_and(|visible| {
                effect.part = Some((id.to_string(), visible));
                true
            }),
            ["orbit", x, y] => pair(x, y).is_some_and(|v| {
                effect.orbit = Some(v);
                true
            }),
            ["pan", x, y] => pair(x, y).is_some_and(|v| {
                effect.pan = Some(v);
                true
            }),
            ["zoom", value] => number(value).is_some_and(|v| {
                effect.zoom = Some(v.clamp(-2000., 2000.));
                true
            }),
            ["view", value]
                if ["front", "back", "left", "right", "top", "three-quarter"].contains(value) =>
            {
                effect.view = Some(value.to_string());
                true
            }
            ["fit"] => {
                effect.fit = true;
                true
            }
            ["reset"] => {
                effect.reset = true;
                true
            }
            _ => false,
        };
        if valid {
            self.status.clear();
        } else {
            self.status = "已忽略无效检查命令".into();
        }
        if effect.selection {
            self.revision = self.revision.wrapping_add(1).max(1);
        }
        effect
    }
    pub fn advance(&mut self, seconds: f32) {
        if !self.playing || !seconds.is_finite() || seconds <= 0. || self.duration <= 0. {
            return;
        }
        self.time += f64::from(seconds) * f64::from(self.speed);
        if self.time > self.duration as f64 {
            if self.looping {
                self.time %= self.duration as f64;
            } else {
                self.time = self.duration as f64;
                self.playing = false;
            }
        }
    }
    pub fn sample_seconds(&self) -> f32 {
        if self.mode == Mode::Game {
            self.frame() as f32 / 60.
        } else {
            self.time as f32
        }
    }
}
fn number(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().filter(|v| v.is_finite())
}
fn boolean(value: &str) -> Option<bool> {
    match value {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}
fn pair(x: &str, y: &str) -> Option<Vec2> {
    Some(Vec2::new(
        number(x)?.clamp(-2000., 2000.),
        number(y)?.clamp(-2000., 2000.),
    ))
}
#[derive(Default)]
pub(super) struct Effect {
    pub selection: bool,
    pub part: Option<(String, bool)>,
    pub orbit: Option<Vec2>,
    pub pan: Option<Vec2>,
    pub zoom: Option<f32>,
    pub view: Option<String>,
    pub fit: bool,
    pub reset: bool,
}
