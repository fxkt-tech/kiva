use super::{
    Action, EntryStatus, Leaf, NativeHandscroll,
    shop::{STALLS, WARES},
};
use crate::{client::loadout, simulation::WeaponPair};
use bevy::{
    prelude::*,
    text::{Justify, LineHeight},
};

pub(super) const PAPER: Color = Color::srgb_u8(241, 224, 191);
pub(super) const INK: Color = Color::srgb_u8(55, 43, 39);
const RED: Color = Color::srgb_u8(145, 57, 40);
const TEAL: Color = Color::srgb_u8(49, 91, 98);
const MUTED: Color = Color::srgb_u8(110, 88, 67);
type BoxPct = [f32; 4];

#[derive(Default, Resource)]
pub(super) struct Page {
    pub elements: Vec<Element>,
    // Separate the label from the (larger) painting area that opens it.
    hits: Vec<(Rect, Action)>,
    order: Vec<Action>,
}

pub(super) struct Element {
    bounds: Rect,
    text: String,
    size: f32,
    color: Color,
    fill: Color,
    border: Color,
    centered: bool,
    action: Option<Action>,
}

fn rect([x, y, w, h]: BoxPct) -> Rect {
    Rect::from_corners(
        Vec2::new(x * 16., y * 9.),
        Vec2::new((x + w) * 16., (y + h) * 9.),
    )
}

impl Element {
    pub(super) fn draw(
        &self,
        commands: &mut Commands,
        paper: Entity,
        font: &Handle<Font>,
        scale: f32,
        focus: Option<Action>,
    ) {
        let focused = self.action.is_some() && self.action == focus;
        let id = commands
            .spawn((
                ChildOf(paper),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(self.bounds.min.x * scale),
                    top: px(self.bounds.min.y * scale),
                    width: px(self.bounds.width() * scale),
                    height: px(self.bounds.height() * scale),
                    border: UiRect::all(px(if focused {
                        2. * scale
                    } else if self.border != Color::NONE {
                        scale
                    } else {
                        0.
                    })),
                    justify_content: if self.centered {
                        JustifyContent::Center
                    } else {
                        JustifyContent::Start
                    },
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(self.fill),
                BorderColor::all(if focused { RED } else { self.border }),
            ))
            .id();
        if !self.text.is_empty() {
            commands.spawn((
                ChildOf(id),
                Text::new(&self.text),
                TextFont::from_font_size(self.size * scale).with_font(font.clone()),
                TextColor(self.color),
                LineHeight::RelativeToFont(if self.text.contains('\n') { 1.05 } else { 1.22 }),
                TextLayout::justify(if self.centered {
                    Justify::Center
                } else {
                    Justify::Left
                }),
            ));
        }
    }
}

#[derive(Clone, Copy)]
enum Style {
    Plain,
    Tab(bool),
    Tag(bool),
    Seal,
    Muted,
}

impl Page {
    fn text(&mut self, area: BoxPct, words: impl Into<String>, size: f32, color: Color) {
        self.elements.push(Element {
            bounds: rect(area),
            text: words.into(),
            size,
            color,
            fill: Color::NONE,
            border: Color::NONE,
            centered: false,
            action: None,
        });
    }
    fn block(&mut self, area: BoxPct, fill: Color, border: Color) {
        self.text(area, "", 0., INK);
        let e = self.elements.last_mut().unwrap();
        e.fill = fill;
        e.border = border;
    }
    fn button(
        &mut self,
        area: BoxPct,
        words: impl Into<String>,
        size: f32,
        style: Style,
        action: Option<Action>,
    ) {
        let (color, fill, border) = match style {
            Style::Plain => (INK, Color::NONE, Color::NONE),
            Style::Tab(selected) => (if selected { RED } else { MUTED }, Color::NONE, Color::NONE),
            Style::Tag(false) => (INK, PAPER, MUTED),
            Style::Seal | Style::Tag(true) => (PAPER, RED, RED),
            Style::Muted => (MUTED, Color::srgba_u8(233, 216, 188, 170), MUTED),
        };
        self.elements.push(Element {
            bounds: rect(area),
            text: words.into(),
            size,
            color,
            fill,
            border,
            centered: true,
            action,
        });
        if let Some(action) = action {
            self.hits.push((rect(area), action));
            self.order.push(action);
        }
        if matches!(style, Style::Tab(true)) {
            let [x, y, w, h] = area;
            self.block([x + 0.5, y + h - 0.6, w - 1., 0.35], RED, Color::NONE);
        }
    }
    pub(super) fn hit(&self, pos: Vec2) -> Option<Action> {
        self.hits
            .iter()
            .rev()
            .find(|(bounds, _)| bounds.contains(pos))
            .map(|(_, action)| *action)
    }
    pub(super) fn center(&self, action: Action) -> Option<Vec2> {
        self.hits
            .iter()
            .find(|(_, a)| *a == action)
            .map(|(r, _)| r.center())
    }
    pub(super) fn step(&self, current: Option<Action>, reverse: bool) -> Option<Action> {
        let n = self.order.len();
        if n == 0 {
            return None;
        }
        let position = self.order.iter().position(|&a| Some(a) == current);
        let index = if reverse {
            position.map_or(n - 1, |i| (i + n - 1) % n)
        } else {
            position.map_or(0, |i| (i + 1) % n)
        };
        Some(self.order[index])
    }
}

