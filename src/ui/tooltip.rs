use crate::crate_prelude::*;

pub(crate) struct TooltipPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for TooltipPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system(
            tooltip_spawner
                .run_in_state(self.state.clone())
        );
        app.add_system(
            fixup_tooltippables
                .run_in_state(self.state.clone())
        );
        app.add_system(
            tooltip_despawner
                .run_in_state(self.state.clone())
        );
    }
}

#[derive(Component)]
pub struct TooltipText {
    pub title: String,
    pub text: String,
}

#[derive(Component, Default)]
struct TooltipSpawnTimer {
    timer: Option<Timer>,
}

#[derive(Component)]
struct TooltipDespawnTimer {
    e_linked: Entity,
    timer: Timer,
}

fn compute_tooltip_position(
    window: &Window,
) -> UiRect {
    // FIXME: this (and bevy_ui itself lol) is broken with multiple windows
    if let Some(cursor_position) = window.cursor_position() {
        let mut rect = UiRect::all(Val::Auto);
        if cursor_position.x < window.width() / 2.0 {
            rect.left = Val::Px(cursor_position.x);
        } else {
            rect.right = Val::Px(cursor_position.x);
        }
        if cursor_position.y < window.height() / 2.0 {
            rect.bottom = Val::Px(window.height() - cursor_position.y);
        } else {
            rect.top = Val::Px(window.height() - cursor_position.y);
        }
        rect
    } else {
        UiRect {
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Auto,
            bottom: Val::Auto,
        }
    }
}

fn fixup_tooltippables(
    mut commands: Commands,
    query: Query<Entity, (With<TooltipText>, Without<TooltipSpawnTimer>)>,
) {
    for e in &query {
        commands.entity(e).insert(TooltipSpawnTimer::default());
    }
}

fn tooltip_spawner(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<EditorAssets>,
    windows: Res<Windows>,
    mut query: Query<(Entity, &Interaction, &TooltipText, &mut TooltipSpawnTimer)>,
) {
    for (e, interaction, tooltip_text, mut timer) in &mut query {
        if *interaction == Interaction::Hovered {
            if let Some(timer) = &mut timer.timer {
                timer.tick(time.delta());
                if timer.just_finished() {
                    let outer = commands.spawn((
                        NodeBundle {
                            focus_policy: FocusPolicy::Pass,
                            z_index: ZIndex::Global(9100), // TODO: make this configurable
                            background_color: BackgroundColor(Color::BLACK),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: compute_tooltip_position(windows.primary()),
                                padding: UiRect::all(Val::Px(2.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        EditorCleanup,
                        TooltipDespawnTimer {
                            e_linked: e,
                            timer: Timer::new(Duration::from_millis(2000), TimerMode::Once),
                        },
                    )).id();
                    let inner = commands.spawn(
                        NodeBundle {
                            focus_policy: FocusPolicy::Pass,
                            background_color: BackgroundColor(Color::BEIGE),
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::FlexStart,
                                padding: UiRect::all(Val::Px(4.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ).id();
                    let title = commands.spawn(
                        TextBundle {
                            style: Style {
                                margin: UiRect {
                                    bottom: Val::Px(4.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text::from_section(tooltip_text.title.clone(), TextStyle {
                                font: assets.font_bold.clone(),
                                font_size: 16.0,
                                color: Color::BLACK,
                            }),
                            ..Default::default()
                        }
                    ).id();
                    let text = commands.spawn(
                        TextBundle {
                            text: Text::from_section(tooltip_text.text.clone(), TextStyle {
                                font: assets.font.clone(),
                                font_size: 14.0,
                                color: Color::BLACK,
                            }),
                            ..Default::default()
                        }
                    ).id();
                    commands.entity(outer).push_children(&[inner]);
                    commands.entity(inner).push_children(&[title, text]);
                }
            } else {
                timer.timer = Some(Timer::new(Duration::from_millis(500), TimerMode::Once));
            }
        } else {
            timer.timer = None;
        }
    }
}

fn tooltip_despawner(
    mut commands: Commands,
    time: Res<Time>,
    q_linked: Query<&Interaction>,
    mut q_tooltip: Query<(Entity, &mut TooltipDespawnTimer)>,
) {
    for (e, mut timer) in &mut q_tooltip {
        let mut progressing = true;

        if let Ok(interaction) = q_linked.get(timer.e_linked) {
            if *interaction != Interaction::None {
                progressing = false;
            }
        }

        if progressing {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                commands.entity(e).despawn_recursive();
            }
        }
    }
}
