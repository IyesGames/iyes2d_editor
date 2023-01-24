use crate::crate_prelude::*;

pub(crate) struct EditorUiPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for EditorUiPlugin<S> {
    fn build(&self, app: &mut App) {
    }
}

