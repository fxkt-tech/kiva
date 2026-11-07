# 模拟层规范

适用：`src/simulation/` 及无窗口规则测试。

## Pre-Development Checklist

1. 阅读 [runtime.md](runtime.md) 和 [docs/architecture.md](../../../docs/architecture.md)。
2. 从 `src/simulation/mod.rs` 查看公开接口、构造入口和固定步系统顺序。
3. 为变动明确唯一写入者、指令拒绝行为、提交时机、死亡/重生行为。
4. 查阅 `tests/simulation.rs` 的 `Arena`，优先通过公开接口描述可观察结果。
5. 涉及属性、伤害和攻速时阅读 [attributes-and-damage.md](attributes-and-damage.md)。
6. 涉及技能、控制、生命身份或强制运动时阅读 [abilities-and-statuses.md](abilities-and-statuses.md)。
7. 涉及角色定义、创建或装配时阅读 [characters.md](characters.md)。

## Quality Check

- `cargo test --locked --test simulation` 验证模拟行为；`make check` 验证整个 crate 及 WASM。
- 改动系统顺序、伤害或动作时机时，检查同一步提交/死亡、取消施法、走砍、暂停和重生回归。
- 整场清理以 `tests/reset.rs` 验证实体/待结算/事件清除、静态内容保留及身份分配器不复用。
- 模拟不读取浏览器、相机、设备输入或显示 Transform；客户端不能直接写入模拟运行态。
