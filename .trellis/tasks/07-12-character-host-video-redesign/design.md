# 角色、主理人与剧本系统设计

## 1. Design thesis

Kiva 的稳定资产分为三层，只有第三层按局变化：

1. **规则身份**：狼人、预言家等，决定能力、信息和胜负；本任务不修改。
2. **固定演员**：12 名玩家角色与主理人闻舟，决定外形、声音、思考入口和表达节奏；跨局不变。
3. **剧本**：创建游戏时选择的共同叙事和制作包装，决定主题背景、氛围和页面/视频视觉；不改变规则或演员。

```text
Script Library ──select──┐
Preset / Random seats ──┼──> Game snapshot ──> Prompt context
Single Presenter ────────┤          │
Fixed Character Cast ────┘          ├──> Home cards
                                    └──> Preview / Remotion export
```

剧本是制作包，不是带个人秘密的剧情本。模型可以把剧本背景用于语气、类比和气氛，但不能把它当作判断身份的证据。

## 2. Domain contracts

### 2.1 Game script definition

新增 `src/core/game-script.ts`，作为剧本未知数据的唯一校验入口。

```ts
type GameScriptVisualStyleKey = "legacy_v1" | "midnight_archive_v1";

type GameScriptPresentation = {
  readonly styleKey: GameScriptVisualStyleKey;
  readonly coverImage: string;
  readonly dayBackground: string;
  readonly nightBackground: string;
  readonly colors: {
    readonly ink: string;
    readonly paper: string;
    readonly accent: string;
    readonly signal: string;
    readonly night: string;
  };
};

type GameScriptDefinition = {
  readonly id: string;
  readonly name: string;
  readonly theme: string;
  readonly background: string;
  readonly atmosphere: readonly string[];
  readonly presentation: GameScriptPresentation;
  readonly enabled: boolean;
  readonly createdAt: string;
  readonly updatedAt: string;
};

type GameScriptSnapshot = {
  readonly scriptSourceId: string;
  readonly name: string;
  readonly theme: string;
  readonly background: string;
  readonly atmosphere: readonly string[];
  readonly presentation: GameScriptPresentation;
};
```

Rules:

- `validateGameScriptDefinitions(unknown)` owns unknown-input validation, unique IDs, safe internal asset paths, non-empty narrative fields, allowed style keys, valid `#RRGGBB` colors and ISO timestamps.
- `createGameScriptSnapshot(definition)` performs a deep clone. Existing games never reread current script wording/tokens.
- `legacyGameScriptSnapshot()` is a code-owned compatibility snapshot using current legacy assets and `legacy_v1`. It is used only when loading a historical Game without `script`; it is not exposed in New Game.
- Asset files use immutable, versioned filenames. Snapshotting a path is sufficient only if bytes behind that path are never replaced; revised art receives a new filename.

### 2.2 Library and seed ownership

- Add `kivdb/scripts.json` as the runtime script library and `src/seeds/scripts.ts` as its validated seed projection.
- Extend `LibraryRecord`/`LibraryRepository` with `scripts`, `getScripts`, `saveScripts`, and batch handling.
- First definition: `midnight_archive`, name `未明档案`.
- No script-authoring Library UI in this task. The library is validated data consumed by New Game. This avoids building a generic theme editor before a second real script proves the contract.
- Character and presenter JSON remain the runtime source. `src/seeds/characters.ts` should import and validate `kivdb/characters.json`, matching the existing presenter pattern, so permanent biographies and voice values have one owner.

### 2.3 Game snapshot and compatibility

`Game` gains a required `script: GameScriptSnapshot`. All new creation paths require an enabled `scriptId`, resolve it once, and store the snapshot.

`GameRepository.normalizeRecord` performs compatibility normalization:

1. validate an existing script snapshot;
2. if absent, inject `legacyGameScriptSnapshot()`;
3. retain the existing presenter/rules/player normalization.

No events or old players are rewritten. Historical presenter snapshots remain readable even after the presenter library is reduced to one definition.

### 2.4 Presenter selection

- Presenter library contains one enabled definition: `wen_zhou` / `闻舟`.
- New Game no longer asks the user to select a presenter. Creation resolves the sole enabled presenter and fails clearly if the library has zero or more than one enabled presenter.
- Existing `Game.presenter` remains a complete snapshot and continues to own its exhaustive semantic line catalog.
- Script does not override presenter lines in v1. 闻舟的措辞设计为可跨剧本使用；剧本背景由标题、画面和玩家上下文承担。这样不会在尚无真实需求时，把不可变的主理人语音清单按剧本成倍扩张。

