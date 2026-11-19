class HandscrollSupply {
  constructor(page, ink, canAnimate) {
    this.page = page;
    this.wallet = { coin: 48210, feather: 1380, stamina: 120 };
    this.currencies = { coin: "拾荒币", feather: "羽晶" };
    // A price and a limit apply to one bundle; the receipt counts the items inside.
    this.wares = {
      tea: { name: "灰烬茶", pack: 1, currency: "feather", price: 60, limit: 3, bought: 1, stamina: 60,
        desc: "饮一壶热茶，回复 60 点体力。" },
      stone: { name: "淬火砺石", pack: 3, currency: "coin", price: 2400, limit: 6, bought: 0,
        desc: "砺石淬过火，可开刃、校弓。" },
      page: { name: "图录残页", pack: 1, currency: "coin", price: 3000, limit: 3, bought: 0,
        desc: "补全未遇之鸟的线稿。" },
      gift: { name: "余烬回响礼匣", pack: 1, currency: "feather", price: 680, limit: 1, bought: 0,
        desc: "限时礼匣，内藏朱羽 ×10、青翎 ×10、能量芯 ×3。" },
      red: { name: "朱羽", pack: 5, currency: "feather", price: 120, limit: 5, bought: 2,
        desc: "朱雀落羽，可增弟弟羽力。" },
      blue: { name: "青翎", pack: 5, currency: "feather", price: 120, limit: 5, bought: 0,
        desc: "青鸾落翎，可增姐姐羽力。" },
      core: { name: "能量芯", pack: 1, currency: "coin", price: 6000, limit: 2, bought: 0,
        desc: "炼厂遗物，可助武器突破。" },
      scrap: { name: "废铁", pack: 20, currency: "coin", price: 800, limit: 10, bought: 4,
        desc: "废站旧铁，供锻造与修补。" },
      gear: { name: "零件", pack: 5, currency: "coin", price: 1500, limit: 10, bought: 10,
        desc: "齿轮、弹簧与螺栓，可升级武器。" },
    };
    const stalls = {
      travel: ["tea", "stone", "page"],
      plume: ["gift", "red", "blue"],
      parts: ["core", "scrap", "gear"],
    };
    this.receipt = page.querySelector("#supply-receipt");
    this.stallButtons = Array.from(page.querySelectorAll("[data-stall]"));
    this.sheets = Array.from(page.querySelectorAll("[data-stock]"));
    this.reveal = new HandscrollReveal(page, this.sheets, ink, canAnimate, "supply-reveal");
    for (const sheet of this.sheets) {
      const goods = sheet.querySelector(".supply-goods");
      for (const key of stalls[sheet.dataset.stock]) {
        const ware = this.wares[key];
        const button = document.createElement("button");
        button.type = "button";
        button.tabIndex = 0;
        button.dataset.ware = key;
        button.setAttribute("aria-controls", "supply-ledger");
        const label = document.createElement("span");
        label.className = "supply-tag";
        label.textContent = ware.name;
        const sold = document.createElement("small");
        sold.textContent = "售罄";
        sold.hidden = ware.bought < ware.limit;
        label.append(sold);
        button.append(label);
        goods.append(button);
        button.addEventListener("click", () => this.selectWare(button));
      }
    }
    for (const button of this.stallButtons) {
      button.addEventListener("click", (event) => this.selectStall(button, event));
    }
    page.querySelector("#supply-less").addEventListener("click", () => this.adjust(-1));
    page.querySelector("#supply-more").addEventListener("click", () => this.adjust(1));
    page.querySelector("#supply-buy").addEventListener("click", () => this.purchase());
    this.selectStall(this.stallButtons[0]);
  }

  selectStall(button, event) {
    if (button === this.stallButton) return;
    this.stallButton = button;
    const stall = button.dataset.stall;
    this.page.dataset.stallView = stall;
    for (const tab of this.stallButtons) tab.setAttribute("aria-pressed", String(tab === button));
    const sheet = this.sheets.find((candidate) => candidate.dataset.stock === stall);
    this.reveal.show(sheet, button, event);
    for (const candidate of this.sheets) candidate.inert = candidate !== sheet;
    this.selectWare(sheet.querySelector('[data-ware][aria-pressed="true"]') || sheet.querySelector("[data-ware]"));
  }

  selectWare(button) {
    this.ware = this.wares[button.dataset.ware];
    this.page.dataset.wareView = button.dataset.ware;
    this.quantity = Math.min(1, this.ware.limit - this.ware.bought);
    this.selectedButton = button;
    for (const item of button.parentElement.children) item.setAttribute("aria-pressed", String(item === button));
    this.receipt.textContent = "";
    this.render();
  }

  adjust(delta) {
    const next = this.quantity + delta;
    if (next < 1 || next > this.ware.limit - this.ware.bought) return;
    this.quantity = next;
    this.receipt.textContent = "";
    this.render();
  }

  purchase() {
    const ware = this.ware;
    const quantity = this.quantity;
    const cost = ware.price * quantity;
    if (quantity < 1 || quantity > ware.limit - ware.bought || cost > this.wallet[ware.currency]) return;
    this.wallet[ware.currency] -= cost;
    ware.bought += quantity;
    const stamina = quantity * (ware.stamina || 0);
    this.wallet.stamina += stamina;
    this.quantity = Math.min(1, ware.limit - ware.bought);
    this.render();
    this.receipt.textContent = `购得${ware.name} ×${quantity * ware.pack}` + (stamina ? `，体力 +${stamina}` : "");
  }

  render() {
    const { ware, quantity, wallet, page } = this;
    const remaining = ware.limit - ware.bought;
    const total = ware.price * quantity;
    const currency = this.currencies[ware.currency];
    page.querySelector("#supply-name").textContent = ware.name;
    page.querySelector("#supply-desc").textContent = ware.desc;
    page.querySelector("#supply-pack").textContent = `每份 ×${ware.pack}`;
    page.querySelector("#supply-price").textContent = `单价 ${ware.price.toLocaleString("zh-CN")} ${currency}`;
    page.querySelector("#supply-stock").textContent = `今日余 ${remaining} / ${ware.limit} 份`;
    page.querySelector("#supply-count").textContent = quantity;
    page.querySelector("#supply-less").setAttribute("aria-disabled", String(quantity <= 1));
    page.querySelector("#supply-more").setAttribute("aria-disabled", String(quantity >= remaining));
    page.querySelector("#supply-total").textContent = `${total.toLocaleString("zh-CN")} ${currency}`;
    const buy = page.querySelector("#supply-buy");
    buy.textContent = !remaining ? "今日售罄" : total > wallet[ware.currency] ? "余额不足" : "购买";
    buy.setAttribute("aria-disabled", String(!remaining || total > wallet[ware.currency]));
    this.selectedButton.querySelector("small").hidden = Boolean(remaining);
    page.querySelector("#supply-coins").textContent = wallet.coin.toLocaleString("zh-CN");
    page.querySelector("#supply-feathers").textContent = wallet.feather.toLocaleString("zh-CN");
    page.querySelector("#supply-stamina").textContent = wallet.stamina;
  }
}
