//! Native development check: `--render-style-shots` waits for the scene to
//! settle, then captures each style plain, with the enemy's hit tint and with
//! its telegraph tint, and finally the shipped look again, into
//! `target/render-style/`, then exits. Comparing the first and last capture
//! verifies that switching back restores the original frame. Mean frame times
//! per capture window are logged; on a software adapter they only hint at
//! relative cost.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

use super::{LevelStyles, RenderStyle};
use crate::client::{EnemyBody, avatar::AvatarSurface};

pub(super) const FLAG: &str = "--render-style-shots";
const SETTLE: u32 = 240;
const STEP: u32 = 30;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tint {
    None,
    Hit,
    Telegraph,
}

const PLAN: [(RenderStyle, Tint, &str); 7] = [
    (RenderStyle::Pbr, Tint::None, "0-pbr"),
    (RenderStyle::Pbr, Tint::Hit, "1-pbr-hit"),
    (RenderStyle::Pbr, Tint::Telegraph, "2-pbr-telegraph"),
    (RenderStyle::Yancai, Tint::None, "3-yancai"),
    (RenderStyle::Yancai, Tint::Hit, "4-yancai-hit"),
    (RenderStyle::Yancai, Tint::Telegraph, "5-yancai-telegraph"),
    (RenderStyle::Pbr, Tint::None, "6-pbr-again"),
];

#[derive(Resource, Default)]
pub(super) struct Shots {
    frame: u32,
    tint: Option<Tint>,
    window: (f32, u32),
}

pub(super) fn configure(app: &mut App) {
    app.init_resource::<Shots>()
        .add_systems(Update, capture)
        // After every Update writer of enemy materials, before extraction.
        .add_systems(PostUpdate, force_tint);
}

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    mut shots: ResMut<Shots>,
    mut style: ResMut<RenderStyle>,
    styles: Res<LevelStyles>,
    mut exit: MessageWriter<AppExit>,
) {
    // The plan needs both styles; a single-style level has nothing to compare.
    if styles.count() < 2 {
        error!("{FLAG} needs a level with more than one render style");
        exit.write(AppExit::error());
        return;
    }
    shots.frame += 1;
    let Some(offset) = shots.frame.checked_sub(SETTLE) else {
        return;
    };
    let index = (offset / STEP) as usize;
    let Some((wanted, tint, name)) = PLAN.get(index) else {
        shots.tint = None;
        // One spare step lets the final screenshot flush before exiting.
        if index > PLAN.len() {
            exit.write(AppExit::Success);
        }
        return;
    };
    style.set_if_neq(*wanted);
    shots.tint = Some(*tint);
    let phase = offset % STEP;
    // Skip the switch frame itself when averaging.
    if phase == 1 {
        shots.window = (0.0, 0);
    } else if phase > 1 {
        shots.window.0 += time.delta_secs();
        shots.window.1 += 1;
    }
    if phase == STEP - 1 {
        let (total, frames) = shots.window;
        info!(
            "render-style shot {name}: mean frame {:.1} ms over {frames} frames",
            total * 1000.0 / frames.max(1) as f32
        );
        let _ = std::fs::create_dir_all("target/render-style");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("target/render-style/{name}.png")));
    }
}

fn force_tint(
    shots: Res<Shots>,
    mut parts: Query<(&AvatarSurface, &mut MeshMaterial3d<StandardMaterial>), With<EnemyBody>>,
) {
    let Some(tint) = shots.tint.filter(|tint| *tint != Tint::None) else {
        return;
    };
    for (surface, mut material) in &mut parts {
        material.0 = surface.material_for(tint == Tint::Hit, tint == Tint::Telegraph);
    }
}
