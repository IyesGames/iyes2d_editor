use crate::crate_prelude::*;

// Optional modules
#[cfg(feature = "bevy_ecs_tilemap")]
pub mod tilemap;

// Non-optional modules
pub mod sprite;

// General editor framework modules
pub mod camera;
pub mod tool;
pub mod selection;

// Internal support modules
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
    pub use bevy::input::common_conditions::*;
    pub use bevy::ui::FocusPolicy;
    pub use bevy::utils::{HashMap, HashSet, Duration, Instant};
    pub use bevy::ecs::schedule::States;
    pub use iyes_bevy_extras::prelude::*;
    pub use crate::tool::*;
    pub use crate::assets::EditorAssets;
    pub use crate::EditorCleanup;
    pub use crate::EditorSet;
    pub(crate) use crate::EditorFlush;
}

/// All entities with this component will be despawned recursively when exiting the editor state
///
/// If you want to spawn extra custom things for your editor, you can add
/// this component for easy clean-up.
#[derive(Component)]
pub struct EditorCleanup;

/// Add this to your App to integrate the Iyes2D Editor!
///
/// This is the main API of this crate, everything else starts from here.
/// Here you configure the editor and how it fits into your app/game.
///
/// You *must* be using `iyes_loopless` states. The editor requires **2**
/// states for itself: a loading state (where it can load its assets, etc.)
/// and the main "in-editor" state, where you operate the editor.
///
/// To enter the editor, transition to the *editor loading state*! If you
/// transition directly to the in-editor state, things will break.
///
/// These can be part of your game's main app state enum (the one you use
/// to control menu screens, loading screens, in-game, etc.), or you can
/// create a separate state type enum just for controlling the editor.
///
/// ## Using a shared state type
///
/// If the editor states are part of your main states enum, then the editor
/// will behave as a dedicated screen/mode in your app. When you are in the
/// editor, none of your gameplay or other systems will run (unless you
/// specifically configure them to run in the editor state). Do this if you
/// prefer a more "standalone" editor experience.
///
/// ## Using a dedicated state type
///
/// If you make a separate states enum type for the editor, then the editor
/// will behave more like an "overlay" that you can bring up / toggle while
/// in any game state. This allows you to use the editor to manipulate
/// arbitrary entities in real-time. However, it can be trickier to ensure
/// your game's systems don't conflict with the editor. The game will keep
/// running while you are in the editor, unless you implement a separate
/// pausing mechanism.
///
/// ## TL;DR: Iyes2D Editor setup/integration instructions:
///
/// 1. Be sure to copy the editor's asset files into your assets folder!
/// 2. Create app states for the editor to run in.
/// 3. Add this plugin to your App, specifying the states you created.
/// 4. Add some system to your app, that transitions into the editor loading
///    state, whenever you want to enter the editor.
pub struct EditorPlugin<S: States> {
    pub asset_load_state: S,
    pub editor_state: S,
}

/// Set for all editor systems
///
/// If you need additional configuration (beyond just running in the given state),
/// you can add it to this system set.
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct EditorSet;

/// If something needs apply_system_buffers, order before/after this
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub(crate) struct EditorFlush;

impl<S: States> Plugin for EditorPlugin<S> {
    fn build(&self, app: &mut App) {
        app.configure_set(
            EditorSet
                .run_if(in_state(self.editor_state.clone()))
        );
        app.configure_set(
            EditorFlush
                .in_set(EditorSet)
        );
        app.add_system(
            apply_system_buffers
                .in_set(EditorFlush)
        );
        app.add_state::<crate::tool::Tool>();
        app.add_plugin(crate::assets::EditorAssetsPlugin {
            asset_load_state: self.asset_load_state.clone(),
            editor_state: self.editor_state.clone(),
        });
        app.add_plugin(crate::camera::CameraPlugin {
            state: self.editor_state.clone()
        });
        app.add_plugin(crate::selection::SelectionPlugin {
            state: self.editor_state.clone()
        });
        app.add_plugin(crate::ui::EditorUiPlugin {
            state: self.editor_state.clone()
        });
        app.add_plugin(crate::sprite::SpriteEditorPlugin {
            state: self.editor_state.clone()
        });
        #[cfg(feature = "bevy_ecs_tilemap")]
        app.add_plugin(crate::tilemap::TilemapEditorPlugin {
            state: self.editor_state.clone()
        });
        app.add_system(
            despawn_all_recursive::<With<EditorCleanup>>
                .in_schedule(OnExit(self.editor_state.clone()))
        );
    }
}
