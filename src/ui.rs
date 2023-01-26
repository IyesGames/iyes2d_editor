use crate::crate_prelude::*;

pub(crate) mod toolbar;
pub(crate) mod tooltip;
pub(crate) mod panel;

pub(crate) struct EditorUiPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for EditorUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_plugin(toolbar::ToolbarPlugin {
            state: self.state.clone(),
        });
        app.add_plugin(tooltip::TooltipPlugin {
            state: self.state.clone(),
        });
        app.add_plugin(panel::PanelPlugin {
            state: self.state.clone(),
        });
        app.add_system(simple_butt_visual.run_in_state(self.state.clone()));
    }
}

#[derive(Component)]
struct SimpleButtVisual;

fn simple_butt_visual(
    assets: Res<EditorAssets>,
    mut q_butt: Query<(&Interaction, &mut UiImage, Option<&UiInactive>), (With<Button>, With<SimpleButtVisual>, Changed<Interaction>)>,
) {
    for (interaction, mut current_image, disabled) in &mut q_butt {
        if let Ok(handle) = assets.butt_change_image(&current_image.0, *interaction, disabled.is_some(), false) {
            current_image.0 = handle;
        }
    }
}
