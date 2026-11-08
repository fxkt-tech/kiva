# 普通与智能施法

## 输入所有权

`controls/casting.rs`拥有CastMode、快捷键映射与按事件更新的Shift状态。
默认Quick；Normal按键进入既有Aim，再左键确认。Shift+QWER/1–6临时Normal。
鼠标技能/装备图标始终走普通选择；SelfCast和合法重施不需要目标确认。

`web/casting.js`在WASM前读取`kiva-cast-mode`本地偏好，向根发布封闭的
`data-cast-mode=quick|normal`，DOMContentLoaded后绑定`#cast-mode`菜单。
未知值回退Quick；存储不可用仍可当场切换。Rust只镜像客户端模式，模拟不读取DOM或按键。
模式跨暂停/重选保留；修饰键在暂停、失焦、无玩家和捕获边界清理。

## 统一施法路径

普通左键与智能keydown共用`cast_at_pointer`，后者复用
`ability_targeting::resolve_intent`与`submit_cast`，不另算资格或最终终点。
指向技能只用实际模型候选，允许OutOfRange交模拟走近；方向/地点沿用原规则。
智能模式无合法目标不提交、不残留瞄准；UI覆盖/手势不产生世界技能。
冷却、费用、生命、装备实例和提交期检查仍属于模拟；按键不跳过前摇。

键盘读取有序WindowEvent，忽略repeat。Web消费pointer.js保存的按键时刻坐标、
UI手势归属与shiftKey，再更新GamePointer镜像/模型拾取；不能用上一帧PointerState
或帧末的最新鼠标位置。两键同帧分别使用各自快照，详见[pointer.md](pointer.md)。
原生平台按CursorMoved/KeyboardInput顺序与左右Shift状态处理。

## 验证

controls无窗口场景覆盖两模式、Shift按下/释放同帧、按键后鼠标移动、重复键、
超距Enemy追近、地点截断、英雄/装备及图标入口；web解码验证有限坐标和封闭标记。
`node --test scripts/casting-unit.test.cjs scripts/pointer-unit.test.cjs`验证真实偏好脚本
和事件快照。既有browser-support fixture明确从真实菜单选Normal，不依赖用户默认偏好。