## 3. Permanent cast bible

All portraits share a semi-realistic Chinese graphic-novel style: square head-and-shoulders composition, modern urban-gothic wardrobe, subtle paper grain, restrained archive geometry, face centered in the 60% safe area, strong rim light, no text, no role/faction symbols, and no scene-specific clues. Each character owns one dominant accent visible at 120 px.

| ID / name | Narrative function | Visual anchor | Speech rhythm | Reasoning entrance | Constructive limitation |
|---|---|---|---|---|---|
| `qin_chuan` 秦川 | High-IQ protagonist / global model | Black high-collar coat, deep teal rim, one silver temple streak, thin metal glasses | Slow, compressed, conclusion after conditions | Rebuilds the whole table as hypotheses and information flow | Can over-model sparse evidence; must mark uncertainty and can be wrong |
| `qiao_ke` 乔可 | Audience proxy / honest reaction | Amber bob, mustard raincoat, round expressive eyes | Short, quick questions; openly admits confusion | Notices emotional discontinuity and asks others to explain plainly | Short logic chain; intuition is a lead, not a fact |
| `zhou_xu` 周序 | Fact ledger / continuity keeper | Square glasses, olive utility vest, pocket notebook | Numbered, clipped, low emotion | Votes, order, confirmed facts, impossible combinations | Weak at reading performance and spontaneous motive |
| `xia_mi` 夏弥 | Social ignition / interaction driver | Short curls, cyan scarf, forward-leaning posture | Fast, warm, calls names directly | Silence, response latency, willingness to take a position | Can create noise and overvalue immediate reactions |
| `ren_ye` 任野 | Survivor / pressure sensor | Broad frame, stubble, burgundy work jacket | Guarded, blunt, starts from self-protection | Who applies pressure, redirects blame, or seeks a scapegoat | Defensive posture may distort neutral pressure into hostility |
| `gu_ling` 顾绫 | Interrogator / conflict engine | Asymmetric black bob, vermilion tailored coat, sharp brow | Low, decisive questions; demands one answer | Contradictions, stance changes, pressure tests | Can force false binaries and mistake composure for honesty |
| `cheng_wu` 程雾 | Risk controller / boundary keeper | Gray-blue braid, navy coat, calm mature silhouette | Slow, precise, explicitly labels certainty | Expected loss, information protection, reversible decisions | Conservative choices may preserve uncertainty too long |
| `ye_chen` 叶忱 | Devil's advocate / consensus breaker | Pale blond crop, ivory-and-black coat, half-smile | Dry, compact counterexamples | Attacks comfortable majority narratives and hidden premises | Opposition is a method, not proof that consensus is wrong |
| `chi_mu` 迟木 | Quiet observer / late reveal | Long fringe, muted violet hoodie, camera strap | Very short, high information density | Tiny wording changes, sudden pivots, details others skip | Low participation can leave useful observations too late |
| `tang_li` 唐梨 | Comic probe / tension release | Copper twin knots, persimmon jacket, playful eyes | Light, teasing, sudden direct follow-up | Humor, pauses, and defensive reactions under a joke | Must not turn every serious scene into a bit |
| `lu_ran` 陆燃 | Charger / forced alignment | Athletic build, shaved sides, burnt-orange jacket | Loud, fast, binary calls to action | Avoidance, follow behavior, willingness to commit | Speed can harden a weak first read |
| `su_xian` 苏弦 | Relationship mapper / social synthesis | Dark green blouse, jade earrings, open posture | Calm restatement followed by one relationship claim | Mutual defense, attacks, distancing, conversational sequence | Harmony bias can soften necessary confrontation |

Permanent friction axes create watchable scenes without encoding alliances:

- 秦川 models the table; 叶忱 attacks the model's hidden assumptions.
- 顾绫 increases pressure; 任野 exposes how pressure can manufacture behavior.
- 夏弥 pulls people into the exchange; 迟木 resists volume and contributes selectively.
- 周序 trusts recorded structure; 唐梨 trusts live reactions.
- 陆燃 values decisive movement; 程雾 values controlled downside.
- 乔可 voices the viewer's confusion; 苏弦 turns scattered emotion into a relationship map.

### 3.1 Voice strategy

