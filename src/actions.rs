use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::text::EditableText;

use crate::state::{ActionMessage, AppAction, AppState, ModalKind, Tone, ViewMode};
use crate::{storage, ui};

#[cfg(target_arch = "wasm32")]
use crate::state::WebImportState;

pub fn handle_actions(mut reader: MessageReader<ActionMessage>, mut state: ResMut<AppState>) {
    for ActionMessage(action) in reader.read() {
        match action.clone() {
            AppAction::NewThought => state.new_thought(),
            AppAction::SelectSphere(id) => state.select_sphere(&id),
            AppAction::SelectNode(id) => state.select_node(&id),
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
            AppAction::CloseModal => state.close_modal(),
            AppAction::SubmitModal => state.submit_modal(),
            AppAction::MoveToGroup(group_id) => state.move_selected_to_group(&group_id),
            AppAction::ConnectTo(node_id) => state.connect_selected_to(&node_id),
            AppAction::OpenLink(url) => open_link(&url),
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
        } else {
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
        state.new_thought();
    } else if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        state.delete_selected();
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
