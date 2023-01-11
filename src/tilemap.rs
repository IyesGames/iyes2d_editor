use crate::crate_prelude::*;
use bevy_ecs_tilemap::{prelude::*, FrustumCulling};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::BevyDefault;

use crate::camera::WorldCursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum SystemLabels {
    TilemapSelect,
}

pub(crate) struct TilemapEditorPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for TilemapEditorPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionTextures>();
        app.init_resource::<GridCursor>();
        app.init_resource::<SelectedTilemap>();
        app.add_system(
            cursor_tilemap_select
                .run_in_state(self.state.clone())
                .run_on_mouse_press(MouseButton::Left)
                .label(SystemLabels::TilemapSelect)
                .after(crate::camera::SystemLabels::WorldCursor)
        );
        app.add_system(
            manage_overlay_tilemap
                .run_in_state(self.state.clone())
                .run_if_resource_changed::<SelectedTilemap>()
                .after(SystemLabels::TilemapSelect)
        );
    }
}

#[derive(Resource, Default)]
pub struct SelectedTilemap {
    pub entity: Option<Entity>,
}

#[derive(Resource, Default)]
struct GridCursor {
    pos: Option<TilePos>,
}

#[derive(Component)]
struct OverlayTilemap;

fn cursor_tilemap_select(
    crs: Res<WorldCursor>,
    mut tm_selected: ResMut<SelectedTilemap>,
    q_tmap: Query<(Entity, &TilemapSize, &TilemapGridSize, &TilemapType, &GlobalTransform)>,
) {
    // TODO: select "through" empty tiles?

    let mut closest_z = f32::MIN;
    for (e_tm, size, grid_size, map_type, xf_tm) in &q_tmap {
        if *map_type != TilemapType::Square {
            warn!("Only tilemaps of Square type are currently supported!");
            continue;
        }

        let xf_tm_translation = xf_tm.translation();
        if xf_tm_translation.z <= closest_z {
            continue;
        }

        let xf_tm_matrix_inverse = xf_tm.compute_matrix().inverse();
        let crs_tm = xf_tm_matrix_inverse * crs.pos
            .extend(xf_tm_translation.z)
            .extend(1.0);

        let min = Vec2::new(
            -grid_size.x / 2.0,
            -grid_size.y / 2.0,
        );
        let max = Vec2::new(
            size.x as f32 * grid_size.x as f32 - grid_size.x / 2.0,
            size.y as f32 * grid_size.y as f32 - grid_size.y / 2.0,
        );
        if crs_tm.x >= min.x && crs_tm.x <= max.x && crs_tm.y >= min.y && crs_tm.y <= max.y {
            tm_selected.entity = Some(e_tm);
            closest_z = xf_tm_translation.z;
        }
    }
    if closest_z == f32::MIN {
        tm_selected.entity = None;
    }

    info!("Selecting tilemap: {:?}", tm_selected.entity);
}

fn manage_overlay_tilemap(
    mut commands: Commands,
    mut selection_textures: ResMut<SelectionTextures>,
    mut images: ResMut<Assets<Image>>,
    tm_selected: Res<SelectedTilemap>,
    q_tm: Query<(&TilemapSize, &TilemapGridSize, &TilemapTileSize, &TilemapType, &TilemapSpacing, &Transform)>,
    q_tm_overlay: Query<Entity, With<OverlayTilemap>>,
) {
    if let Some(e_tm_selected) = tm_selected.entity {
        if let Ok((size, grid_size, tile_size, map_type, spacing, transform)) = q_tm.get(e_tm_selected) {
            if *map_type != TilemapType::Square {
                error!("Only tilemaps of Square type are currently supported!");
                return;
            }
            commands.spawn((
                TilemapBundle {
                    // from selected tilemap
                    size: *size,
                    grid_size: *grid_size,
                    tile_size: *tile_size,
                    map_type: *map_type,
                    spacing: *spacing,
                    transform: *transform,
                    texture: TilemapTexture::Single(
                        selection_textures.get_or_create(&mut images, tile_size.x as u32, tile_size.y as u32),
                    ),
                    // initialized for our tilemap
                    storage: TileStorage::empty(*size),
                    frustum_culling: FrustumCulling(true),
                    visibility: Visibility::VISIBLE,
                    // defaults
                    global_transform: default(),
                    computed_visibility: default(),
                },
                OverlayTilemap,
            ));
        }
    } else {
        for e in &q_tm_overlay {
            commands.entity(e).despawn();
        }
    }
}

#[derive(Resource, Default)]
struct SelectionTextures {
    handles: HashMap<(u32, u32), Handle<Image>>,
}

impl SelectionTextures {
    fn get_or_create(&mut self, images: &mut ResMut<Assets<Image>>, width: u32, height: u32) -> Handle<Image> {
        if let Some(handle) = self.handles.get(&(width, height)) {
            handle.clone()
        } else {
            // create new image
            let image = Image::new_fill(
                Extent3d {
                    depth_or_array_layers: 1,
                    width, height,
                },
                TextureDimension::D2,
                &[0xFF, 0xFF, 0xFF, 0xFF],
                TextureFormat::bevy_default(),
            );
            let handle = images.add(image);
            self.handles.insert((width, height), handle.clone());
            handle
        }
    }
}