pub(super) fn picture(scroll: &NativeHandscroll) -> usize {
    match scroll.leaf {
        Leaf::Home => 0,
        Leaf::Map => 1,
        Leaf::Persona => 2 + scroll.portrait,
        Leaf::Armory => 4 + scroll.pair,
        Leaf::Birds => 6,
        Leaf::Closeup => 7 + scroll.bird,
        Leaf::Supply => 10 + scroll.shop.stall,
    }
}

fn vertical(s: &str) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn compose(
    scroll: &NativeHandscroll,
    equipped: WeaponPair,
    status: &EntryStatus,
) -> Page {
    let mut p = Page::default();
    match scroll.leaf {
        Leaf::Home => home(&mut p, scroll, status),
        Leaf::Map => map(&mut p, scroll, status),
        Leaf::Persona => persona(&mut p, scroll.portrait, equipped),
        Leaf::Armory => armory(&mut p, scroll.pair, equipped),
        Leaf::Birds | Leaf::Closeup => birds(&mut p, scroll),
        Leaf::Supply => supply(&mut p, &scroll.shop),
    }
    if scroll.leaf != Leaf::Home {
        let (x, w, label) = if scroll.leaf == Leaf::Closeup {
            (88., 8.4, "回看全图")
        } else {
            (91.6, 4.8, "收卷")
        };
        p.button(
            [x, 5.1, w, 5.],
            label,
            21.,
            Style::Tag(false),
            Some(Action::Back),
        );
    }
    p
}

fn home(p: &mut Page, scroll: &NativeHandscroll, status: &EntryStatus) {
    let sortie = Action::Open(Leaf::Map);
    p.button(
        [17.5, 16.5, 8.2, 23.8],
        vertical("出击"),
        104.,
        Style::Plain,
        Some(sortie),
    );
    p.text([25.7, 17.2, 2.1, 11.4], vertical("第一章灰原"), 16., MUTED);
    p.button(
        [25.6, 32.5, 1.8, 5.4],
        vertical("废站"),
        17.,
        Style::Tag(false),
        None,
    );
    p.hits.push((rect([17., 15., 11., 26.]), sortie));
    p.block([95.4, 0., 4.6, 100.], Color::srgb_u8(221, 201, 162), MUTED);
    p.block([95.8, 1., 3.8, 98.], Color::NONE, MUTED);
    p.button(
        [96., 2.8, 3.4, 9.5],
        vertical("断轨听羽"),
        17.,
        Style::Plain,
        None,
    );
    for (i, (name, leaf)) in [
        ("人格", Leaf::Persona),
        ("武库", Leaf::Armory),
        ("图录", Leaf::Birds),
        ("补给", Leaf::Supply),
    ]
    .into_iter()
    .enumerate()
    {
        p.button(
            [95.8, 17.3 + 19.2 * i as f32, 3.8, 8.3],
            vertical(name),
            26.,
            Style::Plain,
            Some(Action::Open(leaf)),
        );
    }
    if scroll.can_enter(status) {
        p.button(
            [17.4, 43., 10.8, 5.5],
            "继续战斗",
            23.,
            Style::Seal,
            Some(Action::Enter),
        );
    } else if *status != EntryStatus::Ready {
        p.button(
            [14.8, 42., 16.3, 5.3],
            status.label(),
            17.,
            Style::Tag(false),
            None,
        );
    }
}

