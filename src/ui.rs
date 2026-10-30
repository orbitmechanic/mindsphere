use std::collections::{HashMap, HashSet};

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{AutoFocus, InputFocus};
use bevy::picking::events::PointerDragStart;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::Pressed;
use bevy::ui_widgets::{
    ActivateOnPress, Button, Slider, SliderRange, SliderThumb, SliderValue, TextInput, TrackClick,
    ValueChange,
};

use crate::model::GROUP_COLORS;
use crate::scene::OrbitCamera;
use crate::skins::{BACKGROUNDS, SKINS, background_label, skin_label, skin_swatch};
use crate::state::{AppAction, AppState, ModalKind, Tone, ViewMode};
use crate::util::{
    ACCENT, AMBER, BOTTOM_DOCK_H, CORAL, FAINT, INK, LEFT_PANEL_W, LINE, MINT, MUTED, PANEL,
    RIGHT_PANEL_W, TOP_BAR_H, alpha, color_from_hex,
};

const CARD_BG: Color = Color::srgba(0.043, 0.075, 0.125, 0.98);
const INPUT_BG: Color = Color::srgba(0.02, 0.04, 0.07, 1.0);
const INPUT_LINE: Color = Color::srgba(0.12, 0.18, 0.27, 1.0);
const ACCENT_SOFT: Color = Color::srgba(0.20, 0.62, 0.70, 0.24);

#[derive(Resource)]
pub struct UiAssets {
    pub regular: Handle<Font>,
    pub medium: Handle<Font>,
    pub bold: Handle<Font>,
}

#[derive(Component)]
pub struct SearchInput;

#[derive(Component)]
pub struct SphereListContent;

#[derive(Component)]
pub struct SphereListScroll;

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component)]
pub struct NodeLabelsRoot;

#[derive(Component)]
pub struct NodeLabel(pub String);

#[derive(Resource, Default)]
pub struct NodeLabelMap {
    pub entities: HashMap<String, Entity>,
}

#[derive(Component)]
pub struct BoardColumns;

#[derive(Component)]
pub struct BoardScroll;

#[derive(Component)]
pub struct BoardColumn(pub String);

#[derive(Component)]
pub struct BoardColumnHandle(pub String);

#[derive(Component)]
pub struct BoardCard(pub String);

#[derive(Resource, Default)]
pub struct BoardDragState {
    pub group: Option<String>,
    pub node: Option<String>,
    pub entity: Option<Entity>,
}

#[derive(Component)]
pub struct InspectorRoot;

#[derive(Component)]
pub struct GlobePanelRoot;

#[derive(Component)]
pub struct GlobePanelScroll;

#[derive(Component)]
pub struct ModalRoot;

#[derive(Component)]
pub struct ModalContent;

#[derive(Component)]
pub struct ModalInput;

#[derive(Component)]
pub struct TitleInput;

#[derive(Component)]
pub struct BodyInput;

#[derive(Component)]
pub struct ToastRoot;

#[derive(Component)]
pub struct ToastPanel;

#[derive(Component)]
pub struct ToastText;

#[derive(Component)]
pub struct TopTitle;

#[derive(Component)]
pub struct TopMeta;

#[derive(Component)]
pub struct SaveLabel;

#[derive(Component)]
pub struct SaveDot;

#[derive(Component)]
pub struct ModeButton(pub ViewMode);

#[derive(Component)]
pub struct NewThoughtButton;

#[derive(Component)]
pub struct AddGroupButton;

#[derive(Component)]
pub struct RemoveGroupButton;

#[derive(Component)]
pub struct LinkButton;

#[derive(Component)]
pub struct MultiButton;

#[derive(Component)]
pub struct ConsoleLine1;

#[derive(Component)]
pub struct ConsoleLine2;

#[derive(Component, Clone, Copy)]
pub enum StatKind {
    Thoughts,
    Links,
    Groups,
}

#[derive(Component)]
pub struct StatCount(pub StatKind);

#[derive(Component)]
pub struct ActionButton(pub AppAction);

#[derive(Component)]
pub struct GlobeRadiusSlider;

#[derive(Component)]
pub struct GlobeTransparencySlider;

#[derive(Component)]
pub struct GlobeSliderThumb;

#[derive(Component)]
pub struct GlobeRadiusValue;

#[derive(Component)]
pub struct GlobeTransparencyValue;

#[derive(Component, Clone, Copy)]
pub struct ButtonStyle {
    pub normal: Color,
    pub hover: Color,
    pub pressed: Color,
    pub border: Color,
}

#[derive(Clone, Copy)]
enum ButtonKind {
    Primary,
    Ghost,
    Chip,
}

impl ButtonKind {
    fn style(self) -> ButtonStyle {
        match self {
            Self::Primary => ButtonStyle {
                normal: Color::srgba(0.35, 0.85, 0.91, 1.0),
                hover: Color::srgba(0.46, 0.92, 0.97, 1.0),
                pressed: Color::srgba(0.27, 0.73, 0.79, 1.0),
                border: Color::srgba(0.35, 0.85, 0.91, 1.0),
            },
            Self::Ghost => ButtonStyle {
                normal: Color::srgba(0.05, 0.09, 0.15, 0.9),
                hover: Color::srgba(0.08, 0.14, 0.22, 1.0),
                pressed: Color::srgba(0.04, 0.07, 0.12, 1.0),
                border: Color::srgba(0.11, 0.17, 0.26, 1.0),
            },
            Self::Chip => ButtonStyle {
                normal: Color::srgba(0.03, 0.06, 0.10, 0.95),
                hover: Color::srgba(0.06, 0.11, 0.17, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border: Color::srgba(0.12, 0.19, 0.28, 1.0),
            },
        }
    }
}

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fonts = UiAssets {
        regular: asset_server.load("fonts/NotoSans-Regular.ttf"),
        medium: asset_server.load("fonts/NotoSans-Medium.ttf"),
        bold: asset_server.load("fonts/NotoSans-Bold.ttf"),
    };
    commands.insert_resource(UiAssets {
        regular: fonts.regular.clone(),
        medium: fonts.medium.clone(),
        bold: fonts.bold.clone(),
    });

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Name::new("UI root"),
        ))
        .with_children(|root| {
            spawn_board(root, &fonts);
            spawn_node_labels_root(root);
            spawn_left_panel(root, &fonts);
            spawn_top_bar(root, &fonts);
            spawn_right_panel(root, &fonts);
            spawn_globe_panel(root);
            spawn_bottom_dock(root, &fonts);
            spawn_console(root, &fonts);
            spawn_toast(root, &fonts);
            spawn_modal(root, &fonts);
        });
}

fn spawn_node_labels_root(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            ..default()
        },
        NodeLabelsRoot,
        Pickable::IGNORE,
        Name::new("Node labels"),
    ));
}

fn spawn_left_panel(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            width: px(LEFT_PANEL_W),
            padding: UiRect::all(px(16.0)),
            border: UiRect::right(px(1.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12.0),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(LINE),
        Name::new("Left panel"),
    ))
    .with_children(|panel| {
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10.0),
                ..default()
            })
            .with_children(|brand| {
                brand
                    .spawn((
                        Node {
                            width: px(38.0),
                            height: px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: px(1.0).all(),
                            border_radius: BorderRadius::all(px(11.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.28, 0.34, 0.55)),
                        BorderColor::all(Color::srgba(0.35, 0.85, 0.91, 0.28)),
                    ))
                    .with_children(|icon| {
                        icon.spawn((
                            Node {
                                width: px(16.0),
                                height: px(16.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: px(1.0).all(),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BorderColor::all(ACCENT),
                        ))
                        .with_child((
                            Node {
                                width: px(5.0),
                                height: px(5.0),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(ACCENT),
                        ));
                    });
                brand
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(1.0),
                        ..default()
                    })
                    .with_children(|copy| {
                        copy.spawn((
                            Text::new("Mindsphere"),
                            text_font(fonts, 22.0, true),
                            TextColor(INK),
                            TextShadow::default(),
                        ));
                        copy.spawn((
                            Text::new("Think on a sphere"),
                            text_font(fonts, 10.0, false),
                            TextColor(FAINT),
                        ));
                    });
            });

        panel.spawn(divider());

        panel
            .spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::top(px(2.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("YOUR SPHERES"),
                    text_font(fonts, 10.0, true),
                    TextColor(MUTED),
                ));
                spawn_icon_button(row, fonts, "+", AppAction::NewSphere);
            });

        panel.spawn((
            Text::new("SEARCH"),
            text_font(fonts, 9.5, true),
            TextColor(FAINT),
        ));
        spawn_text_input(
            panel,
            fonts,
            "",
            false,
            36.0,
            (SearchInput, Name::new("Sphere search")),
        );

        panel
            .spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                SphereListScroll,
                Name::new("Sphere list"),
            ))
            .with_children(|list| {
                list.spawn((
                    Node {
                        width: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        ..default()
                    },
                    SphereListContent,
                ));
            });

        panel.spawn(divider());

        panel
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(7.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: px(7.0),
                        height: px(7.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(MINT),
                ));
                row.spawn((
                    Text::new("Stored on this device"),
                    text_font(fonts, 11.0, false),
                    TextColor(MUTED),
                ));
                row.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                row.spawn((
                    Text::new("No cloud"),
                    text_font(fonts, 10.0, false),
                    TextColor(FAINT),
                ));
            });

        panel
            .spawn(Node {
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    fonts,
                    "Import",
                    ButtonKind::Ghost,
                    AppAction::ImportSphere,
                    Some(118.0),
                    34.0,
                );
                spawn_button(
                    row,
                    fonts,
                    "Export",
                    ButtonKind::Ghost,
                    AppAction::ExportSphere,
                    Some(118.0),
                    34.0,
                );
            });
    });
}

