use crate::crate_prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct EditorAssets {
    #[asset(key = "iyes2d_editor.font.regular")]
    pub(crate) font: Handle<Font>,
    #[asset(key = "iyes2d_editor.font.bold")]
    pub(crate) font_bold: Handle<Font>,
    #[asset(key = "logo.small")]
    pub(crate) logo_small: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.smallbutt.depressed")]
    pub(crate) image_ui_smallbutt_depressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.smallbutt.pressed")]
    pub(crate) image_ui_smallbutt_pressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.smallbutt.hover")]
    pub(crate) image_ui_smallbutt_hover: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.depressed")]
    pub(crate) image_ui_toolbar_depressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.pressed")]
    pub(crate) image_ui_toolbar_pressed: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.hover")]
    pub(crate) image_ui_toolbar_hover: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.ui.toolbar.disabled")]
    pub(crate) image_ui_toolbar_disabled: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.wm.close")]
    pub(crate) image_icon_wm_close: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.wm.minify")]
    pub(crate) image_icon_wm_minify: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.selectentities")]
    pub(crate) image_icon_tool_selectentities: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.translation")]
    pub(crate) image_icon_tool_translation: Handle<Image>,
    #[asset(key = "iyes2d_editor.image.icon.tool.selecttilemap")]
    pub(crate) image_icon_tool_selecttilemap: Handle<Image>,
}

impl EditorAssets {
    pub(crate) fn butt_change_image(&self, current_image: &Handle<Image>, interaction: Interaction, disabled: bool, selected: bool) -> Result<Handle<Image>, ()> {
        // depressed, hover, pressed, disabled
        let images =
        if current_image == &self.image_ui_smallbutt_depressed ||
           current_image == &self.image_ui_smallbutt_hover ||
           current_image == &self.image_ui_smallbutt_pressed
        {
            [&self.image_ui_smallbutt_depressed, &self.image_ui_smallbutt_hover, &self.image_ui_smallbutt_pressed, &self.image_ui_smallbutt_depressed]
        } else if current_image == &self.image_ui_toolbar_depressed ||
                  current_image == &self.image_ui_toolbar_hover ||
                  current_image == &self.image_ui_toolbar_pressed
        {
            [&self.image_ui_toolbar_depressed, &self.image_ui_toolbar_hover, &self.image_ui_toolbar_pressed, &self.image_ui_toolbar_disabled]
        } else {
            return Err(());
        };

        let handle = match (selected, disabled, interaction) {
            (true, _, _) => images[2].clone(),
            (false, true, _) => images[3].clone(),
            (false, false, Interaction::None) => images[0].clone(),
            (false, false, Interaction::Hovered) => images[1].clone(),
            (false, false, Interaction::Clicked) => images[2].clone(),
        };
        Ok(handle)
    }
}

pub(crate) struct EditorAssetsPlugin<S: States> {
    pub asset_load_state: S,
    pub editor_state: S,
}

impl<S: States> Plugin for EditorAssetsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(self.asset_load_state.clone())
                .continue_to_state(self.editor_state.clone())
        );
        app.add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            self.asset_load_state.clone(),
            "iyes2d_editor.assets.ron"
        );
        app.add_collection_to_loading_state::<_, EditorAssets>(self.asset_load_state.clone());
        app.add_systems(
            (
                remove_resource::<EditorAssets>,
            ).in_schedule(OnExit(self.editor_state.clone()))
        );
    }
}
