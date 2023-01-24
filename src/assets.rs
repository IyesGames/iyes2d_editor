use crate::crate_prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct EditorAssets {
    #[asset(key = "iyes2d_editor.font.regular")]
    pub(crate) font: Handle<Font>,
    #[asset(key = "iyes2d_editor.font.bold")]
    pub(crate) font_bold: Handle<Font>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.depressed")]
    pub(crate) image_ui_toolbar_depressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.pressed")]
    pub(crate) image_ui_toolbar_pressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.hover")]
    pub(crate) image_ui_toolbar_hover: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.disabled")]
    pub(crate) image_ui_toolbar_disabled: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.selectentities")]
    pub(crate) image_icon_tool_selectentities: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.translation")]
    pub(crate) image_icon_tool_translation: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.selecttilemap")]
    pub(crate) image_icon_tool_selecttilemap: Handle<Image>,
}

pub struct EditorAssetsPlugin<S: StateData> {
    pub asset_load_state: S,
    pub editor_state: S,
}

impl<S: StateData> Plugin for EditorAssetsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(self.asset_load_state.clone())
                .continue_to_state(self.editor_state.clone())
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "iyes2d_editor.assets",
                ])
                .with_collection::<EditorAssets>()
        );
        app.add_exit_system(self.editor_state.clone(), remove_resource::<EditorAssets>);
    }
}