fn spawn_top_bar(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(LEFT_PANEL_W),
            right: px(RIGHT_PANEL_W),
            top: px(0.0),
            height: px(TOP_BAR_H),
            padding: UiRect::horizontal(px(22.0)),
            border: UiRect::bottom(px(1.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.035, 0.06, 0.78)),
        BorderColor::all(LINE),
        Name::new("Top bar"),
    ))
    .with_children(|bar| {
        bar.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(1.0),
            justify_content: JustifyContent::Center,
            flex_grow: 1.0,
            ..default()
        })
        .with_children(|copy| {
            copy.spawn((
                Text::new("Thinking it through"),
                text_font(fonts, 24.0, true),
                TextColor(INK),
                TopTitle,
                TextShadow::default(),
            ));
            copy.spawn((
                Text::new("0 thoughts · 0 connections · 0 groups"),
                text_font(fonts, 10.5, false),
                TextColor(MUTED),
                TopMeta,
            ));
        });

        bar.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            margin: UiRect::right(px(4.0)),
            ..default()
        })
        .with_children(|save| {
            save.spawn((
                Node {
                    width: px(6.0),
                    height: px(6.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(MINT),
                SaveDot,
            ));
            save.spawn((
                Text::new("Saved locally"),
                text_font(fonts, 10.5, false),
                TextColor(MUTED),
                SaveLabel,
            ));
        });

        let new_thought = spawn_button(
            bar,
            fonts,
            "+ New thought",
            ButtonKind::Primary,
            AppAction::NewThought,
            Some(136.0),
            38.0,
        );
        bar.commands().entity(new_thought).insert(NewThoughtButton);
        let add_group = spawn_button(
            bar,
            fonts,
            "Add to group",
            ButtonKind::Primary,
            AppAction::AddSelectedToGroup,
            Some(132.0),
            38.0,
        );
        bar.commands().entity(add_group).insert(AddGroupButton);
        let remove_group = spawn_button(
            bar,
            fonts,
            "Remove group",
            ButtonKind::Ghost,
            AppAction::RemoveSelectedFromGroup,
            Some(132.0),
            38.0,
        );
        bar.commands()
            .entity(remove_group)
            .insert(RemoveGroupButton);
        spawn_button(
            bar,
            fonts,
            "Globe",
            ButtonKind::Ghost,
            AppAction::OpenModal(ModalKind::Globe),
            Some(64.0),
            38.0,
        );
        spawn_button(
            bar,
            fonts,
            "More",
            ButtonKind::Ghost,
            AppAction::More,
            Some(66.0),
            38.0,
        );
    });
}

fn spawn_right_panel(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            width: px(RIGHT_PANEL_W),
            padding: UiRect::all(px(18.0)),
            border: UiRect::left(px(1.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(LINE),
        Name::new("Right panel"),
    ))
    .with_children(|panel| {
        panel
            .spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("OVERVIEW"),
                    text_font(fonts, 10.0, true),
                    TextColor(MUTED),
                ));
                spawn_icon_button(row, fonts, "?", AppAction::OpenModal(ModalKind::Shortcuts));
            });

        panel.spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(12.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            InspectorRoot,
            Name::new("Inspector"),
        ));

        panel
            .spawn(Node {
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    fonts,
                    "+ New thought",
                    ButtonKind::Primary,
                    AppAction::NewThought,
                    Some(180.0),
                    40.0,
                );
                spawn_button(
                    row,
                    fonts,
                    "+ Group",
                    ButtonKind::Ghost,
                    AppAction::OpenModal(ModalKind::NewGroup),
                    Some(110.0),
                    40.0,
                );
            });
    });
}

fn spawn_bottom_dock(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(LEFT_PANEL_W),
            right: px(RIGHT_PANEL_W),
            bottom: px(0.0),
            height: px(BOTTOM_DOCK_H),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            ..default()
        },
        Name::new("Bottom dock"),
    ))
    .with_children(|dock| {
        spawn_mode_button(dock, fonts, "Sphere 3D", ViewMode::Sphere);
        spawn_mode_button(dock, fonts, "Board 2D", ViewMode::Board);
        let link = spawn_button(
            dock,
            fonts,
            "Link",
            ButtonKind::Ghost,
            AppAction::ToggleLinkMode,
            Some(58.0),
            32.0,
        );
        dock.commands().entity(link).insert(LinkButton);
        let multi = spawn_button(
            dock,
            fonts,
            "Multi",
            ButtonKind::Ghost,
            AppAction::ToggleMultiSelect,
            Some(58.0),
            32.0,
        );
        dock.commands().entity(multi).insert(MultiButton);
        dock.spawn((
            Text::new("Drag to orbit · Scroll to zoom · Drag a thought to move it"),
            text_font(fonts, 10.5, false),
            TextColor(FAINT),
        ));
    });
}

fn spawn_board(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(LEFT_PANEL_W),
            right: px(RIGHT_PANEL_W),
            top: px(TOP_BAR_H),
            bottom: px(BOTTOM_DOCK_H),
            padding: UiRect::all(px(22.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(14.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgb(0.012, 0.025, 0.045)),
        BoardRoot,
        Name::new("Board view"),
    ))
    .with_children(|board| {
        board
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            })
            .with_children(|head| {
                head.spawn((
                    Text::new("BOARD VIEW"),
                    text_font(fonts, 10.0, true),
                    TextColor(ACCENT),
                ));
                head.spawn((
                    Text::new("Groups become columns. Pick a card to inspect it."),
                    text_font(fonts, 12.0, false),
                    TextColor(MUTED),
                ));
            });
        board.spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: px(14.0),
                overflow: Overflow::scroll_x(),
                ..default()
            },
            ScrollPosition::default(),
            BoardColumns,
            BoardScroll,
            Name::new("Board columns"),
        ));
    });
}

fn spawn_globe_panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            width: px(360.0),
            padding: UiRect::all(px(14.0)),
            border: UiRect::left(px(1.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            overflow: Overflow::scroll_y(),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.027, 0.043, 0.075, 0.995)),
        BorderColor::all(LINE),
        ZIndex(60),
        ScrollPosition::default(),
        GlobePanelRoot,
        GlobePanelScroll,
        Name::new("Globe panel"),
    ));
}

fn spawn_console(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(LEFT_PANEL_W),
            right: px(RIGHT_PANEL_W),
            bottom: px(BOTTOM_DOCK_H + 2.0),
            height: px(42.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        Name::new("Console"),
    ))
    .with_children(|console| {
        console
            .spawn((
                Node {
                    width: px(640.0),
                    padding: UiRect::axes(px(11.0), px(6.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2.0),
                    border: px(1.0).all(),
                    border_radius: BorderRadius::all(px(7.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.03, 0.05, 0.94)),
                BorderColor::all(alpha(ACCENT, 0.18)),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Select a thought or drag the globe."),
                    text_font(fonts, 10.5, false),
                    TextColor(INK),
                    ConsoleLine1,
                ));
                panel.spawn((
                    Text::new("N for a new thought · Globe for surface controls"),
                    text_font(fonts, 9.5, false),
                    TextColor(MUTED),
                    ConsoleLine2,
                ));
            });
    });
}

fn spawn_toast(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(LEFT_PANEL_W),
            right: px(RIGHT_PANEL_W),
            bottom: px(BOTTOM_DOCK_H + 54.0),
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        ToastRoot,
        Pickable::IGNORE,
        Name::new("Toast"),
    ))
    .with_children(|toast| {
        toast
            .spawn((
                Node {
                    padding: UiRect::axes(px(13.0), px(9.0)),
                    border: px(1.0).all(),
                    border_radius: BorderRadius::all(px(8.0)),
                    ..default()
                },
                BackgroundColor(CARD_BG),
                BorderColor::all(LINE),
                ToastPanel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Saved locally"),
                    text_font(fonts, 11.0, false),
                    TextColor(INK),
                    ToastText,
                ));
            });
    });
}

fn spawn_modal(root: &mut ChildSpawnerCommands, fonts: &UiAssets) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ZIndex(100),
        Pickable::IGNORE,
        ModalRoot,
        Name::new("Modal layer"),
    ))
    .with_children(|layer| {
        layer.spawn((
            Node {
                width: px(560.0),
                max_height: percent(86.0),
                padding: UiRect::all(px(24.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(14.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.043, 0.075, 0.133)),
            BorderColor::all(Color::srgba(0.10, 0.16, 0.25, 1.0)),
            ScrollPosition::default(),
            ModalContent,
            Name::new("Modal card"),
        ));
        let _ = fonts;
    });
}

fn spawn_mode_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    mode: ViewMode,
) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(AppAction::SetMode(mode)),
            ModeButton(mode),
            Node {
                height: px(32.0),
                padding: UiRect::horizontal(px(13.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.08, 0.13, 0.9)),
            BorderColor::all(LINE),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 11.0, true),
            TextColor(MUTED),
        ));
}

fn spawn_icon_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    action: AppAction,
) -> Entity {
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(action),
            ButtonStyle {
                normal: Color::srgba(0.04, 0.08, 0.13, 0.8),
                hover: Color::srgba(0.08, 0.14, 0.22, 1.0),
                pressed: Color::srgba(0.03, 0.06, 0.10, 1.0),
                border: LINE,
            },
            Node {
                width: px(26.0),
                height: px(26.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.08, 0.13, 0.8)),
            BorderColor::all(LINE),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 14.0, true),
            TextColor(MUTED),
        ))
        .id()
}

fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    kind: ButtonKind,
    action: AppAction,
    width: Option<f32>,
    height: f32,
) -> Entity {
    let style = kind.style();
    let text_color = if matches!(kind, ButtonKind::Primary) {
        Color::srgb(0.02, 0.08, 0.12)
    } else {
        INK
    };
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(action),
            ButtonStyle {
                normal: style.normal,
                hover: style.hover,
                pressed: style.pressed,
                border: style.border,
            },
            Node {
                width: width.map(px).unwrap_or(Val::Auto),
                height: px(height),
                min_width: px(0.0),
                padding: UiRect::horizontal(px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(style.normal),
            BorderColor::all(style.border),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 11.5, true),
            TextColor(text_color),
        ))
        .id()
}

