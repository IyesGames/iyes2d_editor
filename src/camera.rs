use bevy::{input::mouse::{MouseMotion, MouseWheel}, window::PrimaryWindow};
use crate::crate_prelude::*;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraSet;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WorldCursorSet;

pub(crate) struct CameraPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for CameraPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursor>();
        app.add_systems(
            (
                ensure_setup_editor_camera,
                showhide_other_cameras::<false>,
            ).in_schedule(OnEnter(self.state.clone()))
        );
        app.add_systems(
            (
                showhide_other_cameras::<true>,
            ).in_schedule(OnExit(self.state.clone()))
        );
        app.configure_set(CameraControlSet.after(WorldCursorSet));
        app.add_system(
            world_cursor
                .in_set(WorldCursorSet)
                .in_set(CameraSet)
                .in_set(EditorSet)
        );
        app.add_systems(
            (
                camera_pan,
                camera_rotate,
                camera_zoom,
            ).in_set(CameraSet)
            .in_set(EditorSet)
            .in_set(CameraControlSet)
        );
    }
}

#[derive(Component)]
pub struct EditorCamera;

#[derive(Resource, Default)]
pub(crate) struct WorldCursor {
    pub pos: Vec2,
}

fn showhide_other_cameras<const VIS: bool>(
    mut q_camera: Query<&mut Visibility, (With<Camera>, Without<EditorCamera>)>,
) {
    for mut vis in &mut q_camera {
        *vis = if VIS {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn ensure_setup_editor_camera(
    mut commands: Commands,
    // To check if one already exists
    q_camera_check: Query<Entity, (With<Camera>, With<EditorCamera>, With<Camera2d>)>,
) {
    use bevy::ecs::query::QuerySingleError;
    let e_camera = match q_camera_check.get_single() {
        Ok(e) => e,
        Err(QuerySingleError::MultipleEntities(_)) => {
            // get the first and despawn the others
            let mut iter = q_camera_check.iter();
            let e = iter.next().unwrap();
            for e in iter {
                commands.entity(e).despawn();
            }
            e
        }
        Err(QuerySingleError::NoEntities(_)) => {
            // spawn one
            commands.spawn((
                Camera2dBundle::default(),
                EditorCamera,
            )).id()
        }
    };
    // TODO: enforce things we care about on the camera, here
}

fn camera_pan(
    mousebutt: Res<Input<MouseButton>>,
    crs: Res<WorldCursor>,
    mut q_camera: Query<&mut Transform, With<EditorCamera>>,
    mut startpos: Local<Vec2>,
) {
    // TODO: transition to a proper input mgmt framework like LWIM
    if mousebutt.just_pressed(MouseButton::Right) {
        *startpos = crs.pos;
    }
    if mousebutt.pressed(MouseButton::Right) {
        // pan based on WorldCursor, so camera follows on-screen nicely
        // this system must run *after* world cursor, or they will race
        // (mathematically, next frame the world cursor should be in the same place)
        let delta = crs.pos - *startpos;
        let mut xf_cam = q_camera.single_mut();
        xf_cam.translation.x -= delta.x;
        xf_cam.translation.y -= delta.y;
    }
}

fn camera_rotate(
    mousebutt: Res<Input<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, With<EditorCamera>>,
) {
    // TODO: transition to a proper input mgmt framework like LWIM
    if mousebutt.pressed(MouseButton::Middle) {
        let delta: f32 = motion.iter().map(|ev| ev.delta.x).sum();
        if delta != 0.0 {
            let mut xf_cam = q_camera.single_mut();
            xf_cam.rotate_z(delta / 256.0);
        }
    }
}

fn camera_zoom(
    kbd: Res<Input<KeyCode>>,
    mut motion: EventReader<MouseWheel>,
    mut last_zoom: Local<Option<Instant>>,
    mut q_camera: Query<&mut Transform, With<EditorCamera>>,
) {
    if !kbd.pressed(KeyCode::LShift) {
        return;
    }

    // TODO: this feels awful but will do for now
    // just throttle how often we can zoom
    if let Some(last) = &*last_zoom {
        if Instant::now() - *last < Duration::from_millis(125) {
            motion.clear();
            return;
        }
    }

    // TODO: transition to a proper input mgmt framework like LWIM
    let delta: f32 = motion.iter().map(|ev| ev.y).sum();
    if delta != 0.0 {
        let mut xf_cam = q_camera.single_mut();
        // let mul = delta.exp();
        let mul = if delta < 0.0 {
            0.5
        } else {
            2.0
        };
        xf_cam.scale.x *= mul;
        xf_cam.scale.y *= mul;
        *last_zoom = Some(Instant::now());
    }
}

fn world_cursor(
    mut crs: ResMut<WorldCursor>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
) {
    let (camera, xf_camera) = q_camera.single();
    // let RenderTarget::Window(w_id) = camera.target
    // else {
    //     panic!("Editor camera must render to a window!");
    // };
    let Some(cursor) = q_windows
        .get_single().ok()
        .and_then(|window| window.cursor_position())
        .and_then(|pos| camera.viewport_to_world(xf_camera, pos))
        .map(|ray| ray.origin.truncate())
    else {
        return;
    };
    crs.pos = cursor;
}
