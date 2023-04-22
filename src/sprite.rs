use crate::crate_prelude::*;

use crate::camera::WorldCursor;
use crate::selection::{Selected, Selection, SelectionCandidateEvent, SelectionCandidateSet, SelectionPending};

pub(crate) struct SpriteEditorPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for SpriteEditorPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system(
            select_sprites
                .in_set(EditorSet)
                .before(SelectionCandidateSet)
                .run_if(with_tools(Tool::SelectEntities))
        );
    }
}

fn select_sprites(
    crs: Res<WorldCursor>,
    q_sprite: Query<(Entity, &Sprite, &Handle<Image>, &GlobalTransform), (Without<Selected>, Without<Selection>, Without<SelectionPending>)>,
    images: Res<Assets<Image>>,
    mut evw_candidate: EventWriter<SelectionCandidateEvent>,
) {
    // PERF: find a way to reduce the number of checks because this will scale horribly

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

        // PERF: dont spam these events every frame
        if crs_local.x >= rect.min.x && crs_local.y >= rect.min.y &&
           crs_local.x <= rect.max.x && crs_local.y <= rect.max.y
        {
            evw_candidate.send(SelectionCandidateEvent::Insert {
                entity: e,
                color: Color::PINK,
                bounds: rect,
            });
        } else {
            evw_candidate.send(SelectionCandidateEvent::Remove {
                entity: e,
            });
        }
    }
}