fn spawn_text_input<M: Bundle>(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    initial: &str,
    multiline: bool,
    height: f32,
    marker: M,
) -> Entity {
    parent
        .spawn((
            Node {
                width: percent(100.0),
                min_height: px(height),
                padding: UiRect::axes(px(10.0), px(8.0)),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            TextInput,
            EditableText {
                allow_newlines: multiline,
                visible_lines: if multiline { Some(6.0) } else { Some(1.0) },
                ..EditableText::new(initial)
            },
            TextLayout {
                linebreak: if multiline {
                    LineBreak::WordOrCharacter
                } else {
                    LineBreak::NoWrap
                },
                ..default()
            },
            TextFont {
                font: fonts.regular.clone().into(),
                font_size: FontSize::Px(if multiline { 13.0 } else { 12.5 }),
                ..default()
            },
            TextColor(INK),
            TextCursorStyle::default(),
            TabIndex(0),
            BackgroundColor(INPUT_BG),
            BorderColor::all(INPUT_LINE),
            marker,
        ))
        .id()
}

fn text_font(fonts: &UiAssets, size: f32, bold: bool) -> TextFont {
    TextFont {
        font: if bold {
            fonts.bold.clone()
        } else {
            fonts.regular.clone()
        }
        .into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

fn divider() -> (Node, BackgroundColor) {
    (
        Node {
            width: percent(100.0),
            height: px(1.0),
            margin: UiRect::vertical(px(2.0)),
            ..default()
        },
        BackgroundColor(LINE),
    )
}

pub fn rebuild_sphere_list(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    fonts: Res<UiAssets>,
    roots: Query<Entity, With<SphereListContent>>,
) {
    if !state.ui.list {
        return;
    }
    state.ui.list = false;
    for root in &roots {
        commands.entity(root).despawn_related::<Children>();
        commands.entity(root).with_children(|list| {
            let spheres = state.filtered_spheres();
            if spheres.is_empty() {
                list.spawn((
                    Text::new("No spheres match that search."),
                    text_font(&fonts, 11.0, false),
                    TextColor(MUTED),
                ));
                return;
            }
            for sphere in spheres {
                let selected = sphere.id == state.current_id;
                let bg = if selected {
                    Color::srgba(0.10, 0.28, 0.34, 0.42)
                } else {
                    Color::NONE
                };
                let border = if selected {
                    Color::srgba(0.35, 0.85, 0.91, 0.35)
                } else {
                    Color::NONE
                };
                let entity = list
                    .spawn((
                        Button,
                        Hovered::default(),
                        ActionButton(AppAction::SelectSphere(sphere.id.clone())),
                        ButtonStyle {
                            normal: bg,
                            hover: Color::srgba(0.07, 0.12, 0.19, 0.8),
                            pressed: Color::srgba(0.04, 0.08, 0.13, 1.0),
                            border,
                        },
                        Node {
                            width: percent(100.0),
                            min_height: px(50.0),
                            padding: UiRect::axes(px(10.0), px(7.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            row_gap: px(2.0),
                            border: px(1.0).all(),
                            border_radius: BorderRadius::all(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor::all(border),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(&sphere.title),
                            text_font(&fonts, 12.5, true),
                            TextColor(if selected {
                                INK
                            } else {
                                Color::srgb(0.80, 0.85, 0.92)
                            }),
                        ));
                        row.spawn((
                            Text::new(format!(
                                "{} thoughts · {} groups",
                                sphere.nodes.len(),
                                sphere.groups.len()
                            )),
                            text_font(&fonts, 10.0, false),
                            TextColor(FAINT),
                        ));
                    })
                    .id();
                let _ = entity;
            }
        });
    }
}

pub fn rebuild_inspector(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    fonts: Res<UiAssets>,
    roots: Query<Entity, With<InspectorRoot>>,
) {
    if !state.ui.inspector {
        return;
    }
    state.ui.inspector = false;
    let selected = state.selected_node().cloned();
    let sphere = state.current().cloned();
    for root in &roots {
        commands.entity(root).despawn_related::<Children>();
        commands.entity(root).with_children(|panel| {
            let Some(sphere) = sphere.as_ref() else {
                return;
            };
            let Some(node) = selected.as_ref() else {
                panel.spawn((
                    Text::new("Nothing selected"),
                    text_font(&fonts, 22.0, true),
                    TextColor(INK),
                ));
                panel.spawn((
                    Text::new(
                        "Pick a thought on the sphere or the board to edit it, move it into a group, or connect it to another thought.",
                    ),
                    text_font(&fonts, 12.0, false),
                    TextColor(MUTED),
                    TextLayout {
                        linebreak: LineBreak::WordOrCharacter,
                        ..default()
                    },
                ));

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|stats| {
                        stat_card(
                            stats,
                            &fonts,
                            sphere.nodes.len(),
                            "thoughts",
                            StatKind::Thoughts,
                        );
                        stat_card(
                            stats,
                            &fonts,
                            sphere.links.len(),
                            "links",
                            StatKind::Links,
                        );
                        stat_card(
                            stats,
                            &fonts,
                            sphere.groups.len(),
                            "groups",
                            StatKind::Groups,
                        );
                    });

                section_label(panel, &fonts, "GROUPS");
                if sphere.groups.is_empty() {
                    panel.spawn((
                        Text::new("No groups yet. Create one to give this map some shape."),
                        text_font(&fonts, 11.0, false),
                        TextColor(MUTED),
                        TextLayout {
                            linebreak: LineBreak::WordOrCharacter,
                            ..default()
                        },
                    ));
                } else {
                    for group in &sphere.groups {
                        let count = sphere
                            .nodes
                            .iter()
                            .filter(|node| node.group_id == group.id)
                            .count();
                        group_row(panel, &fonts, group, count);
                    }
                }
                spawn_button(
                    panel,
                    &fonts,
                    "+ New group",
                    ButtonKind::Chip,
                    AppAction::NewGroup,
                    None,
                    30.0,
                );

                section_label(panel, &fonts, "MOVE AROUND");
                panel.spawn((
                    Text::new(
                        "Drag the globe to turn it. Drag a thought to move it across the surface. Scroll to zoom.",
                    ),
                    text_font(&fonts, 11.0, false),
                    TextColor(MUTED),
                    TextLayout {
                        linebreak: LineBreak::WordOrCharacter,
                        ..default()
                    },
                ));
                return;
            };

            if state.selected_ids.len() > 1 {
                panel.spawn((
                    Text::new("SELECTION"),
                    text_font(&fonts, 10.0, true),
                    TextColor(ACCENT),
                ));
                panel.spawn((
                    Text::new(format!("{} thoughts selected", state.selected_ids.len())),
                    text_font(&fonts, 22.0, true),
                    TextColor(INK),
                ));
                panel.spawn((
                    Text::new("Use Add to group or Remove from group in the top bar."),
                    text_font(&fonts, 12.0, false),
                    TextColor(MUTED),
                    TextLayout {
                        linebreak: LineBreak::WordOrCharacter,
                        ..default()
                    },
                ));
                section_label(panel, &fonts, "THOUGHTS");
                for selected_id in &state.selected_ids {
                    if let Some(selected_thought) = sphere.node(selected_id) {
                        panel.spawn((
                            Text::new(format!("• {}", selected_thought.title)),
                            text_font(&fonts, 10.5, false),
                            TextColor(INK),
                        ));
                    }
                }
                return;
            }
            panel.spawn((
                Text::new("THOUGHT"),
                text_font(&fonts, 10.0, true),
                TextColor(ACCENT),
            ));
            spawn_text_input(
                panel,
                &fonts,
                &node.title,
                false,
                38.0,
                (TitleInput, Name::new("Thought title")),
            );
            section_label(panel, &fonts, "NOTES");
            spawn_text_input(
                panel,
                &fonts,
                &node.body,
                true,
                174.0,
                (BodyInput, Name::new("Thought notes")),
            );

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_child((
                    Text::new("GROUP"),
                    text_font(&fonts, 10.0, true),
                    TextColor(MUTED),
                ))
                .with_child((
                    Text::new(sphere.group_name(&node.group_id)),
                    text_font(&fonts, 10.0, false),
                    TextColor(FAINT),
                ));

            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(5.0),
                    row_gap: px(5.0),
                    ..default()
                })
                .with_children(|chips| {
                    for group in &sphere.groups {
                        let selected_group = group.id == node.group_id;
                        spawn_group_chip(
                            chips,
                            &fonts,
                            &group.name,
                            &group.color,
                            selected_group,
                            AppAction::MoveToGroup(group.id.clone()),
                        );
                    }
                    spawn_group_chip(
                        chips,
                        &fonts,
                        "Unfiled",
                        "#8b95b5",
                        node.group_id.is_empty(),
                        AppAction::MoveToGroup(String::new()),
                    );
                });

            let linked = sphere.links_for(&node.id);
            if !linked.is_empty() {
                section_label(panel, &fonts, "LINKS");
                panel
                    .spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!(
                                "{} link{}",
                                linked.len(),
                                if linked.len() == 1 { "" } else { "s" }
                            )),
                            text_font(&fonts, 10.5, false),
                            TextColor(MUTED),
                        ));
                        row.spawn(Node {
                            flex_grow: 1.0,
                            ..default()
                        });
                        spawn_button(
                            row,
                            &fonts,
                            "Erase link",
                            ButtonKind::Chip,
                            AppAction::EraseLinks,
                            Some(100.0),
                            28.0,
                        );
                    });
            }
            section_label(panel, &fonts, "CONNECT TO");
            let mut candidates = 0;
            for other in &sphere.nodes {
                if other.id == node.id || linked.contains(&other.id.as_str()) {
                    continue;
                }
                if candidates >= 6 {
                    break;
                }
                candidates += 1;
                panel
                    .spawn((
                        Button,
                        Hovered::default(),
                        ActionButton(AppAction::ConnectTo(other.id.clone())),
                        ButtonStyle {
                            normal: Color::srgba(0.03, 0.06, 0.10, 0.9),
                            hover: Color::srgba(0.06, 0.11, 0.17, 1.0),
                            pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                            border: INPUT_LINE,
                        },
                        Node {
                            width: percent(100.0),
                            min_height: px(30.0),
                            padding: UiRect::axes(px(9.0), px(5.0)),
                            align_items: AlignItems::Center,
                            border: px(1.0).all(),
                            border_radius: BorderRadius::all(px(7.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.03, 0.06, 0.10, 0.9)),
                        BorderColor::all(INPUT_LINE),
                    ))
                    .with_child((
                        Text::new(format!("+ {}", other.title)),
                        text_font(&fonts, 10.5, false),
                        TextColor(MUTED),
                    ));
            }
            if candidates == 0 {
                panel.spawn((
                    Text::new("Everything here is already connected."),
                    text_font(&fonts, 11.0, false),
                    TextColor(FAINT),
                ));
            }

            panel.spawn(divider());
            panel.spawn((
                Text::new(format!(
                    "{} connection{} · {}",
                    linked.len(),
                    if linked.len() == 1 { "" } else { "s" },
                    sphere.group_name(&node.group_id)
                )),
                text_font(&fonts, 10.0, false),
                TextColor(FAINT),
            ));
        });
    }
}

