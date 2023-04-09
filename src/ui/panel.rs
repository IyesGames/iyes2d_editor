use bevy::input::mouse::MouseMotion;

use crate::crate_prelude::*;
use crate::ui::tooltip::TooltipText;

use super::SimpleButtVisual;

pub struct PanelPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for PanelPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                setup_panel_layer,
                setup_minibar,
                spawn_panels,
            ).in_schedule(OnEnter(self.state.clone()))
        );
        app.add_systems(
            (
                remove_resource::<PanelLayerEntity>,
            ).in_schedule(OnExit(self.state.clone()))
        );
        app.add_systems(
            (
                reparent_panels.after(EditorFlush),
                panel_focus,
                panel_titlebar_drag,
                panel_titlebar_collapse,
            ).in_set(EditorSet)
        );
    }
}

/// All panels are to be spawned under a common parent entity (the "layer")
/// for easier Z-order management when focusing
#[derive(Resource)]
struct PanelLayerEntity(Entity);

/// Marker for panels
#[derive(Component)]
struct PanelEntity {
    titlebar: Entity,
    contents: Entity,
}

/// Marker for panels' content areas
#[derive(Component)]
struct PanelContentsEntity {
    panel: Entity,
}

/// Marker for panels' titlebars
#[derive(Component)]
struct PanelTitlebarEntity {
    panel: Entity,
    contents: Entity,
}

#[derive(Component)]
struct PanelTitlebarDoubleclick {
    timer: Timer,
}

fn setup_panel_layer(
    mut commands: Commands,
) {
    let layer = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.0)),
                ..Default::default()
            },
            z_index: ZIndex::Global(9001), // TODO: make this configurable
            ..Default::default()
        },
        EditorCleanup,
    )).id();
    commands.insert_resource(PanelLayerEntity(layer));
}

fn reparent_panels(world: &mut World) {
    let e_layer = world.resource::<PanelLayerEntity>().0;
    let mut q: QueryState<(Entity, Option<&Parent>), With<PanelEntity>> = QueryState::new(world);
    let mut buf = vec![];
    for (e, parent) in q.iter(world) {
        let mut good = false;
        if let Some(parent) = parent {
            if parent.get() == e_layer {
                good = true;
            }
        }
        if !good {
            buf.push(e);
        }
    }
    if !buf.is_empty() {
        debug!("Reparenting panels: {:?}", &buf);
        world.entity_mut(e_layer).push_children(&buf);
    }
}

fn spawn_panels(
    mut commands: Commands,
    assets: Res<EditorAssets>,
) {
    let e_contents = spawn_panel(&mut commands, &*assets, "Tool Options");
    let label_snap = commands.spawn((
        TextBundle {
            text: Text::from_section("Snap to Grid:", TextStyle {
                font: assets.font.clone(),
                font_size: 12.0,
                color: Color::BLACK,
            }),
            ..Default::default()
        },
    )).id();
    let label_us = commands.spawn((
        TextBundle {
            text: Text::from_section("Uniform Scaling is currently the preferred mode of operation.", TextStyle {
                font: assets.font.clone(),
                font_size: 12.0,
                color: Color::BLACK,
            }),
            ..Default::default()
        },
    )).id();
    commands.entity(e_contents).push_children(&[label_snap, label_us]);
    let e_contents = spawn_panel(&mut commands, &*assets, "About Editor");
    let label_ver = commands.spawn((
        TextBundle {
            text: Text::from_section(format!("Editor Version: {}", env!("CARGO_PKG_VERSION")), TextStyle {
                font: assets.font.clone(),
                font_size: 12.0,
                color: Color::BLACK,
            }),
            ..Default::default()
        },
    )).id();
    commands.entity(e_contents).push_children(&[label_ver]);
}

fn panel_focus(
    q_titlebar: Query<(&Interaction, &PanelTitlebarEntity)>,
    mut q_panel: Query<&mut ZIndex, With<PanelEntity>>,
) {
    let mut new_focus = None;
    for (interaction, titlebar) in &q_titlebar {
        if *interaction == Interaction::Clicked {
            new_focus = Some(titlebar.panel);
        }
    }
    if let Some(new_focus) = new_focus {
        let (mut min_z, mut max_z) = (i32::MAX, i32::MIN);
        for z in &q_panel {
            if let ZIndex::Local(z) = z {
                if *z < min_z {
                    min_z = *z;
                }
                if *z > max_z {
                    max_z = *z;
                }
            }
        }
        if let Ok(mut z) = q_panel.get_mut(new_focus) {
            *z = ZIndex::Local(max_z + 1);
        }
        for mut z in &mut q_panel {
            if let ZIndex::Local(z) = &mut *z {
                *z -= min_z;
            } else {
                *z = ZIndex::Local(0);
            }
        }
    }
}

