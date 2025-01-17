use std::{
    collections::HashMap,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use ambient_ecs::{components, Networked, Resource, World};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::ServerWorldExt;
use ambient_project_semantic::Semantic;

components!("semantic", {
    @[Resource]
    semantic: Arc<Mutex<Semantic>>,

    @[Resource, Networked]
    ember_name_to_url: HashMap<String, String>,
});

pub fn world_semantic(world: &World) -> Arc<Mutex<Semantic>> {
    world.resource(semantic()).clone()
}

/// Returns the path for the given file in the given ember, or returns a global path
/// if that ember doesn't have an associated URL.
///
/// Note that `path` is relative to the root of the ember's build directory, so an
/// asset will require `assets/` prefixed to the path.
pub fn file_path(world: &World, ember_name: &str, path: &Path) -> anyhow::Result<AbsAssetUrl> {
    let url = world
        .synced_resource(ember_name_to_url())
        .unwrap()
        .get(ember_name);
    if let Some(url) = url {
        Ok(AbsAssetUrl::from_str(&format!("{url}/{}", path.display()))?)
    } else {
        Ok(AbsAssetUrl::from_asset_key(path.to_string_lossy())?)
    }
}
