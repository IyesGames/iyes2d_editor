//! Selection management (tracking of what is currently selected)
//!
//! This module is where all the general (not specific to any particular
//! kind of entity) selection code is.
//!
//! To allow selecting different kinds of entities (sprites, tilemaps, etc.),
//! their respective modules should have systems that help with identifying them
//! and creating the selections.
//!
//! Selections are entities with a `Selection` component to indicate what they
//! are tracking. These entities also carry the components for visualizing the
//! selection.

use bevy::{sprite::Anchor, transform::TransformSystem};

use crate::{crate_prelude::*, camera::WorldCursor};

pub(crate) struct SelectionPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for SelectionPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_exit_system(self.state.clone(), remove_from_all::<Selected>);
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_selection_visual_sprite
                .run_in_state(self.state.clone())
        );
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            selection_follow_entity_transform
                .run_in_state(self.state.clone())
                .after(TransformSystem::TransformPropagate)
        );
        app.add_system(
            deselect_selections_on_click
                .run_in_state(self.state.clone())
                .run_for_tools(Tool::SelectEntities)
        );
    }
}

/// Used on selection entities to track what Entity they are associated with
#[derive(Component)]
pub struct Selection {
    pub target: Entity,
}

/// Used on selected entities to track their selection entity
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Selected {
    pub selection: Entity,
}

/// The rectangle to display on-screen for the selection
#[derive(Component)]
pub(crate) struct SelectionVisualBounds {
    /// in local space
    pub rect: Rect,
}

impl Default for SelectionVisualBounds {
    fn default() -> Self {
        SelectionVisualBounds {
            rect: Rect::new(-0.5, -0.5, 0.5, 0.5),
        }
    }
}

#[derive(Component)]
pub(crate) struct SelectionVisualColor(Color);

impl Default for SelectionVisualColor {
    fn default() -> Self {
        let mut color = Color::PINK;
        color.set_a(0.5);
        SelectionVisualColor(color)
    }
}

#[derive(Bundle)]
pub(crate) struct SelectionBundle {
    sprite: SpriteBundle,
    selection: Selection,
    bounds: SelectionVisualBounds,
    color: SelectionVisualColor,
    cleanup: EditorCleanup,
}

impl SelectionBundle {
    pub(crate) fn from_entity(entity: Entity) -> SelectionBundle {
        SelectionBundle {
            selection: Selection { target: entity },
            sprite: SpriteBundle::default(),
            bounds: SelectionVisualBounds::default(),
            color: SelectionVisualColor::default(),
            cleanup: EditorCleanup,
        }
    }
    pub(crate) fn with_transform(mut self, xf: Transform) -> Self {
        self.sprite.transform = xf;
        self
    }
    pub(crate) fn with_bounds(mut self, rect: Rect) -> Self {
        self.bounds.rect = rect;
        self
    }
}

fn update_selection_visual_sprite(
    mut q_selection: Query<
        (&mut Sprite, &SelectionVisualBounds, &SelectionVisualColor),
        Or<(Changed<SelectionVisualBounds>, Changed<SelectionVisualColor>)>,
    >,
) {
    for (mut sprite, bounds, color) in &mut q_selection {
        sprite.color = color.0;
        sprite.custom_size = Some(bounds.rect.size());

        // we basically need to undo the math that the anchor thing does
        let anchor = (-bounds.rect.min) / (bounds.rect.max - bounds.rect.min) - Vec2::new(0.5, 0.5);
        sprite.anchor = Anchor::Custom(anchor);
    }
}

fn selection_follow_entity_transform(
    mut q_selection: Query<&mut GlobalTransform, With<Selection>>,
    q_target: Query<(&GlobalTransform, &Selected), (Without<Selection>, Changed<GlobalTransform>)>,
) {
    for (xf_tgt, sel) in &q_target {
        if let Ok(mut xf_sel) = q_selection.get_mut(sel.selection) {
            *xf_sel = *xf_tgt;
        }
    }
}

fn deselect_selections_on_click(
    mut commands: Commands,
    btn: Res<Input<MouseButton>>,
    crs: Res<WorldCursor>,
    q_selection: Query<(Entity, &Selection, &GlobalTransform, &SelectionVisualBounds)>,
) {
    if btn.just_pressed(MouseButton::Left) {
        for (e, sel, xf, bounds) in &q_selection {
            let crs_local = xf.compute_matrix().inverse() * crs.pos.extend(0.0).extend(1.0);
            if crs_local.x >= bounds.rect.min.x && crs_local.y >= bounds.rect.min.y &&
               crs_local.x <= bounds.rect.max.x && crs_local.y <= bounds.rect.max.y
            {
                commands.entity(sel.target).remove::<Selected>();
                commands.entity(e).despawn_recursive();
            }
        }
    }
}

