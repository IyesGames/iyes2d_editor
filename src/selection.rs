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

use bevy::{sprite::Anchor, transform::TransformSystem, input::mouse::{MouseWheel, MouseScrollUnit}, utils::FloatOrd};

use crate::{crate_prelude::*, camera::WorldCursor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SelectionCandidateSet;

pub(crate) struct SelectionPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for SelectionPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectionCandidateEvent>();
        app.add_systems(
            (
                remove_from_all::<Selected, With<Selected>>,
                remove_resource::<SelectionCandidates>,
            ).in_schedule(OnExit(self.state.clone()))
        );
        app.add_systems(
            (
                init_resource::<SelectionCandidates>,
                setup_selection_pending,
            ).in_schedule(OnEnter(Tool::SelectEntities))
        );
        app.add_systems(
            (
                remove_resource::<SelectionCandidates>,
                despawn_all::<With<SelectionPending>>,
            ).in_schedule(OnExit(Tool::SelectEntities))
        );
        app.add_systems(
            (
                update_selection_visual_sprite,
            ).in_set(EditorSet).after(EditorFlush)
        );
        app.add_system(
            selection_follow_entity_transform
                .in_base_set(CoreSet::PostUpdate)
                .in_set(EditorSet)
                .after(TransformSystem::TransformPropagate)
        );
        app.add_system(
            deselect_selections_on_click
                .in_set(EditorSet)
                .run_if(with_tools(Tool::SelectEntities))
                .run_if(input_just_pressed(MouseButton::Left))
        );
        app.add_system(
            handle_candidate_events
                .in_set(EditorSet)
                .in_set(SelectionCandidateSet)
                .run_if(with_tools(Tool::SelectEntities))
                .run_if(on_event::<SelectionCandidateEvent>())
        );
        app.add_system(
            disambiguate_candidates
                .in_set(EditorSet)
                .in_set(SelectionCandidateSet)
                .after(handle_candidate_events)
                .run_if(with_tools(Tool::SelectEntities))
                // .run_if(or(on_event::<MouseWheel>(), resource_exists_and_changed::<SelectionCandidates>()))
        );
        app.add_system(
            confirm_selection_on_click
                .in_set(EditorSet)
                .in_set(SelectionCandidateSet)
                .after(disambiguate_candidates)
                .before(EditorFlush)
                .run_if(with_tools(Tool::SelectEntities))
                .run_if(input_just_pressed(MouseButton::Left))
        );
        app.add_system(
            update_pending_visual
                .in_set(EditorSet)
                .in_set(SelectionCandidateSet)
                .after(disambiguate_candidates)
                .before(update_selection_visual_sprite)
                .run_if(with_tools(Tool::SelectEntities))
        );
    }
}

/// The selection process works using this event to indicate what to select
///
/// Other modules (sprite handling, tilemap handling, etc) can check if the
/// mouse cursor is hovering over their respective entities, and emit `Insert`
/// events. This module will aggregate them and allow the user to disambiguate
/// (choose which entity to select if there are multiple candidates). A highlight
/// will be shown so the user has a visual indication to see what is going on.
///
/// `Remove` events can be used for cancellation: to discard selection candidates.
///
/// This module will also allow the user to confirm the selection, and take care
/// of managing the selection entities and components when that happens.
///
/// Note: other modules are responsible for keeping SelectionVisualBounds up to date.
pub enum SelectionCandidateEvent {
    Insert {
        entity: Entity,
        color: Color,
        bounds: Rect,
    },
    Remove {
        entity: Entity,
    },
}

#[derive(Resource, Default)]
struct SelectionCandidates {
    candidates: HashMap<Entity, (Color, Rect)>,
}

