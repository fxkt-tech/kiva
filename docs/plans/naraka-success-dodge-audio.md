# 永劫 PVP 成功闪避音效调查

调查日期：2026-11-12。原客户端及音频包全程只读。用户最终选定 `move_soul_dodge_1p`，已作为 Kiva 成功闪避提示接入；原作 PVP 对应音效未确证。

## 已核对

- 本机原始 WwiseBanks 表的完整字符串键索引：1125 个 bank、51088 个事件，另有 13 个本地化和 3 个 SwitchGroup 记录。所有索引、记录键、长度与表边界完整消费校验通过。
- 原表解码 SHA-256：`0fbb905eb229ebe9e93a37cdfabd9d7e78755256fa0d2c5b5afec542d4b11f75`，与既有冻结证据一致。
- 从原 SoundActor1/2.pck 验证 Event → PlayAction → RandomContainer/Sound → DIDX → 完整 WEM 媒体链，使用既有固定 SHA 的 vgmstream-r2168 解码，不裁剪、拼接或变速。
- 原 ActorEffectConfig_ActorEffect 的 10176 记录标注“完美闪避粒子 / 完美闪避”；记录本身未包含 Wwise 事件名。不能据此认定某个声音是通用成功闪避提示。

## 候选及已证实的范围

| 原事件 | 媒体 | 已证实范围 |
| --- | --- | --- |
| `gman_dodge_01_1p` | 919868544、828546990、364824874、1013417629、384048418、835672119 | 四方向闪避动作轨的起手声，Kiva 已使用；不能当作成功判定专属声 |
| `gman_fx_battle_bullet_dodge_1p` | 777438309、235352018、216649760、221963326 | 名称指向子弹擦身；四个原始等权变体，约 0.85–0.95 秒；尚未证明用于 PVP 躲避近战 |
| `gman_jiguansan_dun_perfect_dodge_attack_light_01_1p` | 1007909166 | 机关伞盾形态的完美闪避后攻击，约 1.85 秒；不能当作通用闪避成功提示 |

## 仍待确认

用户目标是 **PVP 躲过攻击时的声音**，不是普通闪避起手或 PVE 武器反击。
尚未建立该场景与某个独立原始音效事件之间的确证连接；不能声称原游戏不存在该音效，也不能把以上候选直接冒充目标音效。

候选 WAV、原 WEM、媒体哈希、bank 哈希、原事件容器链和匹配配置记录保存在本机研究目录：
`/home/asuka/kiva-asset-research/naraka/success-dodge-2026-11-12/`。
`candidates.json` 保存完整提取链；`matched-config.json` 保存相关原配置记录；`wwise-event-index.json` 保存完整原表索引。

## 扩大筛选与试听页面

用户已否定上述子弹擦身和机关伞反击候选，并要求先集中试听。
新增检查通用接触、攻击限制提示、生命提示、金色攻击结束提示、武器碰撞／弹开／振刀和身体／刀剑／子弹擦碰的默认分支。
从 22 个原始事件提取并按 source ID 合并得到 59 段完整音频，没有拼接、裁剪或变速。全部媒体仍通过 Event → Action → Container/Sound → DIDX → WEM 验证，并使用固定解码器转出 Ogg。

仓库根目录的 [audio-preview.html](../../audio-preview.html) 内嵌全部音频，可用浏览器直接打开；默认展示 54 段，用户已否定的 5 个媒体变体默认隐藏。
支持原事件／媒体 ID／固定 N 编号搜索、分类、逐条或连续试听、候选标记和编号复制。标记尝试保存在浏览器本地，存储不可用时仍可在当前页面使用。

新增原媒体、解码指标、各层权重和 bank 来源保存在研究目录 `audition/manifest.json`，原 WEM/Ogg 同目录保存。
浏览器离线测试覆盖 59 段完整解码、搜索／分类、收藏持久化、播放／停止／连续切换，以及桌面／手机布局。

