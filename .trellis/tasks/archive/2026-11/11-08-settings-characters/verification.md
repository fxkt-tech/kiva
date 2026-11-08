# 设置内角色选择验证

## 结果

设置分类为操作/声音/角色；角色列表和预览直接内嵌，移除对局分类、调整装备及跳转选角按钮。装备继续通过T/原装备按钮。初始选择和设置移动同一character-select节点，不复制选择器/渲染器；预览只改本地草稿，原局继续。

character_catalog由Rust统一CharacterPreview构造，首份成功完整快照发布；publisher的Local Publication拥有静态目录发送状态，characters.js在WASM之前缓存。Active.selection仍为null，后续不重复发送静态目录。

确认冻结match/player/character，捕获事件确认才发ChangeCharacter；成功绑定新对局/玩家投影才关设置。旧身份、未知角色、非设置或未确认捕获都先拒绝；RestartMatch/ChangeCharacter共用recreate_match，统一旧局清理、版本递增和失败Selecting回退。新玩家替换丢弃旧批次尾部。

结算选角同样打开角色页，完成可取消回结果；已冻结的确认不会因旧局刚结束而丢掉。确认入队清除旧DOM暂停但保留设置门控，避免新局被旧暂停再次冻结。Selecting在角色设置启动的捕获/选择错误显示于当前可见表面。

## 检查

- `CARGO_BUILD_JOBS=2 make check`：格式、原生Clippy、422项Rust、WASM编译全部通过；0失败，2项既有soak忽略。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- `node --test scripts/*-unit.test.cjs`：79通过、0失败。相关32项覆盖组件、装备和真实HUD生命周期；本轮新增目录早到、草稿不提交、共享节点、冻结角色/身份、捕获/权威拒绝、结算竞态与初始错误表面。
- 修改JS/CJS语法与git diff空白检查通过；任务JSONL校验通过。
- Rust新增真实整局替换、旧实体/装备清理、版本递增、旧批次尾丢弃、非法边界原局保留，以及结算同/不同角色重开；预览测试检查目录与Selecting预览完全一致。

日志：`/tmp/kiva-settings-characters-check.log`、`/tmp/kiva-settings-characters-wasm.log`、`/tmp/kiva-settings-characters-all-node.log`。

未运行浏览器或启动/停止用户服务。旧浏览器重置用例以保留的ReturnToSelection公共意图夹具继续观察底层生命周期，不将它作为新角色UI通过证据；本轮新UI由普通Node测试验证。未提交推送。

## 提交交付

用户随后授权提交与推送，功能已提交为50d80a3；以上验证为提交前记录。归档与会话日志单独记录交付。
