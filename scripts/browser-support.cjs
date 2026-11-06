const { chromium } = require("playwright");
const fs = require("node:fs/promises");
const { createReadStream } = require("node:fs");
const http = require("node:http");
const path = require("node:path");

async function serveBuild(directory) {
  const root = path.resolve(directory);
  const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm" };
  const server = http.createServer(async (request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const file = path.resolve(root, `.${pathname === "/" ? "/index.html" : pathname}`);
    if (!file.startsWith(root + path.sep)) { response.writeHead(403); response.end(); return; }
    try {
      const stat = await fs.stat(file);
      response.writeHead(200, { "Content-Type": types[path.extname(file)] || "application/octet-stream", "Content-Length": stat.size });
      createReadStream(file).pipe(response);
    } catch {
      response.writeHead(404); response.end();
    }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  return { url: `http://127.0.0.1:${server.address().port}`, close: () => new Promise((resolve) => server.close(resolve)) };
}

async function launchBrowser() {
  const args = ["--no-sandbox", "--force-device-scale-factor=1", "--no-proxy-server"];
  if (process.env.KIVA_BROWSER_SOFTWARE === "1") {
    args.push("--enable-unsafe-webgpu", "--use-angle=swiftshader",
      "--enable-features=Vulkan", "--use-vulkan=swiftshader",
      "--disable-vulkan-surface", "--disable-gpu-watchdog");
  }
  return chromium.launch({
    headless: process.env.KIVA_BROWSER_HEADED !== "1",
    channel: process.env.KIVA_BROWSER_CHANNEL || "chromium",
    args,
  });
}

async function enterGame(page, url = process.env.KIVA_TEST_URL || "http://127.0.0.1:3221") {
  await page.goto(url, { timeout: 120000 });
  await page.locator("#enter").waitFor({ state: "visible", timeout: 120000 });
  await page.bringToFront();
  await page.locator("#enter").click();
  await page.waitForFunction(() => window.__kivaState && !window.__kivaState.paused);
}

async function worldPoint(page, position) {
  return page.evaluate(([x, z, y = 0]) => {
    const { view_projection: m, viewport: [width, height] } = window.__kivaState;
    const w = m[3] * x + m[7] * y + m[11] * z + m[15];
    return [(m[0] * x + m[4] * y + m[8] * z + m[12]) / w * width / 2 + width / 2,
      height / 2 - (m[1] * x + m[5] * y + m[9] * z + m[13]) / w * height / 2];
  }, position);
}

async function clickWorld(page, position, button = "left") {
  const [x, y] = await worldPoint(page, position);
  await page.mouse.click(x, y, { button, delay: 90 });
}

async function reportFailure(page) {
  console.error("Browser state:", await page.evaluate(() => ({
    startup: window.__kivaStartup,
    paused: window.__kivaState?.paused,
    tick: window.__kivaState?.tick,
    aim: window.__kivaState?.aim,
    units: window.__kivaState?.units,
    notice: window.__kivaState?.notice,
    focused: document.hasFocus(),
    activeElement: document.activeElement?.id,
  })).catch(() => "page unavailable"));
}

module.exports = { launchBrowser, enterGame, worldPoint, clickWorld, reportFailure, serveBuild };