- Replace the two-voice gender split with a supported Chinese neural-voice palette, then use restrained rate/pitch/volume offsets.
- At least six base voices must be used across the 12 players; no adjacent default preset seats share the same base voice.
- Rhythm follows the cast bible: speed and loudness may emphasize function, but all voices remain natural and subtitle timing continues to use real word boundaries.
- Implementation must synthesize one short smoke line per distinct base voice before locking definitions; unsupported voices fall back to a verified voice, not silent runtime failure.
- Player audio remains snapshotted per confirmed event, so changing library voices does not mutate historical games.

### 3.2 LLM binding ownership

- Each fixed Character Definition owns its `provider`, `model`, JSON response format, and optional fallback model.
- Game creation copies only the Character binding into `Player.modelBindingSnapshot`; rule roles and preset seats do not select or override an LLM.
- `temperature` and `maxTokens` are not character configuration. OpenAI-compatible requests omit both fields and use provider/model defaults.
- Token usage views include a non-persisted cost calculator. Default prices are ¥6 per million input tokens and ¥30 per million output tokens; input billing combines Prompt + Reasoning, while output billing uses Completion. Users may edit either price for immediate local recalculation.

## 4. Presenter bible

`wen_zhou` / 闻舟 is visually separate from the cast:

- late-40s male silhouette, swept silver-black hair, charcoal Zhongshan-collar coat, thin gold half-rim glasses, one small vermilion seal pin;
- neutral front three-quarter pose, low-key warm key light plus cyan archive rim;
- measured low voice, brief pauses, no melodramatic growl;
- copy uses page-turn verbs and concrete outcomes, not repeated generic suspense phrases;
- never interprets a player's motive or tells the audience who looks suspicious.

Presenter portrait lives under `/kivdb-assets/presenters/`. Player portraits stay under `/kivdb-assets/characters/`. Export snapshot resolution accepts both allow-listed internal prefixes and copies them to immutable job assets.

## 5. `未明档案` script

### 5.1 Narrative

- **Theme:** 一场发生在城市档案馆闭馆后的深夜推理记录。
- **Background:** 十二名记录参与者在封存区完成一轮公开推理。停电、雨声和旧档案只是共同氛围；所有狼人杀事实仍来自事件日志。
- **Atmosphere:** `冷雨`, `停电余光`, `旧纸`, `封存编号`, `红色校印`, `凌晨记录`.

The prompt renders this as a dedicated section after scene/task requirements:

```text
【本局剧本背景——只用于自然表达，不是身份事实】
- 剧本：未明档案
- 共同背景：...
- 氛围：冷雨、停电余光、...
- 可以偶尔使用背景中的意象或类比，但不必复述剧本名称。
- 剧本不提供任何玩家身份、行为、关系或可信度证据。
```

This section appears in both speech and action requests. Action `decisionSummary` may mention the atmosphere only when it clarifies risk, but legal selection remains driven by game state.

### 5.2 Visual system

- Dominant ink: near-black green.
- Paper: warm old-paper ivory.
- Accent: oxidized cyan.
- Signal: vermilion red.
- Rules: thin archival lines, clipped case numbers, stamped state changes, asymmetric editorial spacing.
- Avoid: fake CCTV noise over portraits, unreadable microcopy, persistent dark overlays above supplied background art, or role/faction symbolism embedded in character art.

New immutable assets:

- `script_midnight_archive_cover_v1.png`
- `script_midnight_archive_day_v1.png`
- `script_midnight_archive_night_v1.png`
- 12 versioned player portraits and `presenter_wen_zhou_v1.png`.

### 5.3 Application surfaces

- **New Game:** script choice is the first content block. Card shows cover, name, theme, background excerpt, and atmosphere tags. With one enabled script it is selected by default but remains an explicit field.
- **Home cards:** cover strip, script name, and accent line; the rest of the home dashboard stays a stable tool surface.
- **Editor:** retains the stable global tool theme. Its game header sits above the right-hand Draft panel so the left Preview/Timeline column can use the full viewport height. When Preview is visible, the left column uses a restrained 56:44 Preview/Timeline height split and scales the 16:9 iframe inside its cell, giving Timeline more working space without cropping Preview. Script presentation must not alter its backdrop, header, labels, editing semantics, or density.
- **Preview/video:** the shared 1920x1080 React composition consumes script presentation. Background assets remain unobscured per the programmatic-video spec. Major surfaces, rules, typography hierarchy, stage accents and event stamps use `midnight_archive_v1` tokens.
- **Library:** role, character, presenter and preset management retain the global application theme.

