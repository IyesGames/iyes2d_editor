use crate::crate_prelude::*;

mod toolbar;

pub(crate) struct EditorUiPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for EditorUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_plugin(toolbar::ToolbarPlugin {
            state: self.state.clone(),
        });
    }
}
