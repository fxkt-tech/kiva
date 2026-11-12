//! Large display assets are fetched at runtime instead of being linked into the binary.
//!
//! Each file has one decoder with its own cache, so its raw bytes are handed
//! over once and released. Decoders stay synchronous: callers wait for
//! [`status`] before assembling anything which decodes.
use bevy::{
    asset::{AssetLoader, LoadContext, LoadState, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
};

/// A whole file. PNGs are requested as this type too, which keeps Bevy's image
/// loader out of the way of the forest sampler and mip generation.
#[derive(Asset, TypePath)]
pub(super) struct Blob(Vec<u8>);

#[derive(TypePath)]
struct BlobLoader;

impl AssetLoader for BlobLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(Blob(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["bin"]
    }
}

enum Entry {
    Loading(Handle<Blob>),
    Taken,
}

#[derive(Resource, Default)]
pub(super) struct Bundle(HashMap<&'static str, Entry>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Pending { loaded: u32, total: u32 },
    Ready,
    Failed(&'static str),
}

pub(super) struct BundlePlugin;

impl Plugin for BundlePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Blob>()
            .register_asset_loader(BlobLoader)
            .init_resource::<Bundle>();
    }
}

/// Start loading; repeating a request neither restarts nor duplicates it.
pub(super) fn request(world: &mut World, paths: &[&'static str]) {
    let Some(server) = world.get_resource::<AssetServer>().cloned() else {
        return;
    };
    let mut bundle = world.resource_mut::<Bundle>();
    for &path in paths {
        bundle
            .0
            .entry(path)
            .or_insert_with(|| Entry::Loading(server.load(path)));
    }
}

/// Consumed files count as loaded: their decoded form is already cached.
pub(super) fn status(world: &World, paths: &[&'static str]) -> Status {
    let total = paths.len() as u32;
    let (Some(bundle), Some(server)) = (
        world.get_resource::<Bundle>(),
        world.get_resource::<AssetServer>(),
    ) else {
        // Headless fixtures have no server to wait for; `take` reads the disk.
        return if cfg!(test) {
            Status::Ready
        } else {
            Status::Pending { loaded: 0, total }
        };
    };
    let mut loaded = 0;
    for &path in paths {
        match bundle.0.get(path) {
            Some(Entry::Taken) => loaded += 1,
            Some(Entry::Loading(handle)) => match server.load_state(handle.id()) {
                LoadState::Loaded => loaded += 1,
                LoadState::Failed(_) => return Status::Failed(path),
                _ => {}
            },
            None => {}
        }
    }
    if loaded == total {
        Status::Ready
    } else {
        Status::Pending { loaded, total }
    }
}

/// Move a loaded file out of the asset store and release it there.
pub(super) fn take(world: &mut World, path: &'static str) -> Vec<u8> {
    let handle = world
        .get_resource::<Bundle>()
        .and_then(|bundle| match bundle.0.get(path) {
            Some(Entry::Loading(handle)) => Some(handle.clone()),
            _ => None,
        });
    if let Some(handle) = handle
        && let Some(blob) = world
            .get_resource_mut::<Assets<Blob>>()
            .and_then(|mut blobs| blobs.remove(handle.id()))
    {
        world.resource_mut::<Bundle>().0.insert(path, Entry::Taken);
        return blob.0;
    }
    // Headless tests decode without an asset server or a frame loop.
    #[cfg(test)]
    {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(path);
        std::fs::read(&file).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
    }
    #[cfg(not(test))]
    panic!("{path} was decoded before it was loaded, or decoded twice");
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetPlugin;

    const SMALL: &str = "choulingke/environment/litter.png";

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            },
            BundlePlugin,
        ));
        app
    }

    fn settle(app: &mut App, paths: &[&'static str]) -> Status {
        for _ in 0..2_000 {
            app.update();
            let status = status(app.world(), paths);
            if !matches!(status, Status::Pending { .. }) {
                return status;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        status(app.world(), paths)
    }

    #[test]
    fn a_request_is_pending_until_the_real_file_arrives_and_is_released_when_taken() {
        let mut app = app();
        let paths = [SMALL, "yizhiying/weapon.bin"];
        assert_eq!(
            status(app.world(), &paths),
            Status::Pending {
                loaded: 0,
                total: 2
            }
        );
        request(app.world_mut(), &paths[..1]);
        request(app.world_mut(), &paths[..1]);
        assert_eq!(settle(&mut app, &paths[..1]), Status::Ready);
        // The second file was never requested, so the set cannot be ready.
        assert_eq!(
            status(app.world(), &paths),
            Status::Pending {
                loaded: 1,
                total: 2
            }
        );
        let expected = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(SMALL),
        )
        .unwrap();
        assert_eq!(take(app.world_mut(), SMALL), expected);
        assert_eq!(app.world().resource::<Assets<Blob>>().len(), 0);
        assert_eq!(status(app.world(), &paths[..1]), Status::Ready);
        // A repeated request must not start another download of a decoded file.
        request(app.world_mut(), &paths[..1]);
        app.update();
        assert_eq!(app.world().resource::<Assets<Blob>>().len(), 0);
    }

    #[test]
    fn a_missing_file_fails_the_set_and_names_the_file() {
        let mut app = app();
        let paths = [SMALL, "missing/character.bin"];
        request(app.world_mut(), &paths);
        assert_eq!(
            settle(&mut app, &paths),
            Status::Failed("missing/character.bin")
        );
    }

    #[test]
    fn headless_decoders_read_the_same_file_without_an_asset_server() {
        let mut world = World::new();
        assert!(!take(&mut world, SMALL).is_empty());
    }
}
