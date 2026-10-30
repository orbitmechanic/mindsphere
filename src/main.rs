mod actions;
#[cfg(not(target_arch = "wasm32"))]
mod capture;
mod model;
mod scene;
mod state;
mod storage;
mod ui;
mod util;

use bevy::asset::AssetMetaCheck;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::{MonitorSelection, WindowMode, WindowResolution};

use actions::{autosave, handle_actions, keyboard_shortcuts, poll_web_import};
use scene::{
    DragState, SceneMap, camera_controls, draw_globe, setup_scene, sync_scene, update_camera,
};
use state::{ActionMessage, AppState, WebImportState};
use ui::{
    ActionButton, NodeLabelMap, ensure_ui_pickable, ignore_text_picking, rebuild_board,
    rebuild_inspector, rebuild_modal, rebuild_sphere_list, scroll_lists, setup_ui, style_buttons,
    style_inputs, sync_editor, sync_modal_blocking, sync_modal_input, sync_node_labels,
    sync_search, update_modes, update_save_indicator, update_stats, update_toast,
    update_top_labels,
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
        .init_resource::<NodeLabelMap>()
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
        .add_systems(Startup, (setup_scene, setup_ui).chain())
        .add_systems(
            Update,
            (
                handle_actions,
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
                rebuild_modal,
                camera_controls,
                update_camera,
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
                style_buttons,
                style_inputs,
                sync_modal_blocking,
                update_top_labels,
                update_save_indicator,
                update_stats,
                update_modes,
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