fn map(p: &mut Page, scroll: &NativeHandscroll, status: &EntryStatus) {
    let names = ["废站", "枯河", "沙堡"];
    for (i, (tag, area)) in [
        ("壹", [26.7, 52.4, 3.8, 13.1]),
        ("贰", [53.3, 37.5, 3.25, 11.3]),
        ("叁", [50.2, 10.4, 3.25, 11.3]),
    ]
    .into_iter()
    .enumerate()
    {
        let [x, y, w, h] = area;
        p.block([x + w * 0.5, y + h, 0.1, 2.8], MUTED, Color::NONE);
        p.button(
            area,
            format!("{tag}\n{}", vertical(names[i])),
            if i == scroll.stage { 29. } else { 25. },
            Style::Tag(i == scroll.stage),
            Some(Action::Stage(i)),
        );
        if i != 0 {
            let stamp = [x, y + h, w, 2.0];
            p.button(stamp, "未开放", 13., Style::Tag(i == scroll.stage), None);
            p.hits.push((rect(stamp), Action::Stage(i)));
        }
    }
    p.button(
        [75.7, 16.8, 9.8, 28.7],
        vertical(names[scroll.stage]),
        119.,
        Style::Plain,
        None,
    );
    p.text([85.5, 19.1, 2.1, 13.4], vertical("第一章灰原"), 19., MUTED);
    let detail = if scroll.stage == 0 {
        "目标　│　在积水中找到青鸾"
    } else {
        "状态　│　尚未开放"
    };
    p.button([70.4, 47.7, 24.9, 5.5], detail, 23., Style::Plain, None);
    let available = scroll.can_enter(status);
    let label = if scroll.stage != 0 {
        "进入战斗"
    } else if *status != EntryStatus::Ready {
        "准备出击"
    } else if scroll.playing_before {
        "继续战斗"
    } else {
        "进入战斗"
    };
    p.button(
        [75.1, 56.2, 14., 7.4],
        label,
        37.,
        if available { Style::Seal } else { Style::Muted },
        available.then_some(Action::Enter),
    );
    if scroll.stage == 0 {
        p.button(
            [69.7, 66., 25., 6.4],
            status.label(),
            19.,
            Style::Plain,
            None,
        );
    }
}

fn persona(p: &mut Page, portrait: usize, equipped: WeaponPair) {
    let hud = &loadout::of(equipped).hud;
    let (name, role, weapon, key, skill, pigment) = if portrait == 0 {
        ("朱雀", "近战", hud.melee, "E", hud.melee_skill, RED)
    } else {
        ("青鸾", "远程", hud.ranged, "Q", hud.ranged_skill, TEAL)
    };
    for (i, tab) in ["弟 朱雀", "姐 青鸾"].into_iter().enumerate() {
        p.button(
            [71.5 + 6.9 * i as f32, 15., 6.4, 5.3],
            tab,
            24.,
            Style::Tab(i == portrait),
            Some(Action::Portrait(i)),
        );
    }
    p.text([71.2, 24.2, 14.7, 12.], name, 102., INK);
    p.text([86.1, 28., 6., 5.], role, 26., pigment);
    p.text([71.5, 38., 5.1, 5.], "武器", 20., MUTED);
    p.text([76.2, 37.4, 20.3, 6.2], weapon, 36., INK);
    p.text([71.5, 45.7, 25.3, 5.], format!("{key}　{skill}"), 23., INK);
    p.text(
        [71.5, 54., 25.1, 5.3],
        format!("联动　{}", hud.combo.replace("{companion}", "姐姐")),
        28.,
        INK,
    );
    p.text([71.5, 61., 25.1, 4.2], hud.combo_condition, 20., MUTED);
}

