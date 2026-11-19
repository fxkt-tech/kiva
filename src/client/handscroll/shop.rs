//! Each stock limit and price is per bundle, matching the illustrated supply stall.
pub(super) struct Ware {
    pub name: &'static str,
    pub description: &'static str,
    pub pack: u32,
    pub price: u32,
    pub limit: u32,
    pub feather: bool,
}

pub(super) const STALLS: [&str; 3] = ["行旅", "羽货", "工料"];
const fn ware(
    name: &'static str,
    description: &'static str,
    pack: u32,
    price: u32,
    limit: u32,
    feather: bool,
) -> Ware {
    Ware {
        name,
        description,
        pack,
        price,
        limit,
        feather,
    }
}
pub(super) const WARES: [Ware; 9] = [
    ware("灰烬茶", "饮一壶热茶，回复 60 点体力。", 1, 60, 3, true),
    ware("淬火砺石", "砺石淬过火，可开刃、校弓。", 3, 2400, 6, false),
    ware("图录残页", "补全未遇之鸟的线稿。", 1, 3000, 3, false),
    ware(
        "余烬回响礼匣",
        "限时礼匣，内藏朱羽 ×10、青翎 ×10、能量芯 ×3。",
        1,
        680,
        1,
        true,
    ),
    ware("朱羽", "朱雀落羽，可增弟弟羽力。", 5, 120, 5, true),
    ware("青翎", "青鸾落翎，可增姐姐羽力。", 5, 120, 5, true),
    ware("能量芯", "炼厂遗物，可助武器突破。", 1, 6000, 2, false),
    ware("废铁", "废站旧铁，供锻造与修补。", 20, 800, 10, false),
    ware("零件", "齿轮、弹簧与螺栓，可升级武器。", 5, 1500, 10, false),
];

pub(super) struct Shop {
    pub coins: u32,
    pub feathers: u32,
    pub stamina: u32,
    pub bought: [u32; 9],
    pub stall: usize,
    selected: [usize; 3],
    pub quantity: u32,
    pub receipt: String,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            coins: 48210,
            feathers: 1380,
            stamina: 120,
            bought: [1, 0, 0, 0, 2, 0, 0, 4, 10],
            stall: 0,
            selected: [0, 3, 6],
            quantity: 1,
            receipt: String::new(),
        }
    }
}

impl Shop {
    pub fn index(&self) -> usize {
        self.selected[self.stall]
    }
    pub fn remaining(&self) -> u32 {
        WARES[self.index()].limit - self.bought[self.index()]
    }
    pub fn affordable(&self) -> bool {
        let ware = &WARES[self.index()];
        let balance = if ware.feather {
            self.feathers
        } else {
            self.coins
        };
        self.quantity > 0
            && self.quantity <= self.remaining()
            && self.quantity * ware.price <= balance
    }
    pub fn select_stall(&mut self, stall: usize) {
        if self.stall != stall {
            self.stall = stall;
            self.select(self.index());
        }
    }
    pub fn select(&mut self, index: usize) {
        self.selected[self.stall] = index;
        self.quantity = self.remaining().min(1);
        self.receipt.clear();
    }
    pub fn adjust(&mut self, delta: i32) {
        let next = self.quantity.saturating_add_signed(delta);
        if (1..=self.remaining()).contains(&next) {
            self.quantity = next;
            self.receipt.clear();
        }
    }
    pub fn purchase(&mut self) {
        if !self.affordable() {
            return;
        }
        let index = self.index();
        let ware = &WARES[index];
        let quantity = self.quantity;
        let cost = quantity * ware.price;
        if ware.feather {
            self.feathers -= cost;
        } else {
            self.coins -= cost;
        }
        self.bought[index] += quantity;
        self.quantity = self.remaining().min(1);
        self.receipt = format!("购得{} ×{}", ware.name, quantity * ware.pack);
        if index == 0 {
            self.stamina += quantity * 60;
            self.receipt.push_str(&format!("，体力 +{}", quantity * 60));
        }
    }
}