fn stat_card(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    value: usize,
    label: &str,
    kind: StatKind,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::axes(px(9.0), px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(1.0),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(CARD_BG),
            BorderColor::all(LINE),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(value.to_string()),
                text_font(fonts, 18.0, true),
                TextColor(INK),
                StatCount(kind),
            ));
            card.spawn((
                Text::new(label),
                text_font(fonts, 9.5, false),
                TextColor(FAINT),
            ));
        });
}

fn group_row(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    group: &crate::model::Group,
    count: usize,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(7.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(7.0),
                    height: px(7.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(color_from_hex(&group.color)),
            ));
            row.spawn((
                Text::new(&group.name),
                text_font(fonts, 11.0, false),
                TextColor(MUTED),
            ));
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            row.spawn((
                Text::new(count.to_string()),
                text_font(fonts, 10.0, false),
                TextColor(FAINT),
            ));
        });
}

fn spawn_group_chip(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    name: &str,
    color: &str,
    selected: bool,
    action: AppAction,
) {
    let border = if selected {
        alpha(color_from_hex(color), 0.8)
    } else {
        INPUT_LINE
    };
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(action),
            ButtonStyle {
                normal: if selected {
                    Color::srgba(0.10, 0.24, 0.30, 0.42)
                } else {
                    Color::srgba(0.03, 0.06, 0.10, 0.9)
                },
                hover: Color::srgba(0.06, 0.12, 0.18, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border,
            },
            Node {
                height: px(29.0),
                padding: UiRect::axes(px(8.0), px(4.0)),
                align_items: AlignItems::Center,
                column_gap: px(6.0),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(if selected {
                Color::srgba(0.10, 0.24, 0.30, 0.42)
            } else {
                Color::srgba(0.03, 0.06, 0.10, 0.9)
            }),
            BorderColor::all(border),
        ))
        .with_children(|chip| {
            chip.spawn((
                Node {
                    width: px(6.0),
                    height: px(6.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(color_from_hex(color)),
            ));
            chip.spawn((
                Text::new(name),
                text_font(fonts, 10.5, false),
                TextColor(if selected { INK } else { MUTED }),
            ));
        });
}

fn section_label(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, label: &str) {
    parent.spawn((
        Text::new(label),
        text_font(fonts, 10.0, true),
        TextColor(MUTED),
    ));
}

pub fn rebuild_board(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    fonts: Res<UiAssets>,
    roots: Query<Entity, With<BoardColumns>>,
    mut board_roots: Query<&mut Node, With<BoardRoot>>,
) {
    for mut node in &mut board_roots {
        node.display = if state.mode == ViewMode::Board {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !state.ui.board {
        return;
    }
    state.ui.board = false;
    let sphere = state.current().cloned();
    let selected_ids = state.selected_ids.clone();
    let unlink_targets = state.unlink_targets.clone();
    for root in &roots {
        commands.entity(root).despawn_related::<Children>();
        commands.entity(root).with_children(|board| {
            let Some(sphere) = sphere.as_ref() else {
                return;
            };
            let mut groups: Vec<Option<&crate::model::Group>> =
                sphere.groups.iter().map(Some).collect();
            let has_unfiled = sphere.nodes.iter().any(|node| node.group_id.is_empty());
            if has_unfiled || groups.is_empty() {
                groups.push(None);
            }
            if sphere.nodes.is_empty() {
                board.spawn((
                    Text::new("This sphere is empty. Add a thought to start the board."),
                    text_font(&fonts, 13.0, false),
                    TextColor(MUTED),
                ));
                return;
            }
            for group in groups {
                let (id, name, color, nodes) = match group {
                    Some(group) => (
                        group.id.clone(),
                        group.name.clone(),
                        group.color.clone(),
                        sphere
                            .nodes
                            .iter()
                            .filter(|node| node.group_id == group.id)
                            .collect::<Vec<_>>(),
                    ),
                    None => (
                        String::new(),
                        "Unfiled".to_owned(),
                        "#8b95b5".to_owned(),
                        sphere
                            .nodes
                            .iter()
                            .filter(|node| node.group_id.is_empty())
                            .collect::<Vec<_>>(),
                    ),
                };
                board
                    .spawn((
                        Node {
                            width: px(188.0),
                            min_width: px(188.0),
                            max_height: percent(100.0),
                            padding: UiRect::all(px(11.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8.0),
                            border: px(1.0).all(),
                            border_radius: BorderRadius::all(px(10.0)),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.025, 0.047, 0.08, 0.96)),
                        BorderColor::all(alpha(color_from_hex(&color), 0.55)),
                        BoardColumn(id.clone()),
                        Pickable::default(),
                    ))
                    .with_children(|column| {
                        column
                            .spawn((
                                Node {
                                    width: percent(100.0),
                                    min_height: px(24.0),
                                    align_items: AlignItems::Center,
                                    column_gap: px(6.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.01)),
                                ZIndex(2),
                                Button,
                                BoardColumnHandle(id.clone()),
                                Hovered::default(),
                                Pickable::default(),
                                UiTransform::default(),
                                Outline {
                                    width: px(2.0),
                                    offset: px(1.0),
                                    color: Color::NONE,
                                },
                            ))
                            .observe(board_drag_start)
                            .with_children(|head| {
                                head.spawn((
                                    Node {
                                        width: px(7.0),
                                        height: px(7.0),
                                        border_radius: BorderRadius::MAX,
                                        ..default()
                                    },
                                    BackgroundColor(color_from_hex(&color)),
                                    Pickable::IGNORE,
                                ));
                                head.spawn((
                                    Text::new(&name),
                                    text_font(&fonts, 11.0, true),
                                    TextColor(INK),
                                ));
                                head.spawn((
                                    Node {
                                        flex_grow: 1.0,
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                                head.spawn((
                                    Text::new(nodes.len().to_string()),
                                    text_font(&fonts, 10.0, false),
                                    TextColor(FAINT),
                                ));
                            });
                        if nodes.is_empty() {
                            column.spawn((
                                Text::new("No thoughts yet"),
                                text_font(&fonts, 10.0, false),
                                TextColor(FAINT),
                            ));
                        }
                        for node in nodes {
                            let is_selected = selected_ids.iter().any(|id| id == &node.id);
                            let is_unlink_target = unlink_targets.iter().any(|id| id == &node.id);
                            let border = if is_unlink_target {
                                AMBER
                            } else if is_selected {
                                Color::srgba(0.35, 0.85, 0.91, 0.7)
                            } else {
                                Color::srgba(0.10, 0.16, 0.25, 1.0)
                            };
                            column
                                .spawn((
                                    Button,
                                    Hovered::default(),
                                    ActionButton(AppAction::SelectNode(node.id.clone())),
                                    BoardCard(node.id.clone()),
                                    UiTransform::default(),
                                    Outline {
                                        width: px(2.0),
                                        offset: px(1.0),
                                        color: Color::NONE,
                                    },
                                    ButtonStyle {
                                        normal: if is_unlink_target {
                                            Color::srgba(0.22, 0.12, 0.04, 0.8)
                                        } else if is_selected {
                                            Color::srgba(0.09, 0.2, 0.25, 0.5)
                                        } else {
                                            CARD_BG
                                        },
                                        hover: Color::srgba(0.07, 0.13, 0.2, 1.0),
                                        pressed: Color::srgba(0.04, 0.07, 0.12, 1.0),
                                        border,
                                    },
                                    Node {
                                        width: percent(100.0),
                                        padding: UiRect::axes(px(9.0), px(8.0)),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(3.0),
                                        border: px(1.0).all(),
                                        border_radius: BorderRadius::all(px(8.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if is_unlink_target {
                                        Color::srgba(0.22, 0.12, 0.04, 0.8)
                                    } else if is_selected {
                                        Color::srgba(0.09, 0.2, 0.25, 0.5)
                                    } else {
                                        CARD_BG
                                    }),
                                    BorderColor::all(border),
                                ))
                                .observe(board_drag_start)
                                .with_children(|card| {
                                    card.spawn((
                                        Text::new(&node.title),
                                        text_font(&fonts, 11.5, true),
                                        TextColor(INK),
                                        Pickable::IGNORE,
                                    ));
                                    if !node.body.trim().is_empty() {
                                        let snippet: String = node
                                            .body
                                            .chars()
                                            .take(72)
                                            .collect::<String>()
                                            .replace('\n', " ");
                                        card.spawn((
                                            Text::new(snippet),
                                            text_font(&fonts, 9.5, false),
                                            TextColor(MUTED),
                                            TextLayout {
                                                linebreak: LineBreak::WordOrCharacter,
                                                ..default()
                                            },
                                            Pickable::IGNORE,
                                        ));
                                    }
                                });
                        }
                    });
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn rebuild_globe_panel(
    mut commands: Commands,
    state: Res<AppState>,
    fonts: Res<UiAssets>,
    mut root: Query<(Entity, &mut Node), With<GlobePanelRoot>>,
    mut last_revision: Local<Option<u64>>,
    mut last_sphere: Local<Option<String>>,
) {
    let Ok((root, mut root_node)) = root.single_mut() else {
        return;
    };
    let open = state.modal == Some(ModalKind::Globe);
    root_node.display = if open { Display::Flex } else { Display::None };
    if !open {
        return;
    }
    if *last_revision == Some(state.modal_rev)
        && last_sphere.as_deref() == Some(state.current_id.as_str())
    {
        return;
    }
    *last_revision = Some(state.modal_rev);
    *last_sphere = Some(state.current_id.clone());
    let Some(sphere) = state.current().cloned() else {
        return;
    };
    commands.entity(root).despawn_related::<Children>();
    commands.entity(root).with_children(|panel| {
        panel
            .spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("GLOBE"),
                    text_font(&fonts, 10.0, true),
                    TextColor(ACCENT),
                ));
                spawn_button(
                    row,
                    &fonts,
                    "Done",
                    ButtonKind::Ghost,
                    AppAction::CloseModal,
                    Some(64.0),
                    32.0,
                );
            });
        modal_title(panel, &fonts, "Surface controls");
        modal_hint(
            panel,
            &fonts,
            "Changes update the globe in real time. The overview panel is covered while this panel is open.",
        );
        section_label(panel, &fonts, "SURFACE");
        modal_hint(
            panel,
            &fonts,
            &format!(
                "Current surface: {} · {}",
                skin_label(&sphere.skin),
                background_label(&sphere.background)
            ),
        );
        modal_slider_row(
            panel,
            &fonts,
            "Sphere radius",
            1.8,
            5.0,
            sphere.radius,
            GlobeRadiusSlider,
            GlobeRadiusValue,
        );
        modal_slider_row(
            panel,
            &fonts,
            "Transparency",
            0.15,
            1.0,
            sphere.transparency,
            GlobeTransparencySlider,
            GlobeTransparencyValue,
        );
        section_label(panel, &fonts, "PROJECTION LINES");
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(5.0),
                row_gap: px(5.0),
                ..default()
            })
            .with_children(|row| {
                spawn_choice_button(
                    row,
                    &fonts,
                    "Lat / Long",
                    sphere.grid == "latlong",
                    AppAction::SetGrid("latlong".to_owned()),
                );
                spawn_choice_button(
                    row,
                    &fonts,
                    "Geodesic",
                    sphere.grid == "geodesic",
                    AppAction::SetGrid("geodesic".to_owned()),
                );
                spawn_choice_button(
                    row,
                    &fonts,
                    "Off",
                    sphere.grid == "off",
                    AppAction::SetGrid("off".to_owned()),
                );
            });
        section_label(panel, &fonts, "SURFACE SKIN");
        modal_skin_grid(panel, &fonts, &sphere.skin);
        section_label(panel, &fonts, "BACKDROP");
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6.0),
                ..default()
            })
            .with_children(|row| {
                for (id, label) in BACKGROUNDS {
                    spawn_choice_button(
                        row,
                        &fonts,
                        label,
                        sphere.background == *id,
                        AppAction::SetBackground((*id).to_owned()),
                    );
                }
            });
        section_label(panel, &fonts, "GROUP EDGE COLOURS");
        if sphere.groups.is_empty() {
            modal_hint(panel, &fonts, "Create a group to give this map an edge colour.");
        } else {
            for group in &sphere.groups {
                modal_group_palette(panel, &fonts, group);
            }
        }
        spawn_button(
            panel,
            &fonts,
            "Done",
            ButtonKind::Primary,
            AppAction::CloseModal,
            None,
            38.0,
        );
    });
}

pub fn rebuild_modal(
    mut commands: Commands,
    state: Res<AppState>,
    fonts: Res<UiAssets>,
    mut modal_root: Query<(Entity, &mut Node), With<ModalRoot>>,
    content: Query<Entity, With<ModalContent>>,
    mut last_revision: Local<Option<u64>>,
) {
    if *last_revision == Some(state.modal_rev) {
        return;
    }
    *last_revision = Some(state.modal_rev);
    for (entity, mut node) in &mut modal_root {
        let open = state.modal.is_some() && state.modal != Some(ModalKind::Globe);
        node.display = if open { Display::Flex } else { Display::None };
        commands.entity(entity).insert(if open {
            Pickable::default()
        } else {
            Pickable::IGNORE
        });
    }
    let Ok(content) = content.single() else {
        return;
    };
    commands.entity(content).despawn_related::<Children>();
    let Some(kind) = state.modal else {
        return;
    };
    if kind == ModalKind::Globe {
        return;
    }
    commands.entity(content).with_children(|card| {
        let kicker = match kind {
            ModalKind::More => "SPHERE",
            ModalKind::Shortcuts => "REFERENCE",
            ModalKind::About => "ABOUT",
            ModalKind::Globe => "GLOBE",
            _ => "SPHERE",
        };
        card.spawn((
            Text::new(kicker),
            text_font(&fonts, 10.0, true),
            TextColor(ACCENT),
        ));

        match kind {
            ModalKind::NewSphere => {
                modal_title(card, &fonts, "Create a sphere");
                modal_hint(
                    card,
                    &fonts,
                    "Give this sphere a question or a working title. You can rename it later.",
                );
                spawn_text_input(
                    card,
                    &fonts,
                    &state.modal_input,
                    false,
                    38.0,
                    (ModalInput, AutoFocus, Name::new("New sphere name")),
                );
                modal_buttons(card, &fonts, "Cancel", "Create sphere");
            }
            ModalKind::RenameSphere => {
                modal_title(card, &fonts, "Rename this sphere");
                modal_hint(card, &fonts, "The new name appears in your library immediately.");
                spawn_text_input(
                    card,
                    &fonts,
                    &state.modal_input,
                    false,
                    38.0,
                    (ModalInput, AutoFocus, Name::new("Sphere name")),
                );
                modal_buttons(card, &fonts, "Cancel", "Save name");
            }
            ModalKind::DeleteSphere => {
                modal_title(card, &fonts, "Delete this sphere?");
                modal_hint(
                    card,
                    &fonts,
                    "This removes the sphere and its thoughts from this device. Export it first if you want a copy.",
                );
                modal_buttons(card, &fonts, "Cancel", "Delete sphere");
            }
            ModalKind::NewGroup => {
                modal_title(card, &fonts, "Create a group");
                modal_hint(
                    card,
                    &fonts,
                    "Groups give related thoughts a colour and a column on the board.",
                );
                spawn_text_input(
                    card,
                    &fonts,
                    &state.modal_input,
                    false,
                    38.0,
                    (ModalInput, AutoFocus, Name::new("Group name")),
                );
                modal_buttons(card, &fonts, "Cancel", "Create group");
            }
            ModalKind::Import => {
                modal_title(card, &fonts, "Bring in a sphere");
                modal_hint(
                    card,
                    &fonts,
                    "Paste exported sphere JSON below. On the web, the Import button opens a file picker as well.",
                );
                spawn_text_input(
                    card,
                    &fonts,
                    &state.modal_input,
                    true,
                    220.0,
                    (ModalInput, AutoFocus, Name::new("Imported JSON")),
                );
                modal_buttons(card, &fonts, "Cancel", "Import sphere");
            }
            ModalKind::NewThought => {
                modal_title(card, &fonts, "New thought");
                modal_hint(
                    card,
                    &fonts,
                    "Enter a title. The group defaults to None; open the dropdown to choose another.",
                );
                spawn_text_input(
                    card,
                    &fonts,
                    &state.modal_input,
                    false,
                    38.0,
                    (ModalInput, AutoFocus, Name::new("Thought title")),
                );
                if let Some(sphere) = state.current() {
                    modal_group_dropdown(
                        card,
                        &fonts,
                        sphere,
                        &state.modal_group,
                        state.group_dropdown_open,
                        true,
                    );
                }
                modal_buttons(card, &fonts, "Cancel", "Create thought");
            }
            ModalKind::AddToGroup => {
                modal_title(card, &fonts, "Add selected thoughts to a group");
                modal_hint(
                    card,
                    &fonts,
                    "Choose the destination group. The layout will re-pack automatically.",
                );
                if let Some(sphere) = state.current() {
                    modal_group_dropdown(
                        card,
                        &fonts,
                        sphere,
                        &state.modal_group,
                        state.group_dropdown_open,
                        false,
                    );
                }
                modal_buttons(card, &fonts, "Cancel", "Add to group");
            }
            ModalKind::Globe => {
                modal_title(card, &fonts, "Globe settings");
                modal_hint(
                    card,
                    &fonts,
                    "Change the surface, size, transparency, and star background. The force field keeps thoughts and groups apart as the map changes.",
                );
                let Some(sphere) = state.current() else {
                    return;
                };
                section_label(card, &fonts, "SURFACE");
                modal_hint(
                    card,
                    &fonts,
                    &format!(
                        "Current surface: {} · {}",
                        skin_label(&sphere.skin),
                        background_label(&sphere.background)
                    ),
                );
                modal_slider_row(
                    card,
                    &fonts,
                    "Sphere radius",
                    1.8,
                    5.0,
                    sphere.radius,
                    GlobeRadiusSlider,
                    GlobeRadiusValue,
                );
                modal_slider_row(
                    card,
                    &fonts,
                    "Transparency",
                    0.15,
                    1.0,
                    sphere.transparency,
                    GlobeTransparencySlider,
                    GlobeTransparencyValue,
                );
                modal_skin_grid(card, &fonts, &sphere.skin);
                section_label(card, &fonts, "BACKDROP");
                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    for (id, label) in BACKGROUNDS {
                        let selected = sphere.background == *id;
                        spawn_choice_button(
                            row,
                            &fonts,
                            label,
                            selected,
                            AppAction::SetBackground((*id).to_owned()),
                        );
                    }
                });
                section_label(card, &fonts, "GROUP EDGE COLOURS");
                if sphere.groups.is_empty() {
                    modal_hint(card, &fonts, "Create a group to give this map an edge colour.");
                } else {
                    for group in &sphere.groups {
                        modal_group_palette(card, &fonts, group);
                    }
                }
                modal_buttons_single(card, &fonts, "Close");
            }
            ModalKind::More => {
                modal_title(card, &fonts, "Sphere actions");
                modal_menu_button(card, &fonts, "Rename sphere", AppAction::RenameSphere);
                modal_menu_button(card, &fonts, "Duplicate sphere", AppAction::DuplicateSphere);
                modal_menu_button(card, &fonts, "Export as JSON", AppAction::ExportSphere);
                modal_menu_button(card, &fonts, "Delete sphere", AppAction::DeleteSphere);
                modal_menu_button(
                    card,
                    &fonts,
                    "Keyboard shortcuts",
                    AppAction::OpenModal(ModalKind::Shortcuts),
                );
                modal_menu_button(card, &fonts, "About Mindsphere", AppAction::OpenModal(ModalKind::About));
                modal_menu_button(card, &fonts, "Close", AppAction::CloseModal);
            }
            ModalKind::Shortcuts => {
                modal_title(card, &fonts, "Keyboard shortcuts");
                shortcut_line(card, &fonts, "1 / 2", "Switch sphere and board views");
                shortcut_line(card, &fonts, "N", "New thought");
                shortcut_line(card, &fonts, "Ctrl+F", "Search your spheres");
                shortcut_line(card, &fonts, "Ctrl+S", "Save now");
                shortcut_line(card, &fonts, "Ctrl+E", "Export this sphere");
                shortcut_line(card, &fonts, "Delete", "Delete the selected thought");
                shortcut_line(card, &fonts, "Esc", "Clear selection or close a dialog");
                modal_buttons_single(card, &fonts, "Close");
            }
            ModalKind::About => {
                modal_title(card, &fonts, "A local-first sphere mind mapper");
                modal_hint(
                    card,
                    &fonts,
                    "Mindsphere is built with Rust and Bevy. Every thought gets a place on a globe, and every sphere is plain JSON on your device. No accounts, no cloud, no telemetry.",
                );
                modal_hint(
                    card,
                    &fonts,
                    "The browser build stores spheres in localStorage. Export anything you want to keep.",
                );
                modal_menu_button(
                    card,
                    &fonts,
                    "Gallifrey Station ↗",
                    AppAction::OpenLink("https://orbitmechanic.github.io/".to_owned()),
                );
                modal_menu_button(
                    card,
                    &fonts,
                    "Source on GitHub ↗",
                    AppAction::OpenLink("https://github.com/orbitmechanic/mindsphere".to_owned()),
                );
                modal_buttons_single(card, &fonts, "Close");
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn modal_slider_row(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    min: f32,
    max: f32,
    value: f32,
    slider_marker: impl Component,
    value_marker: impl Component,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|head| {
                head.spawn((
                    Text::new(label),
                    text_font(fonts, 11.0, false),
                    TextColor(MUTED),
                ));
                head.spawn((
                    Text::new(if max <= 1.0 {
                        format!("{:.0}%", value * 100.0)
                    } else {
                        format!("{value:.1}")
                    }),
                    text_font(fonts, 11.0, true),
                    TextColor(ACCENT),
                    value_marker,
                ));
            });
            row.spawn((
                Node {
                    width: percent(100.0),
                    height: px(22.0),
                    ..default()
                },
                Hovered::default(),
                slider_marker,
                Slider {
                    track_click: TrackClick::Snap,
                    ..default()
                },
                SliderValue(value),
                SliderRange::new(min, max),
                TabIndex(0),
            ))
            .with_children(|slider| {
                slider.spawn((
                    Node {
                        width: percent(100.0),
                        height: px(5.0),
                        margin: UiRect::top(px(8.0)),
                        border_radius: BorderRadius::all(px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.14, 0.22, 1.0)),
                ));
                slider
                    .spawn(Node {
                        display: Display::Flex,
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        right: px(10.0),
                        top: px(0.0),
                        bottom: px(0.0),
                        ..default()
                    })
                    .with_child((
                        Node {
                            display: Display::Flex,
                            width: px(10.0),
                            height: px(10.0),
                            position_type: PositionType::Absolute,
                            left: percent(if max > min {
                                (value - min) / (max - min) * 100.0
                            } else {
                                0.0
                            }),
                            top: px(6.0),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                        GlobeSliderThumb,
                        SliderThumb,
                    ));
            });
        });
}

fn modal_skin_grid(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, selected: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(5.0),
            row_gap: px(5.0),
            ..default()
        })
        .with_children(|grid| {
            for skin in SKINS {
                let is_selected = skin.id == selected;
                let border = if is_selected { ACCENT } else { INPUT_LINE };
                grid.spawn((
                    Button,
                    Hovered::default(),
                    ActionButton(AppAction::SetSkin(skin.id.to_owned())),
                    ButtonStyle {
                        normal: if is_selected {
                            Color::srgba(0.10, 0.24, 0.30, 0.55)
                        } else {
                            Color::srgba(0.03, 0.06, 0.10, 0.95)
                        },
                        hover: Color::srgba(0.06, 0.12, 0.18, 1.0),
                        pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                        border,
                    },
                    Node {
                        width: px(160.0),
                        height: px(30.0),
                        padding: UiRect::horizontal(px(7.0)),
                        align_items: AlignItems::Center,
                        column_gap: px(6.0),
                        border: px(1.0).all(),
                        border_radius: BorderRadius::all(px(6.0)),
                        ..default()
                    },
                    BackgroundColor(if is_selected {
                        Color::srgba(0.10, 0.24, 0.30, 0.55)
                    } else {
                        Color::srgba(0.03, 0.06, 0.10, 0.95)
                    }),
                    BorderColor::all(border),
                ))
                .with_children(|chip| {
                    chip.spawn((
                        Node {
                            width: px(8.0),
                            height: px(8.0),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(skin_swatch(skin.id)),
                    ));
                    chip.spawn((
                        Text::new(skin.label),
                        text_font(fonts, 9.5, false),
                        TextColor(if is_selected { INK } else { MUTED }),
                    ));
                });
            }
        });
}

fn spawn_choice_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    selected: bool,
    action: AppAction,
) {
    let border = if selected { ACCENT } else { INPUT_LINE };
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(action),
            ButtonStyle {
                normal: if selected {
                    Color::srgba(0.10, 0.24, 0.30, 0.55)
                } else {
                    Color::srgba(0.03, 0.06, 0.10, 0.95)
                },
                hover: Color::srgba(0.06, 0.12, 0.18, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border,
            },
            Node {
                height: px(32.0),
                padding: UiRect::horizontal(px(10.0)),
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(if selected {
                Color::srgba(0.10, 0.24, 0.30, 0.55)
            } else {
                Color::srgba(0.03, 0.06, 0.10, 0.95)
            }),
            BorderColor::all(border),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 10.5, false),
            TextColor(if selected { INK } else { MUTED }),
        ));
}

fn modal_group_palette(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    group: &crate::model::Group,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(6.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(&group.name),
                text_font(fonts, 10.5, false),
                TextColor(MUTED),
            ));
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            for color in GROUP_COLORS {
                let selected = group.color == color;
                row.spawn((
                    Button,
                    Hovered::default(),
                    ActionButton(AppAction::SetGroupColor(group.id.clone(), color.to_owned())),
                    ButtonStyle {
                        normal: color_from_hex(color),
                        hover: color_from_hex(color),
                        pressed: color_from_hex(color),
                        border: if selected { Color::WHITE } else { Color::NONE },
                    },
                    Node {
                        width: px(17.0),
                        height: px(17.0),
                        border: px(2.0).all(),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(color_from_hex(color)),
                    BorderColor::all(if selected { Color::WHITE } else { Color::NONE }),
                ));
            }
        });
}

fn modal_group_dropdown(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    sphere: &crate::model::Sphere,
    selected: &str,
    open: bool,
    include_none: bool,
) {
    let name = if selected.is_empty() {
        "None".to_owned()
    } else {
        sphere.group_name(selected).to_owned()
    };
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(AppAction::ToggleGroupDropdown),
            ButtonStyle {
                normal: Color::srgba(0.03, 0.06, 0.10, 0.95),
                hover: Color::srgba(0.06, 0.12, 0.18, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border: INPUT_LINE,
            },
            Node {
                width: percent(100.0),
                min_height: px(36.0),
                padding: UiRect::horizontal(px(10.0)),
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.10, 0.95)),
            BorderColor::all(INPUT_LINE),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("Group: {name}")),
                text_font(fonts, 11.0, false),
                TextColor(INK),
            ));
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            row.spawn((
                Text::new(if open { "^" } else { "v" }),
                text_font(fonts, 10.0, true),
                TextColor(ACCENT),
            ));
        });
    if open {
        parent
            .spawn(Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(3.0),
                padding: UiRect::all(px(4.0)),
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            })
            .insert(BackgroundColor(CARD_BG))
            .insert(BorderColor::all(INPUT_LINE))
            .with_children(|list| {
                if include_none {
                    group_option_button(list, fonts, "None", selected.is_empty(), "");
                }
                for group in &sphere.groups {
                    group_option_button(list, fonts, &group.name, group.id == selected, &group.id);
                }
            });
    }
}

