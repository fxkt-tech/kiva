# 实施边界与接口

模拟层不感知设置。客户端controls仍唯一决定SimulationControl；新controls/access.rs区分暂停与GameplayInput.blocked。Active聚焦且无显式暂停即可继续模拟，捕获未确认仅阻断人类输入，避免Esc实际解锁早于通知导致一帧Stop。Selecting、真实失焦、显式暂停和终局仍冻结。

`web::settings_open()`只读根`data-settings-open`。controls同步GameplayInput资源；camera消费该资源阻断键鼠/地图意图，但可保留已锁定镜头自动跟随。屏蔽时丢弃本帧窗口、技能和指针队列及修饰键，不修改已接受订单；重捕获保留原居中与旧事件排空。

`web/settings.js`只拥有DOM分类/开关；工厂接口`createSettingsUi({requestClose,onToggle})`返回`open(),close(),isOpen(),select(category),setBusy(bool),setError(text)`。根标记在open/close同步更新。保持`#cast-mode/#audio-muted/#audio-volume/#equipment-manage/#reselect/#reselection-actions/#reselection-note`各一份，移动到分类面板。新增`#settings-toggle/#settings-dialog/#settings-close/#settings-error`；tab/panel用data-settings-tab/data-settings-panel。关闭按钮只调用父级requestClose，不能自行关掉Active设置，须等待捕获确认。

hud.js父级拥有Esc、捕获、暂停和会话：开设置先设置标记，清装备/瞄准UI，再释放捕获，不暂停；focused Active失锁通知作为Esc无keydown时兜底打开设置。真正blur仍setPaused(true)，显示设置；恢复需确认捕获。Active旧entry不显示，Selecting使用原entry。关闭成功在pointerlockchange确认后关闭设置、恢复输入；失败留在设置并回显错误。设置分类DOM不拥有游戏规则或设备。

装备从设置入口打开时仍使用现有T弹窗，可叠在设置上；底部库存需保持可达，父级syncMenu在装备打开时临时隐藏设置展示（保留settings-open输入门控），关闭后恢复设置，模拟继续。原装备onToggle沿用。设置根标记始终直到捕获恢复才关闭。

Session只扩展CaptureFacts的settings_open读事实，运行中ReturnToSelection须当前玩家、未捕获且设置打开；不经假暂停绕过门控。音频允许实际捕获或设置打开的Active运行态；真实blur/暂停继续清音频。原本地偏好不复制。

主会话拥有Rust、hud.js、audio.js及相关测试、规范与夹具；实现子代理拥有settings.js/HTML/CSS与该组件Node测试。普通检查一次完成后不重复扩张测试。
