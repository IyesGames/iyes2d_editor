use crate::crate_prelude::*;

pub(crate) struct ToolbarPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for ToolbarPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                setup_toolbar,
            ).in_schedule(OnEnter(self.state.clone()))
        );
        app.add_systems(
            (
                toolbar_button_image,
            ).in_set(EditorSet)
        );
    }
}

#[derive(Component, Clone)]
struct ToolbarTool(Tool);

fn setup_toolbar(
    mut commands: Commands,
    assets: Res<EditorAssets>,
) {
    let toolbar = commands.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            z_index: ZIndex::Global(9010), // TODO: make this configurable
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Auto,
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.75, 0.75, 0.75)),
            ..Default::default()
        },
        EditorCleanup,
    )).id();
    // Logo Button
    {
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
        )).id();
        let icon = commands.spawn((
            ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(48.0), Val::Px(48.0)),
                    ..Default::default()
                },
                focus_policy: FocusPolicy::Pass,
                image: UiImage::new(assets.logo_small.clone()),
                ..Default::default()
            },
        )).id();
        commands.entity(button).push_children(&[icon]);
        commands.entity(toolbar).push_children(&[button]);
    }
    // Tool Buttons
    for tool in enum_iterator::all::<Tool>() {
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
            ToolbarTool(tool),
            ClickBehavior::new().entity_system(toolbar_butt_handler),
            tool.tooltip(),
        )).id();
        let icon = commands.spawn((
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                image: UiImage::new(tool.icon(&*assets)),
                ..Default::default()
            },
        )).id();
        commands.entity(button).push_children(&[icon]);
        commands.entity(toolbar).push_children(&[button]);
    }
}

fn toolbar_button_image(
    tool: Res<State<Tool>>,
    assets: Res<EditorAssets>,
    mut query_buttons: Query<(&Interaction, &ToolbarTool, &mut UiImage, Option<&UiDisabled>), With<Button>>,
) {
    // PERF: this could use change detection run conditions
    for (interaction, toolbar_tool, mut current_image, inactive) in &mut query_buttons {
        if let Ok(handle) = assets.butt_change_image(&current_image.texture, *interaction, inactive.is_some(), tool.0 == toolbar_tool.0) {
            current_image.texture = handle;
        }
    }
}

fn toolbar_butt_handler(
    In(entity): In<Entity>,
    q_tool: Query<&ToolbarTool>,
    mut next_state: ResMut<NextState<Tool>>,
) {
    if let Ok(tool) = q_tool.get(entity) {
        next_state.set(tool.0);
    }
}
