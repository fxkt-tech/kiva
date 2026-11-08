# 分类设置与运行/输入边界

## 所有权

`web/settings.js`只拥有设置弹窗和分类：操作（cast-mode）、声音（audio-muted/audio-volume）、角色（直接列表/预览/确认）。不保留对局分类或装备入口，装备走T。控件ID各一份，切换分类不销毁或复制控件，本地偏好仍由casting.js/audio.js唯一拥有。样式集中settings.css。

工厂`createSettingsUi({requestClose,onToggle})`返回open/close/isOpen/select/setBusy/setError。open先同步`data-settings-open=true`，close才清；“完成”只请求父级关闭，不能提前隐藏或恢复游戏输入。分类支持键盘导航和ARIA选中状态。

hud.js唯一拥有Esc、暂停、捕获与会话。Esc幂等打开设置，避免浏览器先失锁后发keydown导致同一次Esc关掉新弹窗；完成/设置按钮在可信手势内重新捕获，事件确认后才关闭。Active不再显示旧entry，entry只用于选角；选角设置关闭不请求捕获。

## 模拟与输入

controls/access.rs区分Access.paused与Access.gameplay，GameplayInput资源由controls写、camera读。
Active且对局可继续、文档聚焦、无暂停请求即可推进；捕获不再作为模拟暂停条件。设置打开或捕获未确认会阻断人类输入，清瞄准、修饰键和旧按键/点击/技能意图，不发Stop、不清已接受订单。世界中的移动、攻击、AI、波次、冷却继续。

实际解锁可能早于pointerlockchange，期间同样只屏蔽输入，不能插入一帧暂停/Stop。真实失焦或显式暂停仍冻结；Selecting/终局冻结。Native继续原Esc暂停/恢复，屏蔽普通游戏事件时保留这一个原生控制入口。

camera消费并丢弃屏蔽期间的地图请求和滚轮，屏蔽手动键鼠；已锁定镜头可继续随角色移动。Pointer Lock坐标、实际捕获和事件确认仍由原指针/会话所有者维护；关闭后原just_captured边界清旧输入并居中。

## 生命周期

聚焦的Active失锁打开设置，模拟继续；blur仍暂停并显示设置。设置打开不写暂停值；已因失焦暂停的局保持暂停，完成并成功捕获后恢复。捕获失败保留设置/当前暂停状态并提示，迟到回调按原attempt身份隔离。

按T打开独立装备目录时可临时隐藏设置展示但保留settings-open门控，使底部库存可操作；关闭目录恢复设置。设置不提供重复装备按钮。

角色列表/预览使用同一#character-select，初始选角与设置分别由宿主容器移动该节点，不复制ID/渲染器。character_catalog只在首份完整快照发送，characters.js在WASM前缓存，迟到HUD仍能取得；预览与Selecting.selected共用Rust CharacterPreview构造，Active.selection保持null。
Active角色草稿只属于UI，浏览不发ReturnToSelection或改世界；确认才冻结match/player/character，捕获事件确认后提交ChangeCharacter。后台校验当前身份、设置边界、聚焦/确认捕获和角色；与RestartMatch共用recreate_match，成功后等新玩家投影才关闭设置。拒绝保留预览并提示重试。同角色确认也重开，旧批次尾部不能影响新局。
ChangeCharacter成功入队时同步data-paused=false，但settings-open继续true屏蔽输入；否则从结算/真实暂停换角会把旧DOM暂停带进新局并释放刚取得的锁。Selecting在设置角色页启动时，选择/捕获错误须显示在角色页/设置错误区，不能只写隐藏的旧entry。
结算的重新选角入口直接打开角色分类，完成可回结果页，确认后重开。普通终局关闭浏览中的设置；已有ChangingCharacter确认可继续按冻结选择处理。

设置打开且运行、聚焦时音频可在未捕获状态继续播放；静音/音量实时走原设备。本地设置不影响模拟；真正暂停/失焦/离开会话继续清声音。

## 验证

Rust覆盖策略矩阵、解锁早于通知、移动/订单/tick不受设置影响、手动相机隔离、原生Esc恢复、设置中重选与旧边界拒绝。
Node实际加载settings.js/hud.js，覆盖分类隔离和原值保留、捕获确认/拒绝/迟到、失焦、选角/终局清理；audio专项覆盖设置中真实播放与音量/静音绑定。
浏览器长战斗fixture需冻结时使用现有data-paused暂停意图，不能再把Esc当暂停；普通运行中的设置行为与冻结观察分开验证。