fn panel_titlebar_collapse(
    time: Res<Time>,
    mut q_titlebar: Query<(&Interaction, &PanelTitlebarEntity, &mut PanelTitlebarDoubleclick)>,
    mut q_contents: Query<&mut Style, With<PanelContentsEntity>>,
) {
    for (interaction, titlebar, mut dblclick) in &mut q_titlebar {
        if *interaction == Interaction::Clicked {
            // if timer is already running, detect double click
            let progress = dblclick.timer.percent();
            if progress > 0.0 && progress < 1.0 {
                // started but not finished, successful doubleclick
                let mut style = q_contents.get_mut(titlebar.contents).unwrap();
                if style.display == Display::None {
                    style.display = Display::Flex;
                } else {
                    style.display = Display::None;
                }
            }
            // start new timer
            dblclick.timer = Timer::new(Duration::from_millis(125), TimerMode::Once);
        } else {
            // tick only when no click
            dblclick.timer.tick(time.delta());
        }
    }
}

fn panel_titlebar_drag(
    mut mousemotion: EventReader<MouseMotion>,
    q_titlebar: Query<(&Interaction, &PanelTitlebarEntity)>,
    mut q_panel: Query<&mut Style, With<PanelEntity>>,
) {
    for (interaction, titlebar) in &q_titlebar {
        if *interaction == Interaction::Clicked {
            let mut delta = Vec2::ZERO;
            for ev in mousemotion.iter() {
                delta += ev.delta;
            }
            let mut p_style = q_panel.get_mut(titlebar.panel).unwrap();
            p_style.position.left.try_add_assign(Val::Px(delta.x)).ok();
            p_style.position.right.try_sub_assign(Val::Px(delta.x)).ok();
            p_style.position.top.try_add_assign(Val::Px(delta.y)).ok();
            p_style.position.bottom.try_sub_assign(Val::Px(delta.y)).ok();
        }
    }
}

/// Helper function to create a Panel
///
/// Returns tuple of the entity ids of the panel itself and its content area
/// The content area is where the caller
/// can populate the panel with UI elements.
///
/// The panel entity it
pub fn spawn_panel(
    commands: &mut Commands,
    assets: &EditorAssets,
    title_str: &str,
) -> Entity {
    let container = commands.spawn(()).id();
    let titlebar = commands.spawn(()).id();
    let contents = commands.spawn(()).id();
    commands.entity(container).insert((
        NodeBundle {
            // focus_policy: FocusPolicy::Block,
            background_color: BackgroundColor(Color::WHITE),
            style: Style {
                padding: UiRect::all(Val::Px(2.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.0),
                    right: Val::Px(64.0),
                    bottom: Val::Auto,
                    left: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            z_index: ZIndex::Local(0),
            ..Default::default()
        },
        PanelEntity {
            titlebar, contents,
        },
        EditorCleanup,
    ));
    commands.entity(titlebar).insert((
        NodeBundle {
            // focus_policy: FocusPolicy::Block,
            background_color: BackgroundColor(Color::BLACK),
            style: Style {
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            ..Default::default()
        },
        Interaction::default(),
        PanelTitlebarDoubleclick {
            timer: Timer::new(Duration::ZERO, TimerMode::Once),
        },
        PanelTitlebarEntity {
            panel: container,
            contents,
        },
    ));
    commands.entity(contents).insert((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            background_color: BackgroundColor(Color::rgb(0.75, 0.75, 0.75)),
            style: Style {
                flex_grow: 1.0,
                padding: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        },
        PanelContentsEntity {
            panel: container,
        }
    ));
    let title = commands.spawn(
        TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(4.0),
                    right: Val::Px(4.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(2.0),
                },
                ..Default::default()
            },
            text: Text::from_section(title_str, TextStyle {
                font: assets.font_bold.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            }),
            ..Default::default()
        }
    ).id();
    let mini_butt = commands.spawn((
        ButtonBundle {
            style: Style {
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Px(16.0), Val::Px(16.0)),
                ..Default::default()
            },
            image: UiImage::new(assets.image_ui_smallbutt_depressed.clone()),
            ..Default::default()
        },
        MiniButt {
            panel: container,
            title: title_str.into(),
        },
        ClickBehavior::new().entity_system(mini_butt_handler),
        SimpleButtVisual,
        TooltipText {
            title: "Minify".into(),
            text: "The Panel will collapse into a button at the bottom of the screen.\nAlternatively, you can double-click the title to collapse into the titlebar.".into(),
        },
    )).id();
    let mini_icon = commands.spawn((
        ImageBundle {
            focus_policy: FocusPolicy::Pass,
            image: UiImage::new(assets.image_icon_wm_minify.clone()),
            ..Default::default()
        },
    )).id();
    let close_butt = commands.spawn((
        ButtonBundle {
            style: Style {
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Px(16.0), Val::Px(16.0)),
                ..Default::default()
            },
            image: UiImage::new(assets.image_ui_smallbutt_depressed.clone()),
            ..Default::default()
        },
        CloseButt {
            panel: container,
        },
        ClickBehavior::new().entity_system(close_butt_handler),
        SimpleButtVisual,
        TooltipText {
            title: "Close".into(),
            text: "The Panel will disappear.".into(),
        },
    )).id();
    let close_icon = commands.spawn((
        ImageBundle {
            focus_policy: FocusPolicy::Pass,
            image: UiImage::new(assets.image_icon_wm_close.clone()),
            ..Default::default()
        },
    )).id();
    commands.entity(close_butt).push_children(&[close_icon]);
    commands.entity(mini_butt).push_children(&[mini_icon]);
    commands.entity(titlebar).push_children(&[mini_butt, title, close_butt]);
    commands.entity(container).push_children(&[titlebar, contents]);

    contents
}