fn pair_name(pair: WeaponPair) -> String {
    let hud = &loadout::of(pair).hud;
    format!("{}·{}", hud.melee_glyph, hud.ranged_glyph)
}
fn armory(p: &mut Page, index: usize, equipped: WeaponPair) {
    let pair = WeaponPair::ALL[index];
    let hud = &loadout::of(pair).hud;
    p.text([72.8, 11.2, 22.5, 9.7], "武库", 76., INK);
    for (i, pair) in WeaponPair::ALL.into_iter().enumerate() {
        p.button(
            [73.1 + 7.1 * i as f32, 22.2, 6., 5.5],
            pair_name(pair),
            26.,
            Style::Tab(i == index),
            Some(Action::Pair(i)),
        );
    }
    for (y, who, weapon, key, skill, pigment) in [
        (31.6, "朱雀 · 近战", hud.melee, "E", hud.melee_skill, RED),
        (41.5, "青鸾 · 远程", hud.ranged, "Q", hud.ranged_skill, TEAL),
    ] {
        p.text([73.1, y, 7., 4.3], who, 19., pigment);
        p.text([80.7, y - 0.7, 16.5, 5.5], weapon, 33., INK);
        p.text(
            [80.7, y + 4.8, 17.7, 3.8],
            format!("{key} · {skill}"),
            18.,
            MUTED,
        );
    }
    p.text(
        [73.1, 51.8, 24., 5.1],
        format!("联动　{}", hud.combo.replace("{companion}", "姐姐")),
        28.,
        INK,
    );
    p.text([73.1, 57.8, 24., 4.5], hud.combo_condition, 20., MUTED);
    let current = pair == equipped;
    p.button(
        [73.1, 65.4, 8.3, 5.3],
        if current { "已装配" } else { "装配此组" },
        24.,
        if current { Style::Muted } else { Style::Seal },
        (!current).then_some(Action::Equip),
    );
    p.text(
        [82.5, 65.6, 15.4, 5.1],
        format!("当前　{}", pair_name(equipped)),
        19.,
        MUTED,
    );
}