fn group_option_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    selected: bool,
    group_id: &str,
) {
    let border = if selected { ACCENT } else { Color::NONE };
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(AppAction::SetModalGroup(group_id.to_owned())),
            ButtonStyle {
                normal: if selected {
                    Color::srgba(0.10, 0.24, 0.30, 0.55)
                } else {
                    Color::NONE
                },
                hover: Color::srgba(0.06, 0.12, 0.18, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border,
            },
            Node {
                width: percent(100.0),
                min_height: px(28.0),
                padding: UiRect::horizontal(px(8.0)),
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(6.0)),
                ..default()
            },
            BackgroundColor(if selected {
                Color::srgba(0.10, 0.24, 0.30, 0.55)
            } else {
                Color::NONE
            }),
            BorderColor::all(border),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 10.5, false),
            TextColor(if selected { INK } else { MUTED }),
        ));
}

fn modal_title(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, text: &str) {
    parent.spawn((
        Text::new(text),
        text_font(fonts, 22.0, true),
        TextColor(INK),
        TextShadow::default(),
    ));
}

fn modal_hint(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, text: &str) {
    parent.spawn((
        Text::new(text),
        text_font(fonts, 12.0, false),
        TextColor(MUTED),
        TextLayout {
            linebreak: LineBreak::WordOrCharacter,
            ..default()
        },
    ));
}

