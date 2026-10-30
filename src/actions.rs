use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::state::{ActionMessage, AppAction, AppState, ModalKind, Tone, ViewMode};
use crate::{storage, ui};

#[cfg(target_arch = "wasm32")]
use crate::state::WebImportState;

pub fn handle_actions(
    mut reader: MessageReader<ActionMessage>,
    mut state: ResMut<AppState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let additive = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        || keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    for ActionMessage(action) in reader.read() {
        match action.clone() {
            AppAction::NewThought => state.open_new_thought_modal(),
            AppAction::SelectSphere(id) => state.select_sphere(&id),
            AppAction::SelectNode(id) => state.select_node_with_modifier(&id, additive),
            AppAction::SetMode(mode) => state.set_mode(mode),
            AppAction::NewSphere | AppAction::OpenModal(ModalKind::NewSphere) => {
                state.open_modal(ModalKind::NewSphere)
            }
            AppAction::RenameSphere | AppAction::OpenModal(ModalKind::RenameSphere) => {
                state.open_modal(ModalKind::RenameSphere)
            }
            AppAction::DeleteSphere | AppAction::OpenModal(ModalKind::DeleteSphere) => {
                state.open_modal(ModalKind::DeleteSphere)
            }
            AppAction::NewGroup | AppAction::OpenModal(ModalKind::NewGroup) => {
                state.open_modal(ModalKind::NewGroup)
            }
            AppAction::ImportSphere | AppAction::OpenModal(ModalKind::Import) => {
                state.open_modal(ModalKind::Import)
            }
            AppAction::More => state.open_modal(ModalKind::More),
            AppAction::OpenModal(kind) => state.open_modal(kind),
            AppAction::DuplicateSphere => {
                state.duplicate_current();
                state.close_modal();
            }
            AppAction::ExportSphere => {
                state.export_current();
                state.close_modal();
            }
            AppAction::CloseModal => {
                state.close_modal();
                state.set_console("Ready.", "Select a thought or use the controls.");
            }
            AppAction::SubmitModal => state.submit_modal(),
            AppAction::MoveToGroup(group_id) => state.move_selected_to_group(&group_id),
            AppAction::ConnectTo(node_id) => state.connect_selected_to(&node_id),
            AppAction::OpenLink(url) => open_link(&url),
            AppAction::SetSkin(skin) => {
                state.set_skin(&skin);
                state.modal_rev = state.modal_rev.wrapping_add(1);
            }
            AppAction::SetBackground(background) => {
                state.set_background(&background);
                state.modal_rev = state.modal_rev.wrapping_add(1);
            }
            AppAction::SetGroupColor(group, color) => {
                state.set_group_color(&group, &color);
                state.modal_rev = state.modal_rev.wrapping_add(1);
            }
            AppAction::SetGrid(grid) => {
                state.set_grid(&grid);
                state.modal_rev = state.modal_rev.wrapping_add(1);
            }
            AppAction::ToggleLinkMode => state.toggle_link_mode(),
            AppAction::ToggleMultiSelect => state.toggle_multi_select(),
            AppAction::EraseLinks => state.erase_selected_links(),
            AppAction::AddSelectedToGroup => state.open_add_to_group_modal(),
            AppAction::RemoveSelectedFromGroup => state.remove_selected_from_groups(),
            AppAction::ToggleGroupDropdown => state.toggle_group_dropdown(),
            AppAction::SetModalGroup(group) => state.set_modal_group(&group),
        }
    }
}

pub fn keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AppState>,
    text_inputs: Query<(), With<EditableText>>,
    search_input: Query<Entity, With<ui::SearchInput>>,
    mut focus: ResMut<InputFocus>,
) {
    let editing = focus
        .get()
        .is_some_and(|entity| text_inputs.contains(entity));
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    if keyboard.just_pressed(KeyCode::Escape) {
        if state.modal.is_some() {
            state.close_modal();
        } else if !state.cancel_modes() {
            state.clear_selection();
        }
        return;
    }

    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        state.save_now();
        state.toast("Saved locally", Tone::Mint);
        return;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyE) {
        state.export_current();
        return;
    }
    if ctrl && keyboard.just_pressed(KeyCode::KeyF) {
        if let Ok(entity) = search_input.single() {
            focus.set(entity, FocusCause::Navigated);
        }
        return;
    }

    if editing {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        state.set_mode(ViewMode::Sphere);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        state.set_mode(ViewMode::Board);
    } else if keyboard.just_pressed(KeyCode::KeyN) {
        state.open_new_thought_modal();
    } else if keyboard.just_pressed(KeyCode::KeyL) {
        state.toggle_link_mode();
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        state.toggle_multi_select();
    } else if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        state.delete_selected();
    }
}

pub fn update_cursor(
    state: Res<AppState>,
    window: Single<Entity, With<Window>>,
    mut commands: Commands,
    mut last: Local<Option<bool>>,
) {
    let active = state.link_mode || state.unlink_mode;
    if *last == Some(active) {
        return;
    }
    *last = Some(active);
    if active {
        commands
            .entity(*window)
            .insert(CursorIcon::from(SystemCursorIcon::Crosshair));
    } else {
        commands.entity(*window).remove::<CursorIcon>();
    }
}

pub fn autosave(time: Res<Time>, mut state: ResMut<AppState>) {
    state.tick_autosave(time.delta_secs());
    state.tick_toast(time.delta_secs());
}

#[cfg(target_arch = "wasm32")]
pub fn poll_web_import(mut state: ResMut<AppState>, mut web: ResMut<WebImportState>) {
    let Some((nonce, text)) = storage::poll_import() else {
        return;
    };
    if nonce == web.last_nonce {
        return;
    }
    web.last_nonce = nonce;
    if !text.trim().is_empty() {
        state.import_text(&text);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn poll_web_import() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn handle_file_drop(
    mut reader: MessageReader<bevy::window::FileDragAndDrop>,
    mut state: ResMut<AppState>,
) {
    for event in reader.read() {
        if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            if path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                if let Some(text) = storage::read_text_file(path_buf) {
                    state.import_text(&text);
                }
            } else {
                state.toast("Drop a .json sphere file to import it.", Tone::Amber);
            }
        }
    }
}

pub fn open_link(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "macos"),
        not(target_os = "windows")
    ))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