/// Used on the highlight to show the current selection candidate
#[derive(Component)]
pub struct SelectionPending {
    pub target: Option<Entity>,
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
struct SelectionBundle {
    sprite: SpriteBundle,
    selection: Selection,
    bounds: SelectionVisualBounds,
    color: SelectionVisualColor,
    cleanup: EditorCleanup,
}

impl SelectionBundle {
    fn from_entity(entity: Entity) -> SelectionBundle {
        SelectionBundle {
            selection: Selection { target: entity },
            sprite: SpriteBundle::default(),
            bounds: SelectionVisualBounds::default(),
            color: SelectionVisualColor::default(),
            cleanup: EditorCleanup,
        }
    }
    fn with_transform(mut self, xf: Transform) -> Self {
        self.sprite.transform = xf;
        self
    }
    fn with_bounds(mut self, rect: Rect) -> Self {
        self.bounds.rect = rect;
        self
    }
    fn with_color(mut self, color: Color) -> Self {
        self.color.0 = color;
        self
    }
}

fn setup_selection_pending(
    mut commands: Commands,
) {
    commands.spawn((
        EditorCleanup,
        SelectionPending { target: None },
        SpriteBundle::default(),
        SelectionVisualBounds::default(),
        SelectionVisualColor::default(),
    ));
}

fn handle_candidate_events(
    mut candidates: ResMut<SelectionCandidates>,
    mut evr_candidate: EventReader<SelectionCandidateEvent>,
) {
    for ev in evr_candidate.iter() {
        match ev {
            SelectionCandidateEvent::Insert { entity, color, bounds } => {
                candidates.candidates.insert(*entity, (*color, *bounds));
            }
            SelectionCandidateEvent::Remove { entity } => {
                candidates.candidates.remove(entity);
            }
        }
    }
}

fn disambiguate_candidates(
    candidates: Res<SelectionCandidates>,
    mut evr_wheel: EventReader<MouseWheel>,
    mut pixel_accum: Local<f32>,
    q_xf: Query<&GlobalTransform, Without<Selected>>,
    mut q_pending: Query<&mut SelectionPending>,
) {
    const PIXEL_SENSITIVITY: f32 = 16.0;

    let mut change = 0;
    for ev in evr_wheel.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                *pixel_accum = 0.0;
                if ev.y > 0.0 {
                    change = 1;
                }
                if ev.y < 0.0 {
                    change = -1;
                }
            }
            MouseScrollUnit::Pixel => {
                *pixel_accum += ev.y;
            }
        }
    }
    if *pixel_accum > PIXEL_SENSITIVITY {
        change = 1;
        *pixel_accum = 0.0;
    }
    if *pixel_accum < -PIXEL_SENSITIVITY {
        change = -1;
        *pixel_accum = 0.0;
    }
    if change != 0 || candidates.is_changed() {
        let mut pending = q_pending.single_mut();
        let mut list: Vec<(f32, Entity)> = candidates.candidates.keys()
            .filter_map(|e| q_xf.get(*e).ok().map(|xf| (xf.translation().z, *e)))
            .collect();
        pending.target = if list.is_empty() {
            None
        } else {
            list.sort_by_key(|(z, _)| FloatOrd(*z));
            let i_old = pending.target.and_then(|target| list.iter().position(|(_, e)| *e == target)).unwrap_or(0);
            let mut i_new = i_old as i32 + change;
            if i_new < 0 {
                i_new = list.len() as i32
            }
            if i_new >= list.len() as i32 {
                i_new = 0;
            }
            Some(list[i_new as usize].1)
        }
    }
}

fn update_pending_visual(
    mut q_visual: Query<(Ref<SelectionPending>, &mut SelectionVisualBounds, &mut SelectionVisualColor)>,
    candidates: Res<SelectionCandidates>,
    mut evr_candidate: EventReader<SelectionCandidateEvent>,
) {
    let mut clear = false;

    let (pending, mut selbounds, mut selcolor) = q_visual.single_mut();
    if pending.is_changed() {
        if let Some(target) = pending.target {
            if let Some((color, rect)) = candidates.candidates.get(&target) {
                selbounds.rect = *rect;
                selcolor.0 = *color;
            } else {
                clear = true;
            }
        } else {
            clear = true;
        }
    }
    if let Some(target) = pending.target {
        for ev in evr_candidate.iter() {
            if let SelectionCandidateEvent::Insert { entity, color, bounds } = ev {
                if *entity == target {
                    selbounds.rect = *bounds;
                    selcolor.0 = *color;
                }
            }
        }
    } else {
        clear = true;
    }

    if clear {
        selbounds.rect = Rect::new(0.0, 0.0, 0.0, 0.0);
        selcolor.0 = Color::NONE;
    } else {
        selcolor.0.set_a(0.25);
    }
}

fn confirm_selection_on_click(
    mut commands: Commands,
    mut q_pending: Query<(&mut SelectionPending, &GlobalTransform, &SelectionVisualBounds, &SelectionVisualColor)>,
    mut evw_candidate: EventWriter<SelectionCandidateEvent>,
) {
    let (mut pending, xf, bounds, color) = q_pending.single_mut();
    if let Some(target) = pending.target {
        let e = commands.spawn(
            SelectionBundle::from_entity(target)
                .with_bounds(bounds.rect)
                .with_color(color.0.with_a(0.5))
                .with_transform(xf.compute_transform())
        ).id();
        commands.entity(target).insert(Selected { selection: e });
        evw_candidate.send(SelectionCandidateEvent::Remove { entity: target });
    }
    pending.target = None;
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
    mut q_selection: Query<(&mut GlobalTransform, &Selection), Without<SelectionPending>>,
    mut q_pending: Query<(&mut GlobalTransform, &SelectionPending), Without<Selection>>,
    q_target: Query<&GlobalTransform, (Without<Selection>, Without<SelectionPending>, Changed<GlobalTransform>)>,
) {
    for (mut xf_sel, sel) in &mut q_selection {
        if let Ok(xf_tgt) = q_target.get(sel.target) {
            *xf_sel = *xf_tgt;
        }
    }
    if let Ok((mut xf_sel, pending)) = q_pending.get_single_mut() {
        if let Some(target) = pending.target {
            if let Ok(xf_tgt) = q_target.get(target) {
                *xf_sel = *xf_tgt;
            }
        }
    }
}

fn deselect_selections_on_click(
    mut commands: Commands,
    candidates: Res<SelectionCandidates>,
    crs: Res<WorldCursor>,
    q_selection: Query<(Entity, &Selection, &GlobalTransform, &SelectionVisualBounds)>,
) {
    // NOTE: assumes .run_if(input_just_pressed(MouseButton::Left))

    if !candidates.candidates.is_empty() {
        return;
    }

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

