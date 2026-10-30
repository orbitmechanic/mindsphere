use bevy::prelude::*;

use crate::model::{CameraData, Settings, Sphere, parse_sphere, spherical};
use crate::storage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Sphere,
    Board,
}

impl ViewMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sphere => "sphere",
            Self::Board => "board",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalKind {
    NewSphere,
    RenameSphere,
    DeleteSphere,
    NewGroup,
    Import,
    More,
    Shortcuts,
    About,
}

#[derive(Clone, Copy, Debug)]
pub enum Tone {
    Mint,
    Amber,
    Coral,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub tone: Tone,
    pub timer: f32,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            text: String::new(),
            tone: Tone::Mint,
            timer: 0.0,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct UiDirty {
    pub list: bool,
    pub inspector: bool,
    pub board: bool,
    pub scene: bool,
    pub toolbar: bool,
}

#[derive(Resource)]
pub struct AppState {
    pub library: Vec<Sphere>,
    pub settings: Settings,
    pub current_id: String,
    pub selected_id: Option<String>,
    pub mode: ViewMode,
    pub search: String,
    pub dirty: bool,
    pub save_timer: f32,
    pub toast: Toast,
    pub modal: Option<ModalKind>,
    pub modal_input: String,
    pub modal_rev: u64,
    pub ui: UiDirty,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub editor_title: String,
    pub editor_body: String,
}

impl AppState {
    pub fn load() -> Self {
        let mut library = storage::load_library();
        if library.is_empty() {
            let welcome = Sphere::welcome();
            let _ = storage::save_library(std::slice::from_ref(&welcome));
            library.push(welcome);
        }
        let settings = storage::load_settings();
        let current_id = if library.iter().any(|s| s.id == settings.last_sphere) {
            settings.last_sphere.clone()
        } else {
            library[0].id.clone()
        };
        let mode = if settings.mode == "board" {
            ViewMode::Board
        } else {
            ViewMode::Sphere
        };
        let camera = library
            .iter()
            .find(|s| s.id == current_id)
            .map(|s| s.camera.clone())
            .unwrap_or_default();

        let mut state = Self {
            library,
            settings,
            current_id,
            selected_id: None,
            mode,
            search: String::new(),
            dirty: false,
            save_timer: 0.0,
            toast: Toast::default(),
            modal: None,
            modal_input: String::new(),
            modal_rev: 0,
            ui: UiDirty {
                list: true,
                inspector: true,
                board: true,
                scene: true,
                toolbar: true,
            },
            camera_yaw: camera.yaw.to_radians(),
            camera_pitch: camera.pitch.to_radians(),
            camera_distance: camera.distance,
            editor_title: String::new(),
            editor_body: String::new(),
        };
        state.editor_mirror_from_selection();
        state.settings.last_sphere = state.current_id.clone();
        state.settings.mode = state.mode.as_str().to_owned();
        let _ = storage::save_library(&state.library);
        state
    }

    pub fn current(&self) -> Option<&Sphere> {
        self.library
            .iter()
            .find(|sphere| sphere.id == self.current_id)
    }

    pub fn current_mut(&mut self) -> Option<&mut Sphere> {
        let id = self.current_id.clone();
        self.library.iter_mut().find(|sphere| sphere.id == id)
    }

    pub fn selected_node(&self) -> Option<&crate::model::Thought> {
        let id = self.selected_id.as_deref()?;
        self.current()?.node(id)
    }

    pub fn select_sphere(&mut self, id: &str) {
        if id == self.current_id {
            return;
        }
        if self.dirty {
            self.save_now();
        }
        if let Some(sphere) = self.library.iter().find(|sphere| sphere.id == id) {
            self.current_id = id.to_owned();
            self.selected_id = None;
            let camera = &sphere.camera;
            self.camera_yaw = camera.yaw.to_radians();
            self.camera_pitch = camera.pitch.to_radians();
            self.camera_distance = camera.distance;
            self.settings.last_sphere = id.to_owned();
            storage::save_settings(&self.settings);
            self.mark_all();
            self.editor_mirror_from_selection();
        }
    }

    pub fn select_node(&mut self, id: &str) {
        if self
            .current()
            .is_some_and(|sphere| sphere.node(id).is_some())
        {
            self.selected_id = Some(id.to_owned());
            self.ui.inspector = true;
            self.ui.board = true;
            self.ui.scene = true;
            self.ui.toolbar = true;
            self.editor_mirror_from_selection();
        }
    }

    pub fn clear_selection(&mut self) {
        if self.selected_id.take().is_some() {
            self.ui.inspector = true;
            self.ui.board = true;
            self.ui.scene = true;
            self.ui.toolbar = true;
            self.editor_mirror_from_selection();
        }
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.settings.mode = mode.as_str().to_owned();
        storage::save_settings(&self.settings);
        self.ui.board = true;
        self.ui.toolbar = true;
    }