#[derive(Component)]
struct MinibarTop;

#[derive(Component, Clone)]
struct MinibarButt {
    panel: Entity,
}

#[derive(Component, Clone)]
struct MiniButt {
    panel: Entity,
    title: String,
}

#[derive(Component, Clone)]
struct CloseButt {
    panel: Entity,
}

fn setup_minibar(
    mut commands: Commands,
    _assets: Res<EditorAssets>,
) {
    let _minibar = commands.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            z_index: ZIndex::Global(9010), // TODO: make this configurable
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    top: Val::Auto,
                    right: Val::Auto,
                },
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.75, 0.75, 0.75)),
            ..Default::default()
        },
        MinibarTop,
        EditorCleanup,
    )).id();
}

fn mini_butt_handler(
    In(entity): In<Entity>,
    mut commands: Commands,
    assets: Res<EditorAssets>,
    q_butt: Query<&MiniButt>,
    mut q_panel: Query<&mut Visibility, With<PanelEntity>>,
    q_minibar: Query<Entity, With<MinibarTop>>,
) {
    let Ok(butt) = q_butt.get(entity) else { return; };
    // hide the panel (by visibility, keep flex layout)
    *q_panel.get_mut(butt.panel).unwrap() = Visibility::Hidden;
    // create a minibar button for restoring it
    for e_minibar in &q_minibar {
        let button = commands.spawn((
            ButtonBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                    ..Default::default()
                },
                image: UiImage::new(assets.image_ui_toolbar_depressed.clone()),
                ..Default::default()
            },
            MinibarButt {
                panel: butt.panel,
            },
            ClickBehavior::new().entity_system(minibar_butt_handler),
            SimpleButtVisual,
            TooltipText {
                title: butt.title.clone(),
                text: "Click to re-open the Panel.".into(),
            },
        )).id();
        // construct the string using initials from the title
        let mut minitext_str = String::new();
        minitext_str.push('[');
        for word in butt.title.split_whitespace() {
            if !word.is_empty() {
                for c in word.chars().next().unwrap().to_uppercase() {
                    minitext_str.push(c);
                }
            }
        }
        minitext_str.push(']');
        let minitext = commands.spawn((
            TextBundle {
                text: Text::from_section(minitext_str, TextStyle {
                    font: assets.font_bold.clone(),
                    font_size: 16.0,
                    color: Color::BLACK,
                }),
                ..Default::default()
            },
        )).id();
        commands.entity(button).push_children(&[minitext]);
        commands.entity(e_minibar).push_children(&[button]);
    }
}

fn minibar_butt_handler(
    In(entity): In<Entity>,
    mut commands: Commands,
    q_butt: Query<&MinibarButt>,
    mut q_panel: Query<&mut Visibility, With<PanelEntity>>,
    q_minibar: Query<(Entity, &MinibarButt)>,
) {
    let Ok(butt) = q_butt.get(entity) else { return; };
    // despawn any minibar buttons
    for (e, minibutt) in &q_minibar {
        if minibutt.panel == butt.panel {
            commands.entity(e).despawn_recursive();
        }
    }
    // unhide the panel
    *q_panel.get_mut(butt.panel).unwrap() = Visibility::Visible;
}

fn close_butt_handler(
    In(entity): In<Entity>,
    q_butt: Query<&CloseButt>,
    mut commands: Commands,
) {
    let Ok(butt) = q_butt.get(entity) else { return; };
    commands.entity(butt.panel).despawn_recursive();
}
