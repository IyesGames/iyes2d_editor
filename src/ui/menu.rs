use std::sync::Arc;
use parking_lot::Mutex;

use bevy::ecs::system::BoxedSystem;

use crate::crate_prelude::*;

pub struct MenuPlugin<S: StateData> {
    pub state: S,
}

impl<S: StateData> Plugin for MenuPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(self.state.clone(), setup_global_menu);
        app.add_exit_system(self.state.clone(), remove_resource::<GlobalMenuContainer>);
        app.add_system(initialize_menu_actions.label("initialize_menu_actions"));
        app.add_system(
            close_submenus
                .run_in_state(self.state.clone())
                .label("close_submenus")
        );
        app.add_system(
            menu_action_handler
                .run_in_state(self.state.clone())
                .after("initialize_menu_actions")
        );
        app.add_system(
            menu_submenu_handler
                .run_in_state(self.state.clone())
                .after("close_submenus")
        );
    }
}

#[derive(Resource)]
struct GlobalMenuContainer(Entity);

#[derive(Component)]
struct MenuContainer;

#[derive(Component)]
struct SubmenuContainer {
    parent_menu: Entity,
}

#[derive(Component)]
struct MenuItem {
    parent_menu: Entity,
}

#[derive(Component, Clone)]
struct MenuAction(Arc<Mutex<BoxedSystem>>);

#[derive(Component)]
struct MenuSubmenu(Entity);

impl MenuAction {
    fn from_system<S, Param>(system: S) -> Self
        where S: IntoSystem<(), (), Param>
    {
        MenuAction(Arc::new(Mutex::new(Box::new(IntoSystem::into_system(system)))))
    }
}

fn test(
    mut commands: Commands,
    mut x: Local<f32>,
) {
    *x += 150.0;
    commands.spawn(
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 200.0)),
                color: Color::PINK,
                ..Default::default()
            },
            transform: Transform::from_xyz(*x, 0.0, 10.0),
            ..Default::default()
        }
    );
}

fn menu_action_handler(
    world: &mut World,
    q: &mut QueryState<(&Interaction, &MenuAction), Changed<Interaction>>,
) {
    let actions: Vec<_> = q.iter(world)
        .filter(|(interaction, _)| **interaction == Interaction::Clicked)
        .map(|(_, action)| action.0.clone())
        .collect();

    for action in actions {
        let mut system = action.lock();
        system.run((), world);
        system.apply_buffers(world);
    }
}

fn menu_submenu_handler(
    q_item: Query<(&Interaction, &Node, &GlobalTransform, &MenuSubmenu), With<MenuItem>>,
    mut q_submenu: Query<(&mut Style, &mut Visibility, &SubmenuContainer)>,
) {
    // FIXME: this system should have "just_pressed" behavior,
    // but it is annoying to do with UI Interaction + Change Detection

    for (interaction, node, transform, submenu) in &q_item {
        if *interaction != Interaction::Clicked {
            continue;
        }
        // found a clicked submenu item!

        // position the submenu we are trying to show, relative to the item node
        if let Ok((mut style, _, _)) = q_submenu.get_mut(submenu.0) {
            // TODO: support right-to-left menu expansion
            style.position_type = PositionType::Absolute;
            style.position = UiRect {
                top: Val::Px(transform.translation().y - node.size().y / 2.0),
                left: Val::Px(transform.translation().x + node.size().x / 2.0),
                bottom: Val::Auto,
                right: Val::Auto,
            };
        } else {
            // something is wrong, but let's not panic
            error!("Submenu hierarchy is broken.");
            continue;
        }

        // hide all submenus
        for (_, mut visibility, _) in &mut q_submenu {
            visibility.is_visible = false;
        }

        // show the chain of submenus we are a part of;
        // start from our submenu we want to open and walk up the tree
        let mut e_menu = submenu.0;
        while let Ok((_, mut visibility, container)) = q_submenu.get_mut(e_menu) {
            visibility.is_visible = true;
            e_menu = container.parent_menu;
        }

        // only handle one submenu activation
        break;
    }
}

fn close_submenus(
    btn: Res<Input<MouseButton>>,
    mut q_submenu: Query<&mut Visibility, With<SubmenuContainer>>,
) {
    if btn.just_pressed(MouseButton::Left) {
        for mut visibility in &mut q_submenu {
            visibility.is_visible = false;
        }
    }
}