struct Bird {
    name: &'static str,
    alias: &'static str,
    place: &'static str,
    habits: [&'static str; 2],
    tactic: [&'static str; 2],
    observation: &'static str,
    marker: BoxPct,
    body: BoxPct,
}
const BIRDS: [Bird; 3] = [
    Bird {
        name: "灰鸦",
        alias: "废站鸦群",
        place: "灰原 · 废站",
        habits: ["栖在废站的电线上，", "爱啄发亮的东西。"],
        tactic: ["待鸦群聚拢，", "一锤可击飞数只。"],
        observation: "一枚铜垫圈挂在喙尖。废站的电线上，常有同伴等它归群。",
        marker: [34.8, 40.6, 2.9, 9.],
        body: [8., 31., 32., 46.],
    },
    Bird {
        name: "秃鹫",
        alias: "盘空者",
        place: "灰原 · 断桥上空",
        habits: ["在断桥上空盘旋，", "专等猎物力竭。"],
        tactic: ["待它落地，", "趁僵直近身。"],
        observation: "翼尖张开，借风盘旋。桥下的猎物力竭时，它才收翼俯冲。",
        marker: [60.5, 13.4, 2.5, 8.4],
        body: [47., 14., 24., 19.],
    },
    Bird {
        name: "雾鸮",
        alias: "河雾里的眼",
        place: "断桥 · 河雾深处",
        habits: ["藏在桥下的芦苇里，", "借河雾隐去身形。"],
        tactic: ["循眼光远攻，", "命中便会现形。"],
        observation: "面盘从芦苇间探出。河雾一浓，先隐去的是羽缘，最后才是那双眼。",
        marker: [67.7, 56.2, 2.5, 8.4],
        body: [63., 56., 8., 16.],
    },
];

fn birds(p: &mut Page, scroll: &NativeHandscroll) {
    let b = &BIRDS[scroll.bird];
    if scroll.leaf == Leaf::Closeup {
        p.text([77., 23., 15.2, 13.], b.name, 94., INK);
        p.text([91.1, 23., 3., 13.], vertical(b.alias), 17., RED);
        p.text([77., 37., 20., 4.5], b.place, 20., MUTED);
        p.text([77., 44., 20., 20.], b.observation, 26., INK);
        p.text([77., 68., 20., 4.7], "应对", 20., RED);
        p.text([77., 73., 20., 10.], b.tactic.concat(), 24., INK);
    } else {
        for (i, bird) in BIRDS.iter().enumerate() {
            let action = Action::Bird(i);
            p.hits.push((rect(bird.body), action));
            p.button(
                bird.marker,
                vertical(bird.name),
                25.,
                Style::Tag(scroll.bird == i),
                Some(action),
            );
        }
        p.text([95.2, 15.9, 2.5, 11.8], vertical("百鸟图"), 24., MUTED);
        p.text([88.2, 23.2, 6.4, 23.], vertical(b.name), 91., INK);
        for (i, line) in b.habits.iter().enumerate() {
            p.text(
                [85.6 - 2.2 * i as f32, 31.5, 2.1, 28.],
                vertical(line),
                24.,
                INK,
            );
        }
        p.text([80.4, 40., 2.1, 7.], vertical("应对"), 18., RED);
        for (i, line) in b.tactic.iter().enumerate() {
            p.text(
                [80.4 - 2.1 * i as f32, 47., 2.1, 24.],
                vertical(line),
                21.,
                INK,
            );
        }
    }
}

fn supply(p: &mut Page, shop: &super::shop::Shop) {
    let index = shop.index();
    let ware = &WARES[index];
    p.text([70.2, 5.2, 9., 4.1], "补给", 24., MUTED);
    for (i, name) in STALLS.into_iter().enumerate() {
        p.button(
            [70.2 + 5.1 * i as f32, 13., 5.1, 5.6],
            name,
            26.,
            Style::Tab(i == shop.stall),
            Some(Action::Stall(i)),
        );
    }
    let x = [[11.5, 32.3, 55.1], [11.1, 29.6, 52.1], [10.7, 30.2, 53.]][shop.stall];
    for (slot, middle) in x.into_iter().enumerate() {
        let which = shop.stall * 3 + slot;
        let good = &WARES[which];
        let text = if shop.bought[which] == good.limit {
            format!("{} │ 售罄", good.name)
        } else {
            good.name.into()
        };
        let action = Action::Ware(which);
        p.hits.push((rect([middle - 9.3, 40., 18.6, 42.]), action));
        let width = text.chars().count() as f32 * 1.75 + 1.9;
        p.button(
            [middle - width / 2., 75.7, width, 6.3],
            text,
            24.,
            Style::Tag(which == index),
            Some(action),
        );
    }
    p.text(
        [70.2, 26.3, 26.4, 10.],
        ware.name,
        if index == 3 { 57. } else { 70. },
        INK,
    );
    p.text([70.2, 36.5, 26.4, 7.8], ware.description, 22., INK);
    let currency = if ware.feather { "羽晶" } else { "拾荒币" };
    p.text(
        [70.2, 44.3, 26.4, 4.8],
        format!("每份 ×{}　　　 单价 {} {currency}", ware.pack, ware.price),
        20.,
        MUTED,
    );
    p.block([70.2, 49.6, 26.4, 0.1], MUTED, Color::NONE);
    p.text(
        [70.2, 50.6, 18., 4.6],
        format!("今日余 {} / {} 份", shop.remaining(), ware.limit),
        21.,
        INK,
    );
    let can_reduce = shop.quantity > 1;
    let can_add = shop.quantity < shop.remaining();
    p.button(
        [88.7, 50.4, 2.8, 4.8],
        "−",
        26.,
        if can_reduce {
            Style::Tag(false)
        } else {
            Style::Muted
        },
        can_reduce.then_some(Action::Adjust(-1)),
    );
    p.button(
        [91.6, 50.4, 2., 4.8],
        shop.quantity.to_string(),
        23.,
        Style::Plain,
        None,
    );
    p.button(
        [93.8, 50.4, 2.8, 4.8],
        "＋",
        26.,
        if can_add {
            Style::Tag(false)
        } else {
            Style::Muted
        },
        can_add.then_some(Action::Adjust(1)),
    );
    p.text(
        [70.2, 57.3, 18.7, 6.1],
        format!("合计　{} {currency}", shop.quantity * ware.price),
        25.,
        INK,
    );
    let can_buy = shop.affordable();
    let label = if shop.remaining() == 0 {
        "今日售罄"
    } else if !can_buy {
        "余额不足"
    } else {
        "购买"
    };
    p.button(
        [89.5, 57.3, 7.1, 6.1],
        label,
        if can_buy { 30. } else { 22. },
        if can_buy { Style::Seal } else { Style::Muted },
        can_buy.then_some(Action::Buy),
    );
    p.text([70.2, 65.3, 26.4, 6.8], &shop.receipt, 21., RED);
    p.block([70.2, 85.6, 26.4, 0.12], MUTED, Color::NONE);
    for (i, (label, value)) in [
        ("拾荒币", shop.coins),
        ("羽晶", shop.feathers),
        ("体力", shop.stamina),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 70.2 + i as f32 * 9.1;
        p.text([x, 86.6, 8.8, 3.3], label, 19., MUTED);
        p.text([x, 90.1, 8.8, 4.8], value.to_string(), 26., INK);
    }
}
