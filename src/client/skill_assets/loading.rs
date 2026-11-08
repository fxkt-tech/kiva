use super::catalog::{MeshKey, TextureKey, mesh_spec, texture_spec};
use bevy::{asset::LoadState, gltf::GltfAssetLabel, prelude::*};
use serde::Serialize;
use std::collections::HashMap;

pub(in crate::client) struct SkillAssetsPlugin;
impl Plugin for SkillAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillAssets>()
            .add_systems(Startup, load)
            .add_systems(Update, prepare.in_set(SkillAssetsSet::Prepare));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::client) enum SkillAssetsSet {
    Prepare,
}

#[derive(Clone, Default, Serialize)]
pub(in crate::client) struct AssetStatus {
    pub ready: bool,
    pub errors: Vec<String>,
}

#[derive(Resource, Default)]
pub(in crate::client) struct SkillAssets {
    textures: HashMap<TextureKey, TextureAsset>,
    meshes: HashMap<MeshKey, MeshAsset>,
    status: AssetStatus,
}
struct TextureAsset {
    image: Handle<Image>,
    material: Handle<StandardMaterial>,
    ready: bool,
}
struct MeshAsset {
    mesh: Handle<Mesh>,
    ready: bool,
}
impl SkillAssets {
    /// No ready handle means the renderer should retain its existing fallback.
    pub fn image(&self, key: TextureKey) -> Option<&Handle<Image>> {
        self.textures
            .get(&key)
            .filter(|asset| asset.ready)
            .map(|asset| &asset.image)
    }
    pub fn material(&self, key: TextureKey) -> Option<&Handle<StandardMaterial>> {
        self.textures
            .get(&key)
            .filter(|asset| asset.ready)
            .map(|asset| &asset.material)
    }
    pub fn mesh(&self, key: MeshKey) -> Option<&Handle<Mesh>> {
        self.meshes
            .get(&key)
            .filter(|asset| asset.ready)
            .map(|asset| &asset.mesh)
    }
    pub fn status(&self) -> &AssetStatus {
        &self.status
    }

    /// Headless lifecycle tests replace only GPU assets, never combat facts.
    #[cfg(test)]
    pub fn fixture(world: &mut World) -> Self {
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let mut assets = Self::default();
        for key in TextureKey::ALL {
            let image = world.resource_mut::<Assets<Image>>().add(Image::default());
            let material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(material_template(key, image.clone()));
            assets.textures.insert(
                key,
                TextureAsset {
                    image,
                    material,
                    ready: true,
                },
            );
        }
        for key in MeshKey::ALL {
            let mesh = world
                .resource_mut::<Assets<Mesh>>()
                .add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
            assets.meshes.insert(key, MeshAsset { mesh, ready: true });
        }
        assets.status.ready = true;
        assets
    }
}

fn material_template(key: TextureKey, image: Handle<Image>) -> StandardMaterial {
    let spec = texture_spec(key);
    StandardMaterial {
        base_color_texture: Some(image),
        alpha_mode: spec.blend.into(),
        uv_transform: spec
            .atlas
            .uv_transform(spec.atlas.default_frame)
            .unwrap_or_default(),
        unlit: true,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        ..default()
    }
}

fn load(
    server: Res<AssetServer>,
    mut assets: ResMut<SkillAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for key in TextureKey::ALL {
        let spec = texture_spec(key);
        let image = server.load(spec.path.clone());
        let material = materials.add(material_template(key, image.clone()));
        assets.textures.insert(
            key,
            TextureAsset {
                image,
                material,
                ready: false,
            },
        );
    }
    for key in MeshKey::ALL {
        let spec = mesh_spec(key);
        let mesh = server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset(spec.path.clone()),
        );
        assets.meshes.insert(key, MeshAsset { mesh, ready: false });
    }
}

fn prepare(
    server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    mut assets: ResMut<SkillAssets>,
) {
    let mut errors = Vec::new();
    for (key, texture) in &mut assets.textures {
        texture.ready = images.get(&texture.image).is_some();
        if let Some(LoadState::Failed(error)) = server.get_load_state(texture.image.id()) {
            errors.push(format!("{}: {error}", texture_spec(*key).path));
        }
    }
    for (key, asset) in &mut assets.meshes {
        asset.ready = meshes.get(&asset.mesh).is_some();
        if let Some(LoadState::Failed(error)) = server.get_load_state(asset.mesh.id()) {
            errors.push(format!("{}: {error}", mesh_spec(*key).path));
        }
    }
    for error in errors {
        if !assets.status.errors.contains(&error) {
            warn!("Skill visual asset unavailable; retaining fallback: {error}");
            assets.status.errors.push(error);
        }
    }
    assets.status.ready = assets.status.errors.is_empty()
        && assets.textures.len() == TextureKey::ALL.len()
        && assets.meshes.len() == MeshKey::ALL.len()
        && assets.textures.values().all(|asset| asset.ready)
        && assets.meshes.values().all(|asset| asset.ready);
}