fn modal_buttons(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, cancel: &str, submit: &str) {
    parent
        .spawn(Node {
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(8.0),
            margin: UiRect::top(px(6.0)),
            ..default()
        })
        .with_children(|row| {
            spawn_button(
                row,
                fonts,
                cancel,
                ButtonKind::Ghost,
                AppAction::CloseModal,
                Some(92.0),
                36.0,
            );
            spawn_button(
                row,
                fonts,
                submit,
                ButtonKind::Primary,
                AppAction::SubmitModal,
                Some(132.0),
                36.0,
            );
        });
}

fn modal_buttons_single(parent: &mut ChildSpawnerCommands, fonts: &UiAssets, label: &str) {
    parent
        .spawn(Node {
            justify_content: JustifyContent::FlexEnd,
            margin: UiRect::top(px(6.0)),
            ..default()
        })
        .with_children(|row| {
            spawn_button(
                row,
                fonts,
                label,
                ButtonKind::Primary,
                AppAction::CloseModal,
                Some(92.0),
                36.0,
            );
        });
}

fn modal_menu_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    label: &str,
    action: AppAction,
) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            ActionButton(action),
            ButtonStyle {
                normal: Color::srgba(0.03, 0.06, 0.10, 0.92),
                hover: Color::srgba(0.07, 0.13, 0.2, 1.0),
                pressed: Color::srgba(0.02, 0.04, 0.07, 1.0),
                border: INPUT_LINE,
            },
            Node {
                width: percent(100.0),
                min_height: px(38.0),
                padding: UiRect::horizontal(px(11.0)),
                align_items: AlignItems::Center,
                border: px(1.0).all(),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.10, 0.92)),
            BorderColor::all(INPUT_LINE),
        ))
        .with_child((
            Text::new(label),
            text_font(fonts, 11.5, false),
            TextColor(INK),
        ));
}

