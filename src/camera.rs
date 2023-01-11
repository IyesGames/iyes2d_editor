use bevy::prelude::*;
use bevy::ecs::schedule::StateData;
use bevy::render::camera::RenderTarget;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum SystemLabels {
    WorldCursor,
}

pub(crate) struct CameraPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for CameraPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursor>();
        app.add_system(
            world_cursor
                .run_in_state(self.state.clone())
                .label(SystemLabels::WorldCursor)
        );
    }
}

#[derive(Component)]
pub struct EditorCamera;

#[derive(Resource, Default)]
pub(crate) struct WorldCursor {
    pub pos: Vec2,
}

fn world_cursor(
    windows: Res<Windows>,
    mut crs: ResMut<WorldCursor>,
    q_camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
) {
    let (camera, xf_camera) = q_camera.single();
    let RenderTarget::Window(w_id) = camera.target
    else {
        panic!("Editor camera must render to a window!");
    };
    let Some(cursor) = windows
        .get(w_id)
        .and_then(|window| window.cursor_position())
        .and_then(|pos| camera.viewport_to_world(xf_camera, pos))
        .map(|ray| ray.origin.truncate())
    else {
        return;
    };
    crs.pos = cursor;
}
