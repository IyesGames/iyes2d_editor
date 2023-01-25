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
    }
}