fn shortcut_line(
    parent: &mut ChildSpawnerCommands,
    fonts: &UiAssets,
    key: &str,
    description: &str,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(10.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(76.0),
                    padding: UiRect::axes(px(6.0), px(4.0)),
                    justify_content: JustifyContent::Center,
                    border: px(1.0).all(),
                    border_radius: BorderRadius::all(px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 1.0)),
                BorderColor::all(INPUT_LINE),
            ))
            .with_child((
                Text::new(key),
                text_font(fonts, 10.0, true),
                TextColor(ACCENT),
            ));
            row.spawn((
                Text::new(description),
                text_font(fonts, 11.0, false),
                TextColor(MUTED),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub fn sync_modal_blocking(
    state: Res<AppState>,
    mut roots: Query<(&mut Node, &mut Pickable), With<ModalRoot>>,
) {
    for (mut node, mut pickable) in &mut roots {
        let open = state.modal.is_some() && state.modal != Some(ModalKind::Globe);
        node.display = if open { Display::Flex } else { Display::None };
        *pickable = if open {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}

pub fn ensure_ui_pickable(
    mut commands: Commands,
    buttons: Query<Entity, Added<Button>>,
    inputs: Query<Entity, (Added<TextInput>, Without<Pickable>)>,
) {
    for entity in &buttons {
        commands
            .entity(entity)
            .insert((Pickable::default(), ActivateOnPress));
    }
    for entity in &inputs {
        commands.entity(entity).insert(Pickable::default());
    }
}

pub fn ignore_text_picking(
    mut commands: Commands,
    texts: Query<Entity, (Added<Text>, Without<Pickable>)>,
) {
    for entity in &texts {
        commands.entity(entity).insert(Pickable::IGNORE);
    }
}

pub fn sync_search(
    inputs: Query<&EditableText, (With<SearchInput>, Changed<EditableText>)>,
    mut state: ResMut<AppState>,
) {
    let Ok(input) = inputs.single() else {
        return;
    };
    let value = input.value().to_string();
    if value != state.search {
        state.search = value;
        state.ui.list = true;
    }
}

pub fn sync_editor(
    titles: Query<&EditableText, (With<TitleInput>, Changed<EditableText>)>,
    bodies: Query<&EditableText, (With<BodyInput>, Changed<EditableText>)>,
    mut state: ResMut<AppState>,
) {
    if let Ok(input) = titles.single() {
        let value = input.value().to_string();
        if value != state.editor_title {
            state.editor_title = value.clone();
            state.update_selected_title(&value);
        }
    }
    if let Ok(input) = bodies.single() {
        let value = input.value().to_string();
        if value != state.editor_body {
            state.editor_body = value.clone();
            state.update_selected_body(&value);
        }
    }
}

pub fn sync_modal_input(
    inputs: Query<&EditableText, (With<ModalInput>, Changed<EditableText>)>,
    mut state: ResMut<AppState>,
) {
    let Ok(input) = inputs.single() else {
        return;
    };
    let value = input.value().to_string();
    if value != state.modal_input {
        state.modal_input = value;
    }
}

#[allow(clippy::type_complexity)]
pub fn style_buttons(
    mut buttons: Query<
        (
            &Hovered,
            Has<Pressed>,
            &ButtonStyle,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (hovered, pressed, style, mut background, mut border) in &mut buttons {
        background.0 = if pressed {
            style.pressed
        } else if hovered.get() {
            style.hover
        } else {
            style.normal
        };
        *border = BorderColor::all(style.border);
    }
}

#[allow(clippy::type_complexity)]
pub fn style_inputs(
    focus: Res<InputFocus>,
    mut inputs: Query<
        (Entity, &mut BorderColor, &mut BackgroundColor),
        (With<EditableText>, Without<Button>),
    >,
) {
    for (entity, mut border, mut background) in &mut inputs {
        let focused = focus.get() == Some(entity);
        *border = BorderColor::all(if focused { ACCENT } else { INPUT_LINE });
        background.0 = if focused {
            Color::srgba(0.025, 0.055, 0.09, 1.0)
        } else {
            INPUT_BG
        };
    }
}

pub fn update_top_labels(
    state: Res<AppState>,
    mut title: Query<&mut Text, With<TopTitle>>,
    mut meta: Query<&mut Text, (With<TopMeta>, Without<TopTitle>)>,
) {
    let Some(sphere) = state.current() else {
        return;
    };
    if let Ok(mut text) = title.single_mut() {
        text.0 = sphere.title.clone();
    }
    if let Ok(mut text) = meta.single_mut() {
        text.0 = format!(
            "{} thoughts · {} connections · {} groups",
            sphere.nodes.len(),
            sphere.links.len(),
            sphere.groups.len()
        );
    }
}

pub fn update_save_indicator(
    state: Res<AppState>,
    mut save: Query<&mut Text, With<SaveLabel>>,
    mut save_dot: Query<&mut BackgroundColor, With<SaveDot>>,
) {
    if let Ok(mut text) = save.single_mut() {
        text.0 = if state.dirty {
            "Saving…".to_owned()
        } else {
            "Saved locally".to_owned()
        };
    }
    if let Ok(mut color) = save_dot.single_mut() {
        color.0 = if state.dirty { AMBER } else { MINT };
    }
}

pub fn update_stats(state: Res<AppState>, mut stats: Query<(&StatCount, &mut Text)>) {
    let Some(sphere) = state.current() else {
        return;
    };
    for (kind, mut text) in &mut stats {
        text.0 = match kind.0 {
            StatKind::Thoughts => sphere.nodes.len().to_string(),
            StatKind::Links => sphere.links.len().to_string(),
            StatKind::Groups => sphere.groups.len().to_string(),
        };
    }
}

pub fn update_modes(state: Res<AppState>, mut modes: Query<(&ModeButton, &mut BackgroundColor)>) {
    for (mode, mut color) in &mut modes {
        color.0 = if state.mode == mode.0 {
            ACCENT_SOFT
        } else {
            Color::srgba(0.04, 0.08, 0.13, 0.9)
        };
    }
}

pub fn update_toast(
    state: Res<AppState>,
    mut root: Query<&mut Node, With<ToastRoot>>,
    mut panel: Query<&mut BackgroundColor, With<ToastPanel>>,
    mut text: Query<&mut Text, With<ToastText>>,
) {
    if let Ok(mut node) = root.single_mut() {
        node.display = if state.toast.timer > 0.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Ok(mut color) = panel.single_mut() {
        color.0 = match state.toast.tone {
            Tone::Mint => Color::srgba(0.05, 0.15, 0.12, 0.98),
            Tone::Amber => Color::srgba(0.16, 0.10, 0.03, 0.98),
            Tone::Coral => alpha(CORAL, 0.22),
        };
    }
    if let Ok(mut label) = text.single_mut() {
        label.0 = state.toast.text.clone();
    }
}

#[allow(clippy::type_complexity)]
pub fn scroll_lists(
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    state: Res<AppState>,
    mut list: Query<
        (&mut ScrollPosition, &ComputedNode),
        (
            With<SphereListScroll>,
            Without<BoardScroll>,
            Without<ModalContent>,
        ),
    >,
    mut board: Query<
        (&mut ScrollPosition, &ComputedNode),
        (
            With<BoardScroll>,
            Without<SphereListScroll>,
            Without<ModalContent>,
        ),
    >,
    mut modal: Query<
        (&mut ScrollPosition, &ComputedNode),
        (
            With<ModalContent>,
            Without<SphereListScroll>,
            Without<BoardScroll>,
        ),
    >,
    mut globe_panel: Query<
        (&mut ScrollPosition, &ComputedNode),
        (
            With<GlobePanelScroll>,
            Without<SphereListScroll>,
            Without<BoardScroll>,
            Without<ModalContent>,
        ),
    >,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let mut delta = 0.0;
    for event in wheel.read() {
        delta += match event.unit {
            MouseScrollUnit::Line => event.y * 34.0,
            MouseScrollUnit::Pixel => event.y,
        };
    }
    if delta == 0.0 {
        return;
    }
    let height = window.resolution.height();
    if state.modal == Some(ModalKind::Globe) && cursor.x > window.resolution.width() - 400.0 {
        if let Ok((mut position, computed)) = globe_panel.single_mut() {
            let max = (computed.content_size().y - computed.size().y).max(0.0)
                * computed.inverse_scale_factor();
            position.y = (position.y - delta).clamp(0.0, max);
        }
        return;
    }
    if state.modal.is_some()
        && cursor.x > 400.0
        && cursor.x < window.resolution.width() - 400.0
        && cursor.y > 64.0
    {
        if let Ok((mut position, computed)) = modal.single_mut() {
            let max = (computed.content_size().y - computed.size().y).max(0.0)
                * computed.inverse_scale_factor();
            position.y = (position.y - delta).clamp(0.0, max);
        }
        return;
    }
    if cursor.x < LEFT_PANEL_W && cursor.y > 180.0 && cursor.y < height - 130.0 {
        if let Ok((mut position, computed)) = list.single_mut() {
            let max = (computed.content_size().y - computed.size().y).max(0.0)
                * computed.inverse_scale_factor();
            position.y = (position.y - delta).clamp(0.0, max);
        }
    } else if cursor.x > LEFT_PANEL_W
        && cursor.x < window.resolution.width() - RIGHT_PANEL_W
        && cursor.y > TOP_BAR_H
        && let Ok((mut position, computed)) = board.single_mut()
    {
        let max = (computed.content_size().x - computed.size().x).max(0.0)
            * computed.inverse_scale_factor();
        position.x = (position.x - delta).clamp(0.0, max);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_node_labels(
    mut commands: Commands,
    state: Res<AppState>,
    fonts: Res<UiAssets>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    window: Single<&Window>,
    root: Query<Entity, With<NodeLabelsRoot>>,
    mut map: ResMut<NodeLabelMap>,
    mut labels: Query<(
        &NodeLabel,
        &mut Node,
        &ComputedNode,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut texts: Query<&mut Text>,
) {
    let Ok(root) = root.single() else {
        return;
    };
    let Some(sphere) = state.current() else {
        return;
    };
    if state.mode == ViewMode::Board {
        for (_, mut node, ..) in &mut labels {
            node.display = Display::None;
        }
        return;
    }
    let live: HashSet<&str> = sphere.nodes.iter().map(|node| node.id.as_str()).collect();
    let stale: Vec<String> = map
        .entities
        .keys()
        .filter(|id| !live.contains(id.as_str()))
        .cloned()
        .collect();
    for id in stale {
        if let Some(entity) = map.entities.remove(&id) {
            commands.entity(entity).despawn();
        }
    }

    for node in &sphere.nodes {
        if !map.entities.contains_key(&node.id) {
            let color = color_from_hex(sphere.group_color(&node.group_id));
            let entity = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        padding: UiRect::axes(px(6.0), px(3.0)),
                        border: px(1.0).all(),
                        border_radius: BorderRadius::all(px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.05, 0.09, 0.84)),
                    BorderColor::all(alpha(color, 0.34)),
                    NodeLabel(node.id.clone()),
                    ChildOf(root),
                    Pickable::IGNORE,
                    ZIndex(4),
                ))
                .with_child((
                    Text::new(node.title.clone()),
                    text_font(&fonts, 10.5, false),
                    TextColor(if state.selected_id.as_deref() == Some(node.id.as_str()) {
                        Color::WHITE
                    } else {
                        lighten_ui(color, 0.32)
                    }),
                ))
                .id();
            map.entities.insert(node.id.clone(), entity);
        }
    }

    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let camera_position = camera_transform.translation();
    let width = window.resolution.width();
    let height = window.resolution.height();

    for (label, mut node, computed, children, mut background, mut border) in &mut labels {
        let Some(thought) = sphere.node(&label.0) else {
            node.display = Display::None;
            continue;
        };
        let world = Vec3::from_array(thought.pos) * (sphere.radius + crate::util::NODE_LIFT);
        let normal = world.normalize_or_zero();
        let to_camera = (camera_position - world).normalize_or_zero();
        let facing = normal.dot(to_camera) > 0.035;
        let screen = camera.world_to_viewport(camera_transform, world).ok();
        let Some(screen) = screen.filter(|_| facing) else {
            node.display = Display::None;
            continue;
        };
        if screen.x < LEFT_PANEL_W + 8.0
            || screen.x > width - RIGHT_PANEL_W - 8.0
            || screen.y < TOP_BAR_H + 4.0
            || screen.y > height - BOTTOM_DOCK_H - 4.0
        {
            node.display = Display::None;
            continue;
        }
        let size = computed.size() * computed.inverse_scale_factor();
        node.display = Display::Flex;
        node.left = px(screen.x - size.x * 0.5);
        node.top = px(screen.y - size.y - 20.0);

        let color = color_from_hex(sphere.group_color(&thought.group_id));
        let selected = state.selected_ids.iter().any(|id| id == &thought.id);
        let unlink_target = state.unlink_targets.iter().any(|id| id == &thought.id);
        background.0 = if unlink_target {
            Color::srgba(0.24, 0.12, 0.04, 0.96)
        } else if selected {
            Color::srgba(0.07, 0.22, 0.26, 0.94)
        } else {
            Color::srgba(0.02, 0.05, 0.09, 0.84)
        };
        *border = BorderColor::all(if unlink_target {
            AMBER
        } else if selected {
            ACCENT
        } else {
            alpha(color, 0.34)
        });
        if let Some(&child) = children.first()
            && let Ok(mut text) = texts.get_mut(child)
            && text.0 != thought.title
        {
            text.0 = thought.title.clone();
        }
    }
}

fn lighten_ui(color: Color, amount: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(
        (c.red + (1.0 - c.red) * amount).min(1.0),
        (c.green + (1.0 - c.green) * amount).min(1.0),
        (c.blue + (1.0 - c.blue) * amount).min(1.0),
        c.alpha,
    )
}

pub fn sync_slider_value(value_change: On<ValueChange<f32>>, mut commands: Commands) {
    commands
        .entity(value_change.source)
        .insert(SliderValue(value_change.value));
}

pub fn globe_sliders(
    radius: Query<(&SliderValue, &GlobeRadiusSlider), Changed<SliderValue>>,
    transparency: Query<(&SliderValue, &GlobeTransparencySlider), Changed<SliderValue>>,
    mut state: ResMut<AppState>,
) {
    for (value, _) in &radius {
        state.set_sphere_radius(value.0);
    }
    for (value, _) in &transparency {
        state.set_sphere_transparency(value.0);
    }
}

#[allow(clippy::type_complexity)]
pub fn update_globe_slider_visuals(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        Or<(With<GlobeRadiusSlider>, With<GlobeTransparencySlider>)>,
    >,
    children: Query<&Children>,
    mut thumbs: Query<&mut Node, With<GlobeSliderThumb>>,
) {
    for (entity, value, range) in &sliders {
        for child in children.iter_descendants(entity) {
            if let Ok(mut thumb) = thumbs.get_mut(child) {
                thumb.left = percent(range.thumb_position(value.0) * 100.0);
            }
        }
    }
}

pub fn update_globe_value_labels(
    state: Res<AppState>,
    mut radius: Query<&mut Text, With<GlobeRadiusValue>>,
    mut transparency: Query<&mut Text, (With<GlobeTransparencyValue>, Without<GlobeRadiusValue>)>,
) {
    let Some(sphere) = state.current() else {
        return;
    };
    if let Ok(mut text) = radius.single_mut() {
        text.0 = format!("{:.1}", sphere.radius);
    }
    if let Ok(mut text) = transparency.single_mut() {
        text.0 = format!("{:.0}%", sphere.transparency * 100.0);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn board_release_drop(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut drag: ResMut<BoardDragState>,
    mut state: ResMut<AppState>,
    card_rects: Query<(&BoardCard, &ComputedNode, &UiGlobalTransform)>,
    column_rects: Query<(&BoardColumn, &ComputedNode, &UiGlobalTransform)>,
    mut outlines: Query<&mut Outline>,
) {
    if !buttons.just_released(MouseButton::Left) || drag.entity.is_none() {
        return;
    }
    let entity = drag.entity;
    let Some(cursor) = window.cursor_position() else {
        let _ = &mut drag;
        return;
    };
    let point = cursor * window.scale_factor();
    let target_card = card_rects
        .iter()
        .find(|(_, node, transform)| node.contains_point(**transform, point))
        .map(|(card, _, _)| card.0.clone());
    let target_group = column_rects
        .iter()
        .find(|(_, node, transform)| node.contains_point(**transform, point))
        .map(|(column, _, _)| column.0.clone())
        .or_else(|| {
            target_card.as_deref().and_then(|card_id| {
                state
                    .current()
                    .and_then(|sphere| sphere.node(card_id))
                    .map(|node| node.group_id.clone())
            })
        });
    if let Some(group) = drag.group.clone() {
        if let Some(target_group) = target_group
            && !target_group.is_empty()
            && group != target_group
        {
            state.reorder_group(&group, &target_group);
        }
    } else if let Some(node) = drag.node.clone()
        && let Some(target_group) = target_group
    {
        let after = target_card.filter(|target| *target != node);
        state.move_node_board(&node, &target_group, after.as_deref());
    }
    if let Some(entity) = entity
        && let Ok(mut outline) = outlines.get_mut(entity)
    {
        outline.color = Color::NONE;
    }
    drag.group = None;
    drag.node = None;
    drag.entity = None;
}

pub fn board_drag_start(
    event: On<PointerDragStart>,
    cards: Query<&BoardCard>,
    handles: Query<&BoardColumnHandle>,
    mut drag: ResMut<BoardDragState>,
    mut outlines: Query<&mut Outline>,
) {
    let entity = event.event_target();
    if let Ok(card) = cards.get(entity) {
        drag.group = None;
        drag.node = Some(card.0.clone());
        drag.entity = Some(entity);
    } else if let Ok(handle) = handles.get(entity) {
        drag.node = None;
        drag.group = Some(handle.0.clone());
        drag.entity = Some(entity);
    } else {
        return;
    }
    if let Ok(mut outline) = outlines.get_mut(entity) {
        outline.color = ACCENT;
    }
}

#[allow(clippy::type_complexity)]
pub fn update_action_buttons(
    state: Res<AppState>,
    mut buttons: Query<
        (
            &mut Node,
            Has<NewThoughtButton>,
            Has<AddGroupButton>,
            Has<RemoveGroupButton>,
        ),
        Or<(
            With<NewThoughtButton>,
            With<AddGroupButton>,
            With<RemoveGroupButton>,
        )>,
    >,
) {
    let multi = state.selected_ids.len() > 1;
    for (mut node, new_thought, add_group, remove_group) in &mut buttons {
        node.display = if new_thought {
            if multi { Display::None } else { Display::Flex }
        } else if add_group || remove_group {
            if multi { Display::Flex } else { Display::None }
        } else {
            Display::Flex
        };
    }
}

#[allow(clippy::type_complexity)]
pub fn update_link_button(
    state: Res<AppState>,
    mut buttons: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            Has<LinkButton>,
            Has<MultiButton>,
        ),
        Or<(With<LinkButton>, With<MultiButton>)>,
    >,
) {
    for (mut background, mut border, is_link, is_multi) in &mut buttons {
        let active = (is_link && state.link_mode) || (is_multi && state.multi_select_mode);
        if active {
            background.0 = Color::srgba(0.18, 0.52, 0.58, 0.55);
            *border = BorderColor::all(ACCENT);
        } else {
            background.0 = Color::srgba(0.04, 0.08, 0.13, 0.9);
            *border = BorderColor::all(LINE);
        }
    }
}

pub fn update_console(
    state: Res<AppState>,
    mut line1: Query<&mut Text, With<ConsoleLine1>>,
    mut line2: Query<&mut Text, (With<ConsoleLine2>, Without<ConsoleLine1>)>,
) {
    if let Ok(mut text) = line1.single_mut()
        && text.0 != state.console_line1
    {
        text.0 = state.console_line1.clone();
    }
    if let Ok(mut text) = line2.single_mut()
        && text.0 != state.console_line2
    {
        text.0 = state.console_line2.clone();
    }
}
