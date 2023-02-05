use crate::crate_prelude::*;

use crate::camera::WorldCursor;
use crate::selection::{Selected, Selection, SelectionBundle};

pub(crate) struct SpriteEditorPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for SpriteEditorPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system(
            select_sprites_mouseclick
                .run_in_state(self.state.clone())
                .run_for_tools(Tool::SelectEntities)
        );
    }
}

fn select_sprites_mouseclick(
    mut commands: Commands,
    btn: Res<Input<MouseButton>>,
    crs: Res<WorldCursor>,
    q_sprite: Query<(Entity, &Sprite, &Handle<Image>, &GlobalTransform), (Without<Selected>, Without<Selection>)>,
    images: Res<Assets<Image>>,
) {
    if btn.just_pressed(MouseButton::Left) {
        for (e, sprite, handle, xf) in &q_sprite {
            let crs_local = xf.compute_matrix().inverse() * crs.pos.extend(0.0).extend(1.0);

            // do the same arithmetic that bevy does when calculating the vertices of the sprite quad
            let sprite_dimensions = if let Some(custom_size) = sprite.custom_size {
                custom_size
            } else if let Some(image) = images.get(handle) {
                image.size()
            } else {
                continue;
            };
            let anchor = sprite.anchor.as_vec();
            let rect = Rect::new(
                (-0.5 - anchor.x) * sprite_dimensions.x,
                (-0.5 - anchor.y) * sprite_dimensions.y,
                ( 0.5 - anchor.x) * sprite_dimensions.x,
                ( 0.5 - anchor.y) * sprite_dimensions.y,
            );

            if crs_local.x >= rect.min.x && crs_local.y >= rect.min.y &&
               crs_local.x <= rect.max.x && crs_local.y <= rect.max.y
            {
                let e_sel = commands.spawn(
                    SelectionBundle::from_entity(e)
                        .with_transform(xf.compute_transform())
                        .with_bounds(rect)
                ).id();
                commands.entity(e).insert(Selected { selection: e_sel });
            }
        }
    }
}
