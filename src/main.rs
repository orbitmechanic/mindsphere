mod actions;
#[cfg(not(target_arch = "wasm32"))]
mod capture;
mod forces;
mod model;
mod scene;
mod skins;
mod state;
mod storage;
mod ui;
mod util;

use bevy::asset::AssetMetaCheck;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::{MonitorSelection, WindowMode, WindowResolution};

use actions::{autosave, handle_actions, keyboard_shortcuts, poll_web_import, update_cursor};
use forces::{ForceState, simulate};
use scene::{
    DragState, SceneMap, camera_controls, draw_globe, setup_scene, sync_globe, sync_scene,
    update_camera,
};
use state::{ActionMessage, AppState, WebImportState};
use ui::{
    ActionButton, BoardDragState, NodeLabelMap, board_release_drop, ensure_ui_pickable,
    globe_sliders, ignore_text_picking, rebuild_board, rebuild_globe_panel, rebuild_inspector,
    rebuild_modal, rebuild_sphere_list, scroll_lists, setup_ui, style_buttons, style_inputs,
    sync_editor, sync_modal_blocking, sync_modal_input, sync_node_labels, sync_search,
    sync_slider_value, update_action_buttons, update_console, update_globe_slider_visuals,
    update_globe_value_labels, update_link_button, update_modes, update_save_indicator,
    update_stats, update_toast, update_top_labels,
};

fn main() {
    let state = AppState::load();
    let capture_fullscreen = std::env::var("MINDSPHERE_FULLSCREEN").is_ok();
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.012, 0.024, 0.045)))
        .insert_resource(state)
        .init_resource::<WebImportState>()
        .init_resource::<DragState>()
        .init_resource::<SceneMap>()
        .init_resource::<ForceState>()
        .init_resource::<NodeLabelMap>()
        .init_resource::<BoardDragState>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Mindsphere".to_owned(),
                        resolution: WindowResolution::new(1440, 900),
                        mode: if capture_fullscreen {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                        } else {
                            WindowMode::Windowed
                        },
                        resizable: true,
                        canvas: Some("#game-canvas".to_owned()),
                        fit_canvas_to_parent: false,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(TabNavigationPlugin)
        .add_message::<ActionMessage>()
        .add_observer(on_activate)
        .add_observer(sync_slider_value)
        .add_systems(Startup, (setup_scene, setup_ui).chain())
        .add_systems(Update, update_link_button.after(style_buttons))
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (
                handle_actions,
                board_release_drop,
                keyboard_shortcuts,
                sync_search,
                sync_editor,
                sync_modal_input,
                poll_web_import,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                rebuild_sphere_list,
                rebuild_inspector,
                rebuild_board,
                rebuild_globe_panel,
                rebuild_modal,
                camera_controls,
                update_camera,
                sync_globe,
                sync_scene,
                sync_node_labels,
                draw_globe,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                ensure_ui_pickable,
                ignore_text_picking,
                globe_sliders,
                update_globe_slider_visuals,
                update_globe_value_labels,
                update_action_buttons,
                style_buttons,
                style_inputs,
                sync_modal_blocking,
                update_top_labels,
                update_save_indicator,
                update_stats,
                update_modes,
                update_console,
                update_cursor,
                update_toast,
                autosave,
                scroll_lists,
            ),
        );

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, actions::handle_file_drop);

    #[cfg(not(target_arch = "wasm32"))]
    capture::install_if_requested(&mut app);

    app.run();
}

fn on_activate(
    event: On<Activate>,
    buttons: Query<&ActionButton>,
    mut actions: MessageWriter<ActionMessage>,
) {
    if let Ok(button) = buttons.get(event.event_target()) {
        actions.write(ActionMessage(button.0.clone()));
    }
}
