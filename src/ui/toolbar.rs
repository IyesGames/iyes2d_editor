use crate::crate_prelude::*;

pub(crate) struct ToolbarPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for ToolbarPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(self.state.clone(), setup_toolbar);
        app.add_system(
            toolbar_button_image
                .run_in_state(self.state.clone())
        );
        app.add_system(
            butt_handler(toolbar_button_handler)
                .run_in_state(self.state.clone())
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
                image: UiImage(assets.image_ui_toolbar_depressed.clone()),
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
                image: UiImage(assets.logo_small.clone()),
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
                image: UiImage(assets.image_ui_toolbar_depressed.clone()),
                ..Default::default()
            },
            ToolbarTool(tool),
            tool.tooltip(),
        )).id();
        let icon = commands.spawn((
            ImageBundle {
                focus_policy: FocusPolicy::Pass,
                image: UiImage(tool.icon(&*assets)),
                ..Default::default()
            },
        )).id();
        commands.entity(button).push_children(&[icon]);
        commands.entity(toolbar).push_children(&[button]);
    }
}

fn toolbar_button_image(
    tool: Res<CurrentState<Tool>>,
    assets: Res<EditorAssets>,
    mut query_buttons: Query<(&Interaction, &ToolbarTool, &mut UiImage, Option<&UiInactive>), With<Button>>,
) {
    // PERF: this could use change detection run conditions
    for (interaction, toolbar_tool, mut current_image, inactive) in &mut query_buttons {
        if let Ok(handle) = assets.butt_change_image(&current_image.0, *interaction, inactive.is_some(), tool.0 == toolbar_tool.0) {
            current_image.0 = handle;
        }
    }
}

fn toolbar_button_handler(
    In(tool): In<ToolbarTool>,
    mut commands: Commands,
) {
    commands.insert_resource(NextState(tool.0));
}
