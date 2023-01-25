use bevy::input::mouse::MouseMotion;

use crate::crate_prelude::*;
use crate::ui::tooltip::TooltipText;

pub struct PanelPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for PanelPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(self.state.clone(), setup_panel_layer);
        app.add_enter_system(self.state.clone(), spawn_panels);
        app.add_exit_system(self.state.clone(), remove_resource::<PanelLayerEntity>);
        app.add_system_to_stage(CoreStage::PostUpdate, reparent_panels.run_in_state(self.state.clone()));
        app.add_system(panel_focus.run_in_state(self.state.clone()));
        app.add_system(panel_titlebar_drag.run_in_state(self.state.clone()));
        app.add_system(panel_titlebar_collapse.run_in_state(self.state.clone()));
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
    commands.entity(e_contents).push_children(&[label_snap]);
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
            p_style.position.left.try_add_assign(Val::Px(delta.x)).unwrap();
            p_style.position.top.try_add_assign(Val::Px(delta.y)).unwrap();
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
    title: &str,
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
                    top: Val::Px(80.0),
                    left: Val::Px(0.0),
                    bottom: Val::Auto,
                    right: Val::Auto,
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
            text: Text::from_section(title, TextStyle {
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
            image: UiImage(assets.image_ui_smallbutt_depressed.clone()),
            ..Default::default()
        },
        TooltipText {
            title: "Minify".into(),
            text: "The Panel will collapse into a button at the bottom of the screen.\nAlternatively, you can double-click the title to collapse into the titlebar.".into(),
        },
    )).id();
    let mini_icon = commands.spawn((
        ImageBundle {
            focus_policy: FocusPolicy::Pass,
            image: UiImage(assets.image_icon_wm_minify.clone()),
            ..Default::default()
        },
    )).id();
    commands.entity(mini_butt).push_children(&[mini_icon]);
    commands.entity(titlebar).push_children(&[mini_butt, title]);
    commands.entity(container).push_children(&[titlebar, contents]);

    contents
}

