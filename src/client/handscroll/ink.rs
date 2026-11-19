//! Reveal the new leaf under a receding edge of wet ink on the previous painting.
use bevy::{
    asset::{load_internal_asset, uuid_handle},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

const SHADER: Handle<Shader> = uuid_handle!("020b152b-59a0-4926-ae4c-07794542ff32");
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub(super) struct Ink {
    #[texture(0)]
    #[sampler(1)]
    painting: Handle<Image>,
    #[uniform(2)]
    spread: Vec4,
}

impl UiMaterial for Ink {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}
pub(super) fn configure(app: &mut App) {
    load_internal_asset!(app, SHADER, "reveal.wesl", Shader::from_wesl);
    app.add_plugins(UiMaterialPlugin::<Ink>::default());
}

pub(super) fn reveal(
    commands: &mut Commands,
    materials: &mut Assets<Ink>,
    paper: Entity,
    painting: Handle<Image>,
    origin: Vec2,
) {
    commands.spawn((
        ChildOf(paper),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ZIndex(2),
        MaterialNode(materials.add(Ink {
            painting,
            spread: Vec4::new(origin.x, origin.y, 0., 0.),
        })),
    ));
}

pub(super) fn advance(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut materials: ResMut<Assets<Ink>>,
    transitions: Query<(Entity, &MaterialNode<Ink>)>,
) {
    for (entity, handle) in &transitions {
        if let Some(mut ink) = materials.get_mut(&handle.0) {
            ink.spread.z += time.delta_secs() / 0.4;
            if ink.spread.z >= 1. {
                commands.entity(entity).despawn();
            }
        }
    }
}
