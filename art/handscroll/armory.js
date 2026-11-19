// The two display rows follow the HUD names and combo windows in src/client/loadout.rs.
class HandscrollArmory {
  constructor(page, host) {
    this.page = page;
    this.sets = {
      hammer: {
        melee: "伏浪锚", ranged: "引冰落", meleeSkill: "范围击飞", rangedSkill: "单发引冰落",
        combo: "姐姐三枪", condition: "锤 E 击飞原目标后可用",
      },
      axe: {
        melee: "谪星·摧山", ranged: "羿之影", meleeSkill: "链斧横扫 · 减速", rangedSkill: "射出链斧",
        combo: "拉近 · 姐姐三箭", condition: "弓 Q 僵直原目标后可用",
      },
    };
    this.buttons = Array.from(page.querySelectorAll("[data-pair]"));
    this.images = Array.from(page.querySelectorAll("[data-pair-art]"));
    this.equip = page.querySelector("#equip-pair");
    this.equipped = "hammer";
    for (const button of this.buttons) {
      button.addEventListener("click", () => this.inspect(button.dataset.pair));
    }
    this.equip.addEventListener("click", () => {
      if (this.viewed === this.equipped) return;
      if (host) host.equip(this.viewed);
      this.setEquipped(host ? host.getState().pair : this.viewed);
    });
    this.inspect(this.equipped);
  }

  setEquipped(key) {
    if (key === this.equipped) return;
    this.equipped = key;
    this.inspect(this.viewed);
  }

  inspect(key) {
    this.viewed = key;
    const set = this.sets[key];
    for (const button of this.buttons) {
      button.setAttribute("aria-pressed", String(button.dataset.pair === key));
    }
    for (const image of this.images) image.hidden = image.dataset.pairArt !== key;
    for (const [field, value] of Object.entries(set)) {
      this.page.querySelector(`[data-weapon-field="${field}"]`).textContent = value;
    }
    const isEquipped = key === this.equipped;
    this.equip.textContent = isEquipped ? "已装配" : "装配此组";
    this.equip.setAttribute("aria-disabled", String(isEquipped));
    this.page.querySelector("#equipped-pair").textContent =
      this.buttons.find((button) => button.dataset.pair === this.equipped).textContent;
  }

  get selectedButton() {
    return this.buttons.find((button) => button.dataset.pair === this.viewed);
  }

  get equippedLoadout() {
    return this.sets[this.equipped];
  }

  get equippedNames() {
    const { melee, ranged } = this.equippedLoadout;
    return `${melee} ／ ${ranged}`;
  }
}
