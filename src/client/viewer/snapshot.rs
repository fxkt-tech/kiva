//! Small dependency-free JSON output. All text, including source part names, is escaped.
use super::super::animation::PoseTrace;
use super::{
    Scene,
    orbit::Orbit,
    session::{self, Mode, Session},
};
use std::fmt::Write;

pub(super) fn quote(value: &str) -> String {
    let mut text = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            c if c < ' ' => {
                write!(text, "\\u{:04x}", c as u32).unwrap();
            }
            c => text.push(c),
        }
    }
    text.push('"');
    text
}
fn entries(values: &[(&str, &str)]) -> String {
    values
        .iter()
        .map(|(id, label)| format!("{{\"id\":{},\"label\":{}}}", quote(id), quote(label)))
        .collect::<Vec<_>>()
        .join(",")
}
fn catalog(session: &Session) -> String {
    let actions = session
        .catalog
        .iter()
        .map(|clip| {
            format!(
                "{{\"id\":{},\"label\":{},\"mode\":{},\"group\":{},\"duration\":{}}}",
                quote(&clip.id),
                quote(&clip.label),
                quote(Mode::for_action(&clip.id).id()),
                quote(clip.group),
                clip.duration
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"appearances\":[{}],\"weapons\":[{}],\"presets\":[{}],\"actions\":[{}]}}",
        entries(&[
            ("lihuowang", "李火旺"),
            ("bailingmiao", "白灵淼"),
            ("choulingke", "愁铃客")
        ]),
        entries(&[
            ("fulangmao", "伏浪锚 · 锤子"),
            ("zhexingcuishan", "谪星摧山 · 链斧"),
            ("yinbingluo", "引冰落 · 鸟铳"),
            ("yizhiying", "羿之影 · 弓箭"),
            ("choulingke_blade", "愁铃客原长刀"),
            ("none", "无手持")
        ]),
        entries(&[
            ("brother_hammer", "哥哥 · 伏浪锚"),
            ("brother_axe", "哥哥 · 谪星摧山"),
            ("sister_musket", "妹妹 · 引冰落"),
            ("sister_bow", "妹妹 · 羿之影"),
            ("enemy_blade", "敌人 · 愁铃客原长刀")
        ]),
        actions
    )
}
pub(super) fn render(
    session: &Session,
    scene: &Scene,
    orbit: &Orbit,
    trace: &PoseTrace,
    ready: bool,
    failed: bool,
    include_catalog: bool,
) -> String {
    let trace_text = format!(
        "状态：{}\nCC0底层：{} · {:.3}s\n源层：{} · {:.3}s · 权重 {:.3}\n骨架：{}\n实际源层：{}\n组合范围：{}\n阶段：{:?} · 防御：{:?} {:.3}s · 手势：{:?} · 已打断 {}\n适配：{}\n回退：{}\n过渡 {:.3} · 蹲姿IK {:.3} · 双手IK {}",
        trace.state,
        trace.cc0_clip.as_deref().unwrap_or("混合／参考"),
        trace.cc0_seconds,
        trace.source_clip.as_deref().unwrap_or("无"),
        trace.source_seconds,
        trace.source_weight,
        trace.source_rig.unwrap_or("无"),
        trace.source_layers.as_deref().unwrap_or("无"),
        trace.source_scope.unwrap_or("CC0"),
        trace.melee_stage,
        trace.defense,
        trace.defense_seconds,
        trace.defense_gesture,
        trace.defense_interrupted,
        trace.adaptation.unwrap_or("CC0 fallback"),
        trace.fallback.unwrap_or("无"),
        trace.transition,
        trace.crouch_weight,
        if trace.companion_ik {
            "开启"
        } else {
            "关闭"
        }
    );
    let inspection = format!(
        "Kiva 模型／武器／动作检查 v1\n角色：{}\n武器：{}\n配置：{}\n模式：{}\n动作：{}\n时间：{:.6}s · 预览帧 {}（60Hz）\n镜头 yaw={:.6}, pitch={:.6}, distance={:.6}, target={:?}\n{}\n{}",
        session::appearance_id(session.appearance),
        session.weapon.id(),
        session.preset(),
        session.mode.id(),
        session.action,
        session.sample_seconds(),
        session.frame(),
        orbit.yaw,
        orbit.pitch,
        orbit.distance,
        orbit.target.to_array(),
        trace_text,
        session.warning()
    );
    let parts = scene
        .parts
        .iter()
        .map(|part| {
            format!(
                "{{\"id\":{},\"label\":{},\"visible\":{}}}",
                quote(&part.id),
                quote(&part.label),
                part.visible
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let status = if failed {
        "模型画面准备失败，请重新加载后重试"
    } else {
        &session.status
    };
    let mut snapshot = format!(
        "{{\"version\":1,\"revision\":{},\"ready\":{},\"failed\":{},\"appearance\":{},\"weapon\":{},\"preset\":{},\"mode\":{},\"action\":{},\"time\":{},\"duration\":{},\"frame\":{},\"playing\":{},\"speed\":{},\"looping\":{},\"trace\":{},\"warning\":{},\"status\":{},\"camera\":{{\"yaw\":{},\"pitch\":{},\"distance\":{},\"target\":[{},{},{}]}},\"debug\":{{\"grid\":{},\"axes\":{},\"bones\":{},\"grips\":{},\"bounds\":{}}},\"parts\":[{}],\"inspection\":{}",
        session.revision,
        ready,
        failed,
        quote(session::appearance_id(session.appearance)),
        quote(session.weapon.id()),
        quote(session.preset()),
        quote(session.mode.id()),
        quote(&session.action),
        session.sample_seconds(),
        session.duration,
        session.frame(),
        session.playing,
        session.speed,
        session.looping,
        quote(&trace_text),
        quote(session.warning()),
        quote(status),
        orbit.yaw,
        orbit.pitch,
        orbit.distance,
        orbit.target.x,
        orbit.target.y,
        orbit.target.z,
        session.debug.grid,
        session.debug.axes,
        session.debug.bones,
        session.debug.grips,
        session.debug.bounds,
        parts,
        quote(&inspection)
    );
    if include_catalog {
        snapshot.push_str(",\"catalog\":");
        snapshot.push_str(&catalog(session));
    }
    snapshot.push('}');
    snapshot
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_all_json_control_characters_and_source_labels() {
        assert_eq!(
            quote("a\"b\\\n\t\r\u{1}帽"),
            "\"a\\\"b\\\\\\n\\t\\r\\u0001帽\""
        );
    }
}