fn initialize_menu_actions(
    world: &mut World,
    q: &mut QueryState<&MenuAction, Added<MenuAction>>,
) {
    let actions: Vec<_> = q.iter(world)
        .map(|action| action.0.clone())
        .collect();

    for action in actions {
        action.lock().initialize(world);
    }
}

fn setup_global_menu(
    mut commands: Commands,
    assets: Res<EditorAssets>,
) {
    let menu = spawn_menu(&mut commands, &*assets, UiRect {
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        right: Val::Auto,
        bottom: Val::Auto,
    }, true);
    let (_, app_submenu) = spawn_menuitem_submenu(&mut commands, &*assets, menu, "Iyes2D Editor");
    let (_, submenu2) = spawn_menuitem_submenu(&mut commands, &*assets, menu, "Menu 2");
    let (_, submenu3) = spawn_menuitem_submenu(&mut commands, &*assets, app_submenu, "Menu 3");
    spawn_menuitem_action(&mut commands, &*assets, app_submenu, "Test", test);
    spawn_menuitem_action(&mut commands, &*assets, submenu2, "Test 2", test);
    spawn_menuitem_action(&mut commands, &*assets, submenu3, "Test 3", test);
    commands.insert_resource(GlobalMenuContainer(menu));
}

pub fn spawn_menu(
    commands: &mut Commands,
    assets: &EditorAssets,
    position: UiRect,
    is_visible: bool,
) -> Entity {
    let menu = commands.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Pass,
            background_color: BackgroundColor(Color::WHITE),
            z_index: ZIndex::Global(9010),
            style: Style {
                position_type: PositionType::Absolute,
                position,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            visibility: Visibility {
                is_visible,
            },
            ..Default::default()
        },
        MenuContainer,
        EditorCleanup,
    )).id();
    menu
}

/// Spawn a menu item for a submenu
///
/// Returns (item_entity, menu_entity).
pub fn spawn_menuitem_submenu(
    commands: &mut Commands,
    assets: &EditorAssets,
    parent_menu: Entity,
    text_str: &str,
) -> (Entity, Entity) {
    let submenu = spawn_menu(commands, assets, UiRect::all(Val::Auto), false);
    let item = spawn_menuitem_helper(commands, assets, parent_menu, true, text_str);
    commands.entity(item).insert(MenuSubmenu(submenu));
    commands.entity(submenu).insert(SubmenuContainer { parent_menu });
    (item, submenu)
}

/// Spawn a menu item for an action
pub fn spawn_menuitem_action<S, P>(
    commands: &mut Commands,
    assets: &EditorAssets,
    parent_menu: Entity,
    text_str: &str,
    action: S,
) -> Entity
    where S: IntoSystem<(), (), P>
{
    let item = spawn_menuitem_helper(commands, assets, parent_menu, false, text_str);
    commands.entity(item).insert(MenuAction::from_system(action));
    item
}

fn spawn_menuitem_helper(
    commands: &mut Commands,
    assets: &EditorAssets,
    parent_menu: Entity,
    is_submenu: bool,
    text_str: &str,
) -> Entity {
    let item = commands.spawn((
        ButtonBundle {
            focus_policy: FocusPolicy::Block,
            background_color: BackgroundColor(Color::BLACK),
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(2.0)),
                margin: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        MenuItem {
            parent_menu,
        },
        EditorCleanup,
    )).id();
    let text = commands.spawn((
        TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(4.0),
                    right: Val::Px(4.0),
                    top: Val::Auto,
                    bottom: Val::Auto,
                },
                ..Default::default()
            },
            text: Text::from_section(text_str, TextStyle {
                color: Color::WHITE,
                font: assets.font.clone(),
                font_size: 14.0,
            }),
            ..Default::default()
        },
    )).id();
    commands.entity(item).push_children(&[text]);
    if is_submenu {
        let submenu_indicator = commands.spawn((
            NodeBundle {
                focus_policy: FocusPolicy::Pass,
                ..Default::default()
            },
        )).id();
        let submenu_indicator_text = commands.spawn((
            TextBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Px(8.0),
                        right: Val::Auto,
                        top: Val::Auto,
                        bottom: Val::Auto,
                    },
                    ..Default::default()
                },
                text: Text::from_section(">", TextStyle {
                    color: Color::WHITE,
                    font: assets.font_bold.clone(),
                    font_size: 16.0,
                }),
                ..Default::default()
            },
        )).id();
        commands.entity(submenu_indicator).push_children(&[submenu_indicator_text]);
        commands.entity(item).push_children(&[submenu_indicator]);
    }
    commands.entity(parent_menu).push_children(&[item]);
    item
}
