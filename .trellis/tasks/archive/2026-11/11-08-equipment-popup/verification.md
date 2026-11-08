# 验证记录

## 跨层实现

- 装备列表与底部六格仍在Web渲染；T和关闭按钮不修改暂停/捕获，弹窗无重复库存及全屏遮罩。
- `equipment.js`只提交原Equip/Unequip协议；底部卸装读取当前实例，等待回执时阻止重复管理。目录仍保留首份静态快照缓存。
- `equipment.rs`移除暂停/未捕获门控；仍校验当前Active玩家、可继续对局、严格请求结构、FIFO、格/定义/实例，使用原核心装卸接口。
- `click_ui`根据同一输入事件的CSS位置向实际按钮分派左键或次键UI行为。右键UI不取消瞄准、不替换世界订单，不建立第二个光标所有者。
- 弹窗目录锁定滚动读取现有指针位置；窗口外继续世界操作。暂停菜单打开目录时只切换展示，暂停值保持；关闭恢复菜单。

## 普通代码检查

- `CARGO_BUILD_JOBS=2 make check`通过：格式、原生Clippy、全部Rust测试、WASM编译；413项通过，0失败，2项既有soak测试忽略。
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`通过。
- 装备适配7项回归包括真实运行中装卸后移动订单保留、tick/位置继续推进、真实基地移除导致终局后拒绝管理；输入回归覆盖次键UI不穿透或取消瞄准。
- `node --test scripts/*-unit.test.cjs`：54通过、0失败；新增8项装备UI专项覆盖T开关、P失效、暂停/捕获不变、目录缓存、空格/满格与回执、当前实例卸装、主动左键、生命周期及锁定目录滚动；已有指针/启动/施法等Node回归一起运行。
- 相关JavaScript/CJS语法、git diff空白检查、任务JSONL检查通过。

日志：`/tmp/kiva-equipment-popup-check.log`、`/tmp/kiva-equipment-popup-wasm.log`、`/tmp/kiva-equipment-popup-node.log`。

本次未运行浏览器测试，也未启停用户服务。旧浏览器夹具同步更新T、弹窗及卸装入口，尚未执行，不作为有窗口或视觉验收证据。

未提交或推送。规范、README和架构文档已同步当前行为，任务保留供后续提交关联。

## 提交交付

用户随后授权提交与推送，功能已提交为50d80a3；以上验证为提交前记录。归档与会话日志单独记录交付。