    pub fn new_thought(&mut self) {
        let selected_group = self
            .selected_node()
            .map(|node| node.group_id.clone())
            .unwrap_or_default();
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let group_id = if !selected_group.is_empty() {
            selected_group
        } else {
            sphere
                .groups
                .first()
                .map(|g| g.id.clone())
                .unwrap_or_default()
        };
        let index = sphere.nodes.len() as f32;
        let lat = (index * 47.0).sin() * 55.0;
        let lon = index * 137.5;
        let id = sphere.add_thought("New thought", &group_id, spherical(lat, lon), "");
        self.selected_id = Some(id);
        self.mark_data();
        self.ui.inspector = true;
        self.ui.scene = true;
        self.editor_mirror_from_selection();
        self.toast("Thought added", Tone::Mint);
    }

    pub fn update_selected_title(&mut self, title: &str) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        if let Some(node) = self.current_mut().and_then(|sphere| sphere.node_mut(&id)) {
            node.title = if title.trim().is_empty() {
                "Untitled thought".to_owned()
            } else {
                title.to_owned()
            };
            self.mark_data();
            self.ui.scene = true;
        }
    }

    pub fn update_selected_body(&mut self, body: &str) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        if let Some(node) = self.current_mut().and_then(|sphere| sphere.node_mut(&id)) {
            node.body = body.to_owned();
            self.mark_data();
        }
    }

    pub fn move_selected_to_group(&mut self, group_id: &str) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        if let Some(node) = self.current_mut().and_then(|sphere| sphere.node_mut(&id)) {
            node.group_id = group_id.to_owned();
            self.mark_data();
            self.ui.scene = true;
            self.ui.inspector = true;
        }
    }

    pub fn connect_selected_to(&mut self, other_id: &str) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        if let Some(sphere) = self.current_mut()
            && sphere.add_link(&id, other_id)
        {
            self.mark_data();
            self.ui.scene = true;
            self.ui.inspector = true;
            self.toast("Connection added", Tone::Mint);
        }
    }

    pub fn delete_selected(&mut self) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        if let Some(sphere) = self.current_mut() {
            sphere.remove_thought(&id);
            self.selected_id = None;
            self.mark_data();
            self.ui.scene = true;
            self.ui.inspector = true;
            self.editor_mirror_from_selection();
            self.toast("Thought deleted", Tone::Amber);
        }
    }

    pub fn set_node_position(&mut self, id: &str, pos: Vec3) {
        if let Some(node) = self.current_mut().and_then(|sphere| sphere.node_mut(id)) {
            let normalized = pos.normalize_or_zero();
            if normalized.length_squared() > 0.5 {
                node.pos = [normalized.x, normalized.y, normalized.z];
                self.mark_dirty();
                self.ui.scene = true;
            }
        }
    }

    pub fn new_sphere(&mut self, title: &str) {
        let sphere = Sphere::blank(title);
        self.current_id = sphere.id.clone();
        self.selected_id = None;
        self.library.push(sphere);
        self.settings.last_sphere = self.current_id.clone();
        self.camera_yaw = 0.0;
        self.camera_pitch = 8f32.to_radians();
        self.camera_distance = 8.8;
        self.mark_all();
        self.editor_mirror_from_selection();
        self.save_now();
        self.toast("Sphere created", Tone::Mint);
    }

    pub fn rename_current(&mut self, title: &str) {
        if title.trim().is_empty() {
            return;
        }
        if let Some(sphere) = self.current_mut() {
            sphere.title = title.trim().to_owned();
            self.mark_data();
            self.ui.list = true;
            self.ui.toolbar = true;
            self.toast("Sphere renamed", Tone::Mint);
        }
    }

    pub fn duplicate_current(&mut self) {
        let Some(copy) = self.current().map(Sphere::duplicate) else {
            return;
        };
        self.current_id = copy.id.clone();
        self.selected_id = None;
        self.library.push(copy);
        self.settings.last_sphere = self.current_id.clone();
        self.mark_all();
        self.editor_mirror_from_selection();
        self.save_now();
        self.toast("Sphere duplicated", Tone::Mint);
    }

    pub fn delete_current(&mut self) {
        let id = self.current_id.clone();
        self.library.retain(|sphere| sphere.id != id);
        if self.library.is_empty() {
            let welcome = Sphere::welcome();
            self.current_id = welcome.id.clone();
            self.library.push(welcome);
        } else {
            self.current_id = self.library[0].id.clone();
        }
        self.selected_id = None;
        self.settings.last_sphere = self.current_id.clone();
        self.mark_all();
        self.editor_mirror_from_selection();
        self.save_now();
        self.toast("Sphere deleted", Tone::Amber);
    }

    pub fn new_group(&mut self, name: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        if name.trim().is_empty() {
            return;
        }
        sphere.add_group(name, None);
        self.mark_data();
        self.ui.inspector = true;
        self.toast("Group created", Tone::Mint);
    }

    pub fn import_text(&mut self, text: &str) {
        let Some(sphere) = parse_sphere(text) else {
            self.toast("That file is not a valid sphere.", Tone::Coral);
            return;
        };
        self.current_id = sphere.id.clone();
        self.selected_id = None;
        self.library.push(sphere);
        self.settings.last_sphere = self.current_id.clone();
        self.mark_all();
        self.editor_mirror_from_selection();
        self.save_now();
        self.toast("Sphere imported. A new local copy was created.", Tone::Mint);
    }

    pub fn export_current(&mut self) {
        let Some(sphere) = self.current() else {
            return;
        };
        match storage::export_sphere(sphere) {
            Ok(message) => self.toast(&message, Tone::Mint),
            Err(message) => self.toast(&message, Tone::Coral),
        }
    }

    pub fn save_now(&mut self) {
        let camera = CameraData {
            yaw: self.camera_yaw.to_degrees(),
            pitch: self.camera_pitch.to_degrees(),
            distance: self.camera_distance,
        };
        if let Some(sphere) = self.current_mut() {
            sphere.updated_at = crate::model::now_secs();
            sphere.camera = camera;
        }
        let _ = storage::save_library(&self.library);
        self.settings.last_sphere = self.current_id.clone();
        self.settings.mode = self.mode.as_str().to_owned();
        storage::save_settings(&self.settings);
        self.dirty = false;
        self.save_timer = 0.0;
        self.ui.toolbar = true;
    }

    pub fn tick_autosave(&mut self, dt: f32) {
        if self.dirty {
            self.save_timer -= dt;
            if self.save_timer <= 0.0 {
                self.save_now();
            }
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_timer = 0.75;
        self.ui.toolbar = true;
    }

    pub fn mark_data(&mut self) {
        self.mark_dirty();
        self.ui.list = true;
        self.ui.board = true;
        self.ui.toolbar = true;
    }

    pub fn mark_all(&mut self) {
        self.mark_dirty();
        self.ui.list = true;
        self.ui.inspector = true;
        self.ui.board = true;
        self.ui.scene = true;
        self.ui.toolbar = true;
    }

    pub fn editor_mirror_from_selection(&mut self) {
        let mirror = self
            .selected_node()
            .map(|node| (node.title.clone(), node.body.clone()));
        if let Some((title, body)) = mirror {
            self.editor_title = title;
            self.editor_body = body;
        } else {
            self.editor_title.clear();
            self.editor_body.clear();
        }
    }

    pub fn open_modal(&mut self, kind: ModalKind) {
        self.modal = Some(kind);
        self.modal_input = match kind {
            ModalKind::RenameSphere => self
                .current()
                .map(|sphere| sphere.title.clone())
                .unwrap_or_default(),
            _ => String::new(),
        };
        self.modal_rev = self.modal_rev.wrapping_add(1);
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
        self.modal_input.clear();
        self.modal_rev = self.modal_rev.wrapping_add(1);
    }

    pub fn submit_modal(&mut self) {
        let Some(kind) = self.modal else {
            return;
        };
        let input = self.modal_input.clone();
        match kind {
            ModalKind::NewSphere => self.new_sphere(&input),
            ModalKind::RenameSphere => self.rename_current(&input),
            ModalKind::DeleteSphere => self.delete_current(),
            ModalKind::NewGroup => self.new_group(&input),
            ModalKind::Import => self.import_text(&input),
            ModalKind::More | ModalKind::Shortcuts | ModalKind::About => {}
        }
        self.close_modal();
    }

    pub fn toast(&mut self, text: &str, tone: Tone) {
        self.toast = Toast {
            text: text.to_owned(),
            tone,
            timer: 3.0,
        };
    }

    pub fn tick_toast(&mut self, dt: f32) {
        if self.toast.timer > 0.0 {
            self.toast.timer -= dt;
            if self.toast.timer <= 0.0 {
                self.toast.timer = 0.0;
            }
        }
    }

    pub fn filtered_spheres(&self) -> Vec<&Sphere> {
        let query = self.search.trim().to_lowercase();
        let mut result: Vec<&Sphere> = self
            .library
            .iter()
            .filter(|sphere| query.is_empty() || sphere.title.to_lowercase().contains(&query))
            .collect();
        result.sort_by_key(|sphere| std::cmp::Reverse(sphere.updated_at));
        result
    }
}

#[derive(Clone, Debug)]
pub enum AppAction {
    NewThought,
    SelectSphere(String),
    SelectNode(String),
    SetMode(ViewMode),
    NewSphere,
    RenameSphere,
    DuplicateSphere,
    DeleteSphere,
    ExportSphere,
    ImportSphere,
    More,
    NewGroup,
    OpenModal(ModalKind),
    CloseModal,
    SubmitModal,
    MoveToGroup(String),
    ConnectTo(String),
    OpenLink(String),
}

#[derive(Message, Clone, Debug)]
pub struct ActionMessage(pub AppAction);

#[derive(Resource, Default)]
pub struct WebImportState {
    #[cfg(target_arch = "wasm32")]
    pub last_nonce: u64,
}
