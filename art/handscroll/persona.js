// Browsing a portrait reads the equipped pair. The armory owns all equipment changes.
class HandscrollPersona {
  constructor(page, armory, ink, canAnimate) {
    this.page = page;
    this.armory = armory;
    this.buttons = Array.from(page.querySelectorAll("[data-persona]"));
    this.images = Array.from(page.querySelectorAll("[data-persona-art]"));
    this.reveal = new HandscrollReveal(page, this.images, ink, canAnimate, "portrait-reveal");
    this.selectedButton = this.buttons.find((button) => button.getAttribute("aria-pressed") === "true");
    for (const button of this.buttons) {
      button.addEventListener("click", (event) => this.inspect(button, event));
    }
    this.inspect(this.selectedButton);
  }

  inspect(button, event) {
    if (event && button === this.selectedButton) return;
    this.selectedButton = button;
    const { persona, slot, key, role } = button.dataset;
    const set = this.armory.equippedLoadout;
    this.page.dataset.portrait = persona;
    for (const item of this.buttons) item.setAttribute("aria-pressed", String(item === button));
    this.page.querySelector("#persona-name").textContent = button.querySelector("b").textContent;
    this.page.querySelector("#persona-role").textContent = role;
    this.page.querySelector("#persona-weapon").textContent = set[slot];
    this.page.querySelector("#persona-key").textContent = key;
    this.page.querySelector("#persona-skill").textContent = set[slot + "Skill"];
    this.page.querySelector("#persona-combo").textContent = set.combo;
    this.page.querySelector("#persona-condition").textContent = set.condition;
    const target = this.images.find((image) => image.dataset.personaArt === persona);
    this.reveal.show(target, button, event);
  }

  finishTransition() {
    this.reveal.finish();
  }
}