## 6. Data flow and ownership

### 6.1 Create game

```text
NewGameDialog(scriptId, preset/random seats)
  -> server action
  -> LibraryRepository.getAll()
  -> require one enabled presenter + selected enabled script
  -> createGameFromPreset(..., presenter, script)
  -> Game { presenter snapshot, script snapshot, player snapshots }
  -> GameRepository.save()
```

The same selected `Game.script` drives every downstream consumer. Components never reload scripts by ID for an existing game.

### 6.2 Prompt

```text
Game.script snapshot
  -> buildPlayerLlmContext().script
  -> prompt-builders semantic script section
  -> model speech/action
```

The core prompt builder owns the wording and safety label. UI code does not concatenate background prose into prompts.

### 6.3 Preview and export

```text
Game.script snapshot
  -> preview page / export snapshot builder
  -> createCompositionInput({ items, script })
  -> VideoCompositionInput.presentation + assets
  -> createHtmlFrameViewModel
  -> HtmlPlaybackStage
```

Preview and Remotion continue to share one stage component. Script style is input data, not viewport state. Export copies script background assets and both avatar-prefix families into the job directory, then rewrites URLs once.

### 6.4 Script Author Agent v3

Script Author 是一个由应用层持久化状态驱动的逻辑 Agent，不是一次巨型 Prompt，也不是把所有历史消息不断追加的聊天会话：

```text
compileEpisodePlan(Game)
  -> EpisodeAuthorWorkspace(inputHash, dedicated model binding)
  -> story
  -> compact ensemble assignments
  -> character × 12
  -> relationship × selected pairs
  -> local scene beats (<= 5 speech steps)
  -> deterministic assembly + dramaturgy validation
  -> EpisodeScriptSnapshot schema v2
```

- `EpisodeAuthorWorkspace` 是恢复真相源，包含故事、群像分工、已完成角色弧线、关系弧线和节拍；请求日志只负责审计，不用于猜测下一任务。
- 每个子任务从工作区投影最小上下文，并在成功或失败后通过 `onCheckpoint` 在 Game lock 内落盘。相同 input hash 的 `generating`/`failed` 状态从首个缺口继续。
- 工作区增长有硬边界：故事 1–4 幕、群像 1–6 对关系、所有创作字符串不超过 80 字符、单个场景最多 5 个发言节拍，并且每名当前演员只携带 1 个先前 move。
- Script Author 的 model binding 独立于 Player 快照。固定角色的模型只负责该角色在运行时的行动和最终台词。
- provider 在 JSON 解析前提供的 `finishReason`/usage 必须被保存。`length` 截断走任务级降级：场景节拍递归拆分，其他小任务仅从原始紧凑输入再生成一次；截断文本不进入修复 Prompt。
- 完整但 schema 错误的 JSON 仍可做一次最小结构修复。这样把“输出没写完”和“输出写完但形状错了”分成两个根因，不使用同一种补丁。
- Author 中间态不做兼容：只接受 `episode-author:v3` 的 task/workspace/request。最终 Episode 快照继续使用 schema v2，因为其锁定的是合法执行结构，而不是 Agent 实现细节。
- Script 页面读取落盘 workspace 展示当前阶段和进度；失败重试保留所有历史成本与已经完成的创作结果。

运行时玩家台词生成仍使用当前 Actor Brief + 可见事实链。本阶段不重构它，但后续重构应沿用同一原则：按局部目标拆任务、把工作状态与模型对话分离、按 provider 失败原因选择恢复策略。

## 7. Migration and rollback

- Old Game records: inject `legacy_v1` snapshot at read time; do not rewrite files eagerly.
- Old exports: retained and untouched.
- Old presenter directories/manifests: may remain on disk for retained exports and old games, but `presenters.json` exposes only 闻舟. Runtime paths continue to load old presenter manifests by snapshot source ID when an old game is replayed.
- New library data can be rolled back by restoring JSON files and assets. New Game records remain readable because they carry snapshots; removing `midnight_archive_v1` renderer is not a valid rollback while such records exist.
- Visual renderer dispatch must retain `legacy_v1` and `midnight_archive_v1` exhaustively.

## 8. Explicit non-goals

- No generic no-code theme editor.
- No per-script character variants, relationships, secrets, or role bindings.
- No independent game-summary or narrator LLM chain outside the bounded Script Author Agent.
- No face animation, lip sync, 3D stage, or sound-effects/music system.
- No implementation of the B/C visual directions in this task.