等待用户在试听页确认编号后，再冻结对应来源并接入实际 CombatEffect::Dodge 成功事件。Kiva 当前仍只播放普通闪避起手声。


## 第二批试听

第一批全部被用户排除。试听页已替换为第二批，编号使用独立的 `B001` 起始序列，浏览器标记也与第一批隔离。

- 筛选范围扩展到特殊脱身、拳刃闪避、状态／战斗提示、反制挥击及角色专用闪避。
- 从 32 个可验证事件获得 44 段全新音频，约 85 秒。按媒体 ID、原 WEM 哈希及解码 PCM 哈希排除上一批，并去除本批相同解码内容；未裁剪、拼接或变速。
- `gman_all_debuff_wind` 的 PlayAction 属性不在当前已验证解析范围内，明确跳过，没有猜测解析。
- 原媒体、bank 来源、媒体链、排除项与解析失败项记录在本机研究目录 `audition-batch-2/manifest.json`。第一批独立 HTML 归档到研究目录 `audio-preview-batch-1.html`。
- 当前 [audio-preview.html](../../audio-preview.html) 仅包含第二批音频；仍需用户试听确认，未接入游戏成功闪避事件。


## 第三批：端游近战／蓄力场景

用户确认前两批均不匹配，目标来自电脑端游，出现在躲过近战／蓄力攻击时，暂不记得音色。

- 重点改查闪步／后撤、近战挥击和蓄力释放、防御与震荡反馈。原媒体 ID、WEM 哈希及解码 PCM 均与前两批去重；编号独立使用 `C` 前缀。
- 通过原资源索引定位 `fx_battle_wanmeishanbi.prefab` 和 `fx_battle_wanmeishanbi_plus.prefab`，读取根对象和关联组件。它们的 MonoBehaviour ManagedReference 无法被现有 UnityPy 完整解析，已记录错误和原字节，不声称完成原类型树解析或证明不存在声音。
- `fx_buff_dodge_hit.prefab` 及其 PVE 变体的原组件字节里明确出现 `AudioEvent` 和 `action_scene_yinlei`，将该原事件作为用途待确认的优先试听线索；这仍不证明它是用户所指的 PVP 通用成功闪避音效。
- 组件来源、原字节和字符串证据保存在研究目录 `perfect-dodge-prefabs.json`；第三批完整媒体链及去重记录在 `audition-batch-3/manifest.json`。
- 参考官方 [2024-08-29 更新公告](https://www.narakathegame.com/news/update/20240820/33459_1175601.html) 的 Combat Adjustments：增加成功躲过敌方攻击的特效提示。该文字仅证明视觉反馈变更，不能据此推断存在独立音效。

当前试听页更新为第三批，第二批 HTML 归档至研究目录 `audio-preview-batch-2.html`。仍未修改游戏的成功闪避音效接线。


## 最终选择与接入

用户结束继续查找，指定使用第二批的 `move_soul_dodge_1p`。

- 游戏 cue 为 `naraka_dodge_success`，优先级 3，由玩家实际 `CombatEffect::Dodge` 结算触发。普通闪避起手仍由独立的 `MotionKind::Dodge` 播放 `naraka_dodge`。
- 原 `move_soul` bank 来自 `SoundActor1.pck`；三个等权媒体为 `56972138`、`71696684`、`374972779`。完整 WEM、无损 PCM、事件／容器链和来源哈希进入既有冻结包，运行时 Ogg 使用同一可复现导入器生成。
- 保留自然音高、完整时长和每层原始权重，不裁剪或拼接。使用音效基础增益 0.45，BGM 保留 0.18。
- CombatFeedback 游标保证同一事件只消费一次；暂停、失焦取消和非玩家闪避不会在恢复后补播成功提示。声音由结果事件驱动，不通过播放声音推断游戏规则。
- 这属于用户选定的 Kiva 用途适配，不声称恢复了原作 PVP 的成功闪避音效或触发关系。
