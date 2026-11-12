# 执行计划：攻击判定统一

按子任务顺序执行，每个子任务单独验证、单独提交（提交需用户授权）。

1. `11-12-anim-sim-clock` → 见其 implement.md
2. `11-12-melee-sweep` → 依赖 1
3. `11-12-projectiles` → 依赖 1，可在 2 之后或并行

## 集成检查（全部子任务完成后）

- [ ] `make check`
- [ ] `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`
- [ ] `CARGO_NET_OFFLINE=true trunk build --dist <临时目录>`，并确认 Cargo.lock 未变化
- [ ] `make bake-hits` 之后 `git diff --exit-code src/simulation/combat/hit_tracks.rs`（烘焙稳定）
- [ ] 规范：combat-v1.md（判定、弹体、Boss 契约）、simulation/index.md、client/combat-presentation.md、client/naraka-motion.md、docs/architecture.md（固定步顺序与新模块）
- [ ] 交给用户试玩验收：刀与命中点一致、Boss 出招可读、远程手感
