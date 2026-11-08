# 实施顺序
1. root写清参考、PRD和架构边界并向用户说明；用户已明确授权方案后立即实现，不再等待新的流程确认。
2. HUD代理负责index.html、web样式/展示模块，读参考图与完整规范；保留输入/启动协议，先用轻量DOM布局检查。
3. 镜头代理负责src/client/camera.rs和ui.rs（取景与只读footprint），必要的native测试；与HUD约定camera_footprint字段，不碰Web或模拟。
4. root负责浏览器断言迁移/新布局与取景检查、文档和集成，GPU串行。先定向native和JS检查，再CARGO_BUILD_JOBS=2 make check与标准WASM Clippy（不加all-targets），构建前冻结运行时源，避免重复长构建。
5. 独立Trellis全范围检查；最终截图目检1920/1280/800/DPR2、实际鼠标操作/血条/恢复与重选；根据结果做局部修正后只重跑相关检查。
6. 规范/README/TODO及验证记录同步；HUD不自动提交或归档，用户自行管理开发服务。
