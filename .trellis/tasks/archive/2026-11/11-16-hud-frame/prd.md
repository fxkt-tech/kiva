# HUD 桥接层：每帧一个具名字段快照 HudFrame

桥接整改第 2 步（第 1 步见 archive/2026-11/11-16-hud-bridge-data）。

## 现状问题

- `client/hud.rs` 每帧调用 8 个 `window.kivaHud.render*`，参数按位置传递（多处连续 f32），顺序写错时 Rust 和 JS 都不会报错。
  Node 测试按 JS 签名手写调用，不能发现 Rust 与 JS 不一致。
- 枚举靠数字和注释对齐：Boss 提示 `kind` 0/1/2、锁定 `mode` 0–3 加 `side` ±1、`defense` 0–4。
- 用哨兵值表示“没有”：`render_target("", 0, 0, "", 1, 1)`、`render_lock(0, …)`。
- 有字段每帧都发，但 JS 从不读取：`persona`、`weapon`、`cooldown`、`attack`、`guard`、`defense`、`fraction`。
- 每次 `render*` 都会触发一次完整的 `paint()`，一帧要重绘 8 次。
- 联动窗口总时长由 JS 自行记录“本次见过的最大剩余”，这属于页面里的派生时钟。

## 需求

1. Rust 每帧构造一个 `HudFrame`，序列化为 JSON，通过 `kivaHud.frame(json)` 调用一次；JS 每帧只 `paint()` 一次。
2. 字段具名（camelCase）。“没有”用 `null` 表示（target / cue / lock）；锁定位置用 `kind` 字符串区分（onScreen / aside / behind），Boss 提示用 `striking` 布尔值。
3. 只发送页面用得到的字段；删除上面列出的未用字段。
4. 模拟层 `ComboWindow` 增加 `duration`（窗口打开时的总时长），HUD 发送 `{left, total}`，JS 删除自行记录的 window。
5. 非有限数值序列化为 `null`，JS 端继续把非法值截断为 0。JSON 仍然用无依赖的 `client/json.rs` 生成。
6. 契约 fixture：Rust 测试把几种典型帧和 `loadouts_json()` 写入 `tests/fixtures/hud/*.json` 并与已有文件比较；
   设置 `KIVA_UPDATE_FIXTURES=1` 时重新生成。Node 测试只从这些 fixture 出发构造输入，覆盖字段时 key 必须已在 fixture 中存在。

## 不做

- 数据未变时跳过发送，以及把构建和发送拆成无 cfg 的纯函数（第 3 步）。
- 输入方向的位标志（另行处理）。

## 验收

- `rg "render(Loadout|Resources|Energy|Durability|Target|BossCue|Lock)\b" src web scripts` 无结果。
- Rust 测试与 fixture 一致；Node HUD 测试全部从 fixture 出发并通过；画面行为与改动前一致（需要浏览器验收）。
- `make check`、WASM Clippy、临时 Trunk 构建通过，`Cargo.lock` 不变。
