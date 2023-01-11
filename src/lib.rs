use crate::crate_prelude::*;

#[cfg(feature = "bevy_ecs_tilemap")]
pub mod tilemap;

pub mod camera;
mod assets;
mod ui;
mod misc;

/// Public prelude
pub mod prelude {
    pub use crate::EditorPlugin;
}

/// Common prelude for internal use
mod crate_prelude {
    pub use bevy::prelude::*;
    pub use bevy::utils::{HashMap, HashSet, Duration, Instant};
    pub use bevy::ecs::schedule::StateData;
    pub use iyes_loopless::prelude::*;
    pub use iyes_bevy_util::prelude::*;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum SystemLabels {
    WorldCursor,
    TilemapSelect,
}

pub struct EditorPlugin<S: StateData> {
    pub asset_load_state: S,
    pub editor_state: S,
}

impl<S: StateData> Plugin for EditorPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_plugin(crate::assets::EditorAssetsPlugin {
            asset_load_state: self.asset_load_state.clone(),
            editor_state: self.editor_state.clone(),
        });
        app.add_plugin(crate::camera::CameraPlugin {
            state: self.editor_state.clone()
        });
        #[cfg(feature = "bevy_ecs_tilemap")]
        app.add_plugin(crate::tilemap::TilemapEditorPlugin {
            state: self.editor_state.clone()
        });
    }
}
