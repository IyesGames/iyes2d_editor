use crate::crate_prelude::*;
use crate::ui::tooltip::TooltipText;

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
    // TOOL BAR
    let container = commands.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..Default::default()
            },
            z_index: ZIndex::Global(9010), // TODO: make this configurable
            ..Default::default()
        },
        EditorCleanup,
    )).id();
    let toolbar = commands.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            style: Style {
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect {
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    right: Val::Px(4.0),
                    bottom: Val::Px(4.0),
                },
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.75, 0.75, 0.75)),
            ..Default::default()
        },
    )).id();
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
    commands.entity(container).push_children(&[toolbar]);
}

fn toolbar_button_image(
    tool: Res<CurrentState<Tool>>,
    assets: Res<EditorAssets>,
    mut query_buttons: Query<(&Interaction, &ToolbarTool, &mut UiImage, Option<&UiInactive>), With<Button>>,
) {
    // PERF: this could use change detection run conditions
    for (interaction, toolbar_tool, mut image, inactive) in &mut query_buttons {
        if toolbar_tool.0 == tool.0 {
            image.0 = assets.image_ui_toolbar_pressed.clone();
        } else if inactive.is_some() {
            image.0 = assets.image_ui_toolbar_disabled.clone();
        } else {
            image.0 = match interaction {
                Interaction::Clicked => assets.image_ui_toolbar_pressed.clone(),
                Interaction::Hovered => assets.image_ui_toolbar_hover.clone(),
                Interaction::None => assets.image_ui_toolbar_depressed.clone(),
            };
        }
    }
}

fn toolbar_button_handler(
    In(tool): In<ToolbarTool>,
    mut commands: Commands,
) {
    commands.insert_resource(NextState(tool.0));
}
