use crate::crate_prelude::*;

pub(crate) mod toolbar;
pub(crate) mod tooltip;
pub(crate) mod panel;
pub(crate) mod menu;

pub(crate) struct EditorUiPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for EditorUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_plugin(iyes_bevy_extras::ui::UiExtrasPlugin);
        app.add_plugin(toolbar::ToolbarPlugin {
            state: self.state.clone(),
        });
        app.add_plugin(tooltip::TooltipPlugin {
            state: self.state.clone(),
        });
        app.add_plugin(panel::PanelPlugin {
            state: self.state.clone(),
        });
        app.add_plugin(menu::MenuPlugin {
            state: self.state.clone(),
        });
        app.add_system(simple_butt_visual.in_set(EditorSet));
    }
}

#[derive(Component)]
struct SimpleButtVisual;

fn simple_butt_visual(
    assets: Res<EditorAssets>,
    mut q_butt: Query<
        (&Interaction, &mut UiImage, Option<&UiDisabled>),
        (With<Button>, With<SimpleButtVisual>, Changed<Interaction>)
    >,
) {
    for (interaction, mut current_image, disabled) in &mut q_butt {
        if let Ok(handle) = assets.butt_change_image(&current_image.texture, *interaction, disabled.is_some(), false) {
            current_image.texture = handle;
        }
    }
}
