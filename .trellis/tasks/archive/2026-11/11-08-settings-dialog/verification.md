# 验证记录

## 实现

- 新settings.js/settings.css拥有分类与弹窗；操作、声音、对局各自面板，原施法/静音/音量/装备/重选控件各一份。
- hud.js继续唯一协调捕获/暂停/会话。Esc幂等打开；关闭请求等事件确认后才清设置标记，拒绝保持弹窗与原运行态。选角设置无需捕获关闭；真实失焦暂停，迟到捕获不擅自恢复；终局显示结果，重选回选角。
- controls/access.rs将模拟暂停与GameplayInput拆开，未确认/丢捕获仅屏蔽输入；设置不下Stop，已有订单和tick继续，输入队列/修饰键清理。Native保留原Esc恢复入口。
- camera屏蔽手动输入、丢弃旧滚轮/地图请求，可继续已锁定跟随；声音在运行中的设置继续播放并实时响应原音量/静音绑定。
- Session严格身份/捕获校验保留，允许运行中设置边界发起原整场重选。上轮T装备弹窗与运行中装卸保留。

## 普通质量检查

- `CARGO_BUILD_JOBS=2 make check`通过：格式、原生Clippy、419项Rust测试、WASM编译；0失败，2项既有soak忽略。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- `node --test scripts/*-unit.test.cjs`：70通过、0失败。新增6项设置组件、9项实际hud.js生命周期、1项设置音频回归；原装备/指针/启动/偏好回归保留。
- 修改JS/CJS语法检查通过；diff空白、任务上下文校验通过。
- Rust覆盖设置/早解锁策略矩阵、已有移动/订单/tick继续、手动镜头隔离、运行中重选、原生Esc暂停后恢复。
- Node覆盖分类独立与原值保留、关闭等待事件而非Promise、捕获拒绝/迟到/失焦、Selecting和终局清理、设置期间声音与音量/静音实际绑定。

日志：`/tmp/kiva-settings-check.log`、`/tmp/kiva-settings-wasm.log`、`/tmp/kiva-settings-node.log`、`/tmp/kiva-settings-ui-node.log`。

## 验证边界

本轮未运行浏览器、GPU或窗口截图，也未启动/关闭用户服务。浏览器夹具已同步分类入口、关闭按钮和显式暂停观察，不将这些未执行脚本当作真实浏览器验收。初次会话测试发现独立夹具缺新GameplayInput资源，已补齐；输入审查发现Native暂停会误丢Esc恢复，已修复并保留真实回归，最终质量检查通过。

没有提交或推送；代码、规范、README和架构文档同步，任务保留待后续提交关联。

## 提交交付

用户随后授权提交与推送，功能已提交为50d80a3；以上验证为提交前记录。归档与会话日志单独记录交付。
