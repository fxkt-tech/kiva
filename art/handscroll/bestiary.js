const handscrollBirds = {
  crow: {
    alias: "废站鸦群", location: "灰原 · 废站",
    habits: ["栖在废站的电线上，", "爱啄发亮的东西。"],
    tactic: ["待鸦群聚拢，", "一锤可击飞数只。"],
    observation: "一枚铜垫圈挂在喙尖。废站的电线上，常有同伴等它归群。",
  },
  vulture: {
    alias: "盘空者", location: "灰原 · 断桥上空",
    habits: ["在断桥上空盘旋，", "专等猎物力竭。"],
    tactic: ["待它落地，", "趁僵直近身。"],
    observation: "翼尖张开，借风盘旋。桥下的猎物力竭时，它才收翼俯冲。",
  },
  owl: {
    alias: "河雾里的眼", location: "断桥 · 河雾深处",
    habits: ["藏在桥下的芦苇里，", "借河雾隐去身形。"],
    tactic: ["循眼光远攻，", "命中便会现形。"],
    observation: "面盘从芦苇间探出。河雾一浓，先隐去的是羽缘，最后才是那双眼。",
  },
};

class HandscrollBestiary {
  constructor(page, ink, canAnimate) {
    this.page = page;
    this.overview = page.querySelector("#bird-overview");
    this.detail = page.querySelector("#bird-closeup");
    this.back = page.querySelector("#bestiary-overview-back");
    this.buttons = Array.from(page.querySelectorAll(".bird-marks button"));
    this.selectedButton = this.buttons.find((button) => button.hasAttribute("aria-current"));
    this.reveal = new HandscrollReveal(page, [this.overview, this.detail], ink, canAnimate, "bird-reveal");

    this.overview.addEventListener("click", (event) => {
      const marker = event.target.closest(".bird-marks button, .bird-body");
      if (!marker) return;
      const id = marker.dataset.bird || marker.dataset.birdBody;
      const button = this.buttons.find((item) => item.dataset.bird === id);
      this.open(button, event);
    });
    this.back.addEventListener("click", (event) => this.showOverview(event));
  }

  get inDetail() {
    return this.reveal.visible === this.detail;
  }

  get focusControl() {
    return this.inDetail ? this.back : this.selectedButton;
  }

  open(button, event) {
    if (button !== this.selectedButton && this.reveal.masked) this.reveal.finish();
    this.selectedButton = button;
    const key = button.dataset.bird;
    const bird = handscrollBirds[key];
    this.page.dataset.selected = key;
    for (const image of this.detail.querySelectorAll("[data-bird-art]")) {
      image.hidden = image.dataset.birdArt !== key;
    }
    this.detail.querySelector("#bird-study-name").textContent = button.textContent;
    this.detail.querySelector(".bird-alias").textContent = bird.alias;
    this.detail.querySelector(".bird-location").textContent = bird.location;
    this.detail.querySelector(".bird-observation").textContent = bird.observation;
    this.detail.querySelector(".bird-encounter span").textContent = bird.tactic.join("");
    this.switchView(this.detail, button, event);
  }

  showOverview(event) {
    const selected = this.selectedButton;
    const bird = handscrollBirds[selected.dataset.bird];
    for (const button of this.buttons) {
      if (button === selected) button.setAttribute("aria-current", "true");
      else button.removeAttribute("aria-current");
    }
    this.overview.querySelector("#bird-name").textContent = selected.textContent;
    for (const [selector, lines] of [[".bird-habits", bird.habits], [".bird-tactic", bird.tactic]]) {
      this.overview.querySelectorAll(selector + " span").forEach((span, i) => { span.textContent = lines[i]; });
    }
    this.switchView(this.overview, this.back, event);
  }

  switchView(target, origin, event) {
    this.page.dataset.view = target === this.detail ? "detail" : "overview";
    this.reveal.show(target, origin, event);
    this.overview.inert = this.inDetail;
    this.detail.inert = !this.inDetail;
    if (event && !event.detail) this.focusControl.focus({ preventScroll: true });
  }

  finishTransition() {
    this.reveal.finish();
  }
}
