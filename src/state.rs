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
    Globe,
    NewThought,
    AddToGroup,
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
    pub globe: bool,
    pub toolbar: bool,
}

#[derive(Resource)]
pub struct AppState {
    pub library: Vec<Sphere>,
    pub settings: Settings,
    pub current_id: String,
    pub selected_id: Option<String>,
    pub selected_ids: Vec<String>,
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
    pub sim_active: bool,
    pub link_mode: bool,
    pub multi_select_mode: bool,
    pub link_chain: Vec<String>,
    pub unlink_mode: bool,
    pub unlink_source: Option<String>,
    pub unlink_targets: Vec<String>,
    pub modal_group: String,
    pub group_dropdown_open: bool,
    pub console_line1: String,
    pub console_line2: String,
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
            selected_ids: Vec::new(),
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
                globe: true,
                toolbar: true,
            },
            camera_yaw: camera.yaw.to_radians(),
            camera_pitch: camera.pitch.to_radians(),
            camera_distance: camera.distance,
            editor_title: String::new(),
            editor_body: String::new(),
            sim_active: true,
            link_mode: false,
            multi_select_mode: false,
            link_chain: Vec::new(),
            unlink_mode: false,
            unlink_source: None,
            unlink_targets: Vec::new(),
            modal_group: String::new(),
            group_dropdown_open: false,
            console_line1: "Select a thought or drag the globe.".to_owned(),
            console_line2: "N for a new thought · Globe for surface controls".to_owned(),
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
            self.selected_ids.clear();
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

    pub fn select_node_with_modifier(&mut self, id: &str, additive: bool) {
        if self
            .current()
            .is_some_and(|sphere| sphere.node(id).is_some())
        {
            let additive = additive || self.multi_select_mode;
            if self.link_mode {
                self.link_thought(id);
                return;
            }
            if self.unlink_mode {
                self.handle_unlink_click(id);
                return;
            }
            if additive {
                if let Some(index) = self.selected_ids.iter().position(|selected| selected == id) {
                    self.selected_ids.remove(index);
                    if self.selected_id.as_deref() == Some(id) {
                        self.selected_id = self.selected_ids.last().cloned();
                    }
                } else {
                    self.selected_ids.push(id.to_owned());
                    self.selected_id = Some(id.to_owned());
                }
            } else {
                self.selected_ids = vec![id.to_owned()];
                self.selected_id = Some(id.to_owned());
            }
            self.selection_changed();
        }
    }

    fn selection_changed(&mut self) {
        self.ui.inspector = true;
        self.ui.board = true;
        self.ui.scene = true;
        self.ui.toolbar = true;
        self.editor_mirror_from_selection();
        let count = self.selected_ids.len();
        if count > 1 {
            self.set_console(
                &format!("{count} thoughts selected."),
                "Use Add to group or Remove from group above.",
            );
        } else {
            self.set_console("Thought selected.", "Drag to move · Edit in the inspector");
        }
    }

    pub fn clear_selection(&mut self) {
        if self.selected_id.take().is_some() || !self.selected_ids.is_empty() {
            self.selected_ids.clear();
            self.link_chain.clear();
            self.ui.inspector = true;
            self.ui.board = true;
            self.ui.scene = true;
            self.ui.toolbar = true;
            self.editor_mirror_from_selection();
            if self.link_mode {
                self.link_chain.clear();
                self.set_console(
                    "Select the first thought to link.",
                    "Click thoughts in order; links chain as you go.",
                );
            } else if self.unlink_mode {
                self.set_console(
                    "Select the highlighted thought to unlink.",
                    "Press Esc to leave unlink mode.",
                );
            } else {
                self.set_console(
                    "Select a thought or drag the globe.",
                    "N for a new thought · Globe for surface controls",
                );
            }
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

    pub fn open_new_thought_modal(&mut self) {
        self.modal = Some(ModalKind::NewThought);
        self.modal_input.clear();
        self.modal_group.clear();
        self.group_dropdown_open = false;
        self.modal_rev = self.modal_rev.wrapping_add(1);
        self.set_console(
            "Enter a title for the new thought.",
            "Choose a group (default: None).",
        );
    }

    pub fn create_thought(&mut self, title: &str, group_id: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let pos = if !group_id.is_empty() {
            let members: Vec<Vec3> = sphere
                .nodes
                .iter()
                .filter(|node| node.group_id == group_id)
                .map(|node| Vec3::from_array(node.pos))
                .collect();
            if members.is_empty() {
                spherical(
                    sphere.nodes.len() as f32 * 37.0 - 20.0,
                    sphere.nodes.len() as f32 * 73.0,
                )
            } else {
                let center = members
                    .iter()
                    .fold(Vec3::ZERO, |sum, value| sum + *value)
                    .normalize_or_zero();
                [center.x, center.y, center.z]
            }
        } else {
            let index = sphere.nodes.len() as f32;
            spherical((index * 47.0).sin() * 55.0, index * 137.5)
        };
        let title = if title.trim().is_empty() {
            "New thought"
        } else {
            title.trim()
        };
        let id = sphere.add_thought(title, group_id, pos, "");
        self.selected_id = Some(id.clone());
        self.selected_ids = vec![id.clone()];
        self.mark_data();
        self.ui.inspector = true;
        self.ui.scene = true;
        self.editor_mirror_from_selection();
        self.set_console(
            "Thought added.",
            "Edit its title and notes in the inspector.",
        );
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
        let ids = self.selected_ids.clone();
        if ids.is_empty() {
            return;
        }
        if let Some(sphere) = self.current_mut() {
            for id in &ids {
                if let Some(node) = sphere.node_mut(id) {
                    node.group_id = group_id.to_owned();
                }
            }
        }
        self.mark_data();
        self.ui.scene = true;
        self.ui.inspector = true;
        self.set_console(
            if ids.len() == 1 {
                "Thought moved."
            } else {
                "Thoughts moved."
            },
            "The group layout will re-pack automatically.",
        );
    }

    pub fn remove_selected_from_groups(&mut self) {
        let ids = self.selected_ids.clone();
        if ids.is_empty() {
            return;
        }
        if let Some(sphere) = self.current_mut() {
            for id in &ids {
                if let Some(node) = sphere.node_mut(id) {
                    node.group_id.clear();
                }
            }
        }
        self.mark_data();
        self.ui.scene = true;
        self.ui.inspector = true;
        self.set_console("Removed from group.", "The thoughts are now unfiled.");
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
        let ids = self.selected_ids.clone();
        if ids.is_empty() {
            return;
        }
        if let Some(sphere) = self.current_mut() {
            for id in &ids {
                sphere.remove_thought(id);
            }
            self.selected_id = None;
            self.selected_ids.clear();
            self.mark_data();
            self.ui.scene = true;
            self.ui.inspector = true;
            self.editor_mirror_from_selection();
            self.set_console("Thoughts deleted.", "Select another thought to continue.");
            self.toast("Thoughts deleted", Tone::Amber);
        }
    }

    pub fn set_node_position(&mut self, id: &str, pos: Vec3) {
        if let Some(node) = self.current_mut().and_then(|sphere| sphere.node_mut(id)) {
            let normalized = pos.normalize_or_zero();
            if normalized.length_squared() > 0.5 {
                node.pos = [normalized.x, normalized.y, normalized.z];
                self.mark_dirty();
                self.ui.scene = true;
                self.sim_active = true;
            }
        }
    }

    pub fn set_console(&mut self, line1: &str, line2: &str) {
        self.console_line1 = line1.to_owned();
        self.console_line2 = line2.to_owned();
    }

    pub fn cancel_modes(&mut self) -> bool {
        let active = self.link_mode || self.unlink_mode || self.multi_select_mode;
        self.link_mode = false;
        self.unlink_mode = false;
        self.multi_select_mode = false;
        self.link_chain.clear();
        self.unlink_source = None;
        self.unlink_targets.clear();
        self.ui.scene = true;
        self.ui.toolbar = true;
        if active {
            self.set_console("Modes cleared.", "Select a thought or drag the globe.");
        }
        active
    }

    pub fn toggle_multi_select(&mut self) {
        self.multi_select_mode = !self.multi_select_mode;
        self.ui.toolbar = true;
        if self.multi_select_mode {
            self.set_console(
                "Multi-select on.",
                "Click thoughts to add or remove them from the selection.",
            );
        } else {
            self.set_console(
                "Multi-select off.",
                "Clicking a thought selects only that thought.",
            );
        }
    }

    pub fn toggle_link_mode(&mut self) {
        self.link_mode = !self.link_mode;
        self.unlink_mode = false;
        self.unlink_source = None;
        self.unlink_targets.clear();
        self.link_chain.clear();
        self.ui.toolbar = true;
        if self.link_mode {
            self.set_console(
                "Select the first thought to link.",
                "Click thoughts in order; links chain as you go.",
            );
        } else {
            self.set_console("Link mode off.", "Select a thought to inspect it.");
        }
    }

    pub fn link_thought(&mut self, id: &str) {
        if !self.link_mode {
            return;
        }
        let previous = self.link_chain.last().cloned();
        if previous.as_deref() == Some(id) {
            self.set_console(
                "That thought is already the end of the chain.",
                "Select a different thought to link.",
            );
            return;
        }
        let already_selected = self.selected_ids.iter().any(|selected| selected == id);
        let (exists, linked) = {
            let Some(sphere) = self.current_mut() else {
                return;
            };
            let exists = sphere.node(id).is_some();
            let linked = previous
                .as_deref()
                .map(|previous| sphere.add_link(previous, id))
                .unwrap_or(false);
            (exists, linked)
        };
        if !exists {
            return;
        }
        self.link_chain.push(id.to_owned());
        if !already_selected {
            self.selected_ids.push(id.to_owned());
        }
        self.selected_id = Some(id.to_owned());
        self.mark_data();
        self.ui.scene = true;
        self.ui.inspector = true;
        self.ui.toolbar = true;
        let names = {
            let Some(sphere) = self.current() else {
                return;
            };
            self.link_chain
                .iter()
                .filter_map(|thought| sphere.node(thought).map(|node| node.title.as_str()))
                .collect::<Vec<_>>()
                .join(" → ")
        };
        if linked {
            self.set_console("Link created. Select the next thought to link.", &names);
        } else {
            self.set_console("Select the next thought to link.", &names);
        }
    }

    pub fn erase_selected_links(&mut self) {
        let Some(source) = self.selected_id.clone() else {
            self.set_console("Select a thought with links first.", "");
            return;
        };
        let Some(sphere) = self.current() else {
            return;
        };
        let links = sphere
            .links_for(&source)
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if links.is_empty() {
            self.set_console("This thought has no links to erase.", "");
            return;
        }
        if links.len() == 1 {
            let target = links[0].clone();
            if let Some(sphere) = self.current_mut() {
                sphere.remove_link(&source, &target);
            }
            self.mark_data();
            self.ui.scene = true;
            self.ui.inspector = true;
            self.set_console("Link removed.", "Select another thought to continue.");
            return;
        }
        self.link_mode = false;
        self.link_chain.clear();
        self.unlink_mode = true;
        self.unlink_source = Some(source.clone());
        self.unlink_targets = links;
        self.ui.scene = true;
        self.ui.toolbar = true;
        self.set_console(
            "Select the highlighted thought to unlink.",
            "The linked thoughts are glowing on the sphere.",
        );
    }

    pub fn handle_unlink_click(&mut self, id: &str) {
        if !self.unlink_mode {
            return;
        }
        let Some(source) = self.unlink_source.clone() else {
            return;
        };
        if id == source {
            return;
        }
        if !self.unlink_targets.iter().any(|target| target == id) {
            self.set_console(
                "Select one of the highlighted linked thoughts.",
                "Those are the thoughts connected to the selected thought.",
            );
            return;
        }
        if let Some(sphere) = self.current_mut() {
            sphere.remove_link(&source, id);
        }
        self.unlink_mode = false;
        self.unlink_source = None;
        self.unlink_targets.clear();
        self.mark_data();
        self.ui.scene = true;
        self.ui.inspector = true;
        self.ui.toolbar = true;
        self.set_console("Link removed.", "Select a thought to continue.");
    }

    pub fn open_add_to_group_modal(&mut self) {
        let group_id = self
            .current()
            .and_then(|sphere| sphere.groups.first().map(|group| group.id.clone()));
        let Some(group_id) = group_id else {
            self.set_console(
                "Create a group first.",
                "+ New group is below the overview panel.",
            );
            self.toast("Create a group first.", Tone::Amber);
            return;
        };
        self.modal = Some(ModalKind::AddToGroup);
        self.modal_group = group_id;
        self.group_dropdown_open = false;
        self.modal_rev = self.modal_rev.wrapping_add(1);
        self.set_console(
            "Choose a group for the selected thoughts.",
            "The selected thoughts will move together.",
        );
    }

    pub fn add_selected_to_group(&mut self, group_id: &str) {
        if group_id.is_empty() {
            self.set_console("Choose a group first.", "");
            return;
        }
        self.move_selected_to_group(group_id);
        self.set_console("Added to group.", "The group will re-pack automatically.");
    }

    pub fn toggle_group_dropdown(&mut self) {
        self.group_dropdown_open = !self.group_dropdown_open;
        self.modal_rev = self.modal_rev.wrapping_add(1);
    }

    pub fn set_modal_group(&mut self, group_id: &str) {
        self.modal_group = group_id.to_owned();
        self.group_dropdown_open = false;
        self.modal_rev = self.modal_rev.wrapping_add(1);
    }

    pub fn set_grid(&mut self, grid: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        if !matches!(grid, "latlong" | "geodesic" | "off") {
            return;
        }
        if sphere.grid == grid {
            return;
        }
        sphere.grid = grid.to_owned();
        self.mark_dirty();
        self.ui.globe = true;
        self.set_console(
            match grid {
                "latlong" => "Latitude / longitude grid on.",
                "geodesic" => "Geodesic grid on.",
                _ => "Projection lines off.",
            },
            "Change it in Globe settings on the right.",
        );
    }

    pub fn set_sphere_radius(&mut self, radius: f32) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let old = sphere.radius.max(0.1);
        let new = radius.clamp(1.8, 5.0);
        if (old - new).abs() < 0.0001 {
            return;
        }
        sphere.radius = new;
        self.camera_distance = (self.camera_distance * (new / old)).clamp(4.0, 24.0);
        self.mark_data();
        self.ui.scene = true;
        self.ui.globe = true;
        self.sim_active = true;
    }

    pub fn set_sphere_transparency(&mut self, transparency: f32) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let value = transparency.clamp(0.15, 1.0);
        if (sphere.transparency - value).abs() < 0.0001 {
            return;
        }
        sphere.transparency = value;
        self.mark_dirty();
        self.ui.globe = true;
    }

    pub fn set_skin(&mut self, skin: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        if sphere.skin == skin {
            return;
        }
        sphere.skin = skin.to_owned();
        self.mark_dirty();
        self.ui.globe = true;
    }

    pub fn set_background(&mut self, background: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        if sphere.background == background {
            return;
        }
        sphere.background = background.to_owned();
        self.mark_dirty();
        self.ui.globe = true;
    }

    pub fn set_group_color(&mut self, group_id: &str, color: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let Some(group) = sphere.groups.iter_mut().find(|group| group.id == group_id) else {
            return;
        };
        if group.color == color {
            return;
        }
        group.color = color.to_owned();
        self.mark_data();
        self.ui.scene = true;
        self.ui.board = true;
        self.ui.inspector = true;
    }

    pub fn reorder_group(&mut self, from: &str, to: &str) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let Some(from_idx) = sphere.groups.iter().position(|group| group.id == from) else {
            return;
        };
        let Some(to_idx) = sphere.groups.iter().position(|group| group.id == to) else {
            return;
        };
        if from_idx == to_idx {
            return;
        }
        let group = sphere.groups.remove(from_idx);
        sphere.groups.insert(to_idx.min(sphere.groups.len()), group);
        self.mark_data();
        self.ui.board = true;
        self.ui.inspector = true;
    }

    pub fn move_node_board(&mut self, node_id: &str, target_group: &str, after_node: Option<&str>) {
        let Some(sphere) = self.current_mut() else {
            return;
        };
        let Some(node_idx) = sphere.nodes.iter().position(|node| node.id == node_id) else {
            return;
        };
        let mut node = sphere.nodes.remove(node_idx);
        node.group_id = target_group.to_owned();
        let insert_at = after_node
            .and_then(|after| {
                sphere
                    .nodes
                    .iter()
                    .position(|n| n.id == after)
                    .map(|i| i + 1)
            })
            .unwrap_or(sphere.nodes.len());
        sphere.nodes.insert(insert_at.min(sphere.nodes.len()), node);
        self.mark_data();
        self.ui.scene = true;
        self.ui.board = true;
        self.ui.inspector = true;
    }

    pub fn new_sphere(&mut self, title: &str) {
        let sphere = Sphere::blank(title);
        self.current_id = sphere.id.clone();
        self.selected_id = None;
        self.selected_ids.clear();
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
        self.selected_ids.clear();
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
        self.selected_ids.clear();
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
        self.selected_ids.clear();
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
        self.sim_active = true;
    }

    pub fn mark_all(&mut self) {
        self.mark_dirty();
        self.ui.list = true;
        self.ui.inspector = true;
        self.ui.board = true;
        self.ui.scene = true;
        self.ui.globe = true;
        self.ui.toolbar = true;
        self.sim_active = true;
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
        self.group_dropdown_open = false;
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
            ModalKind::NewThought => {
                let group = self.modal_group.clone();
                self.create_thought(&input, &group);
            }
            ModalKind::AddToGroup => {
                let group = self.modal_group.clone();
                self.add_selected_to_group(&group);
            }
            ModalKind::More | ModalKind::Shortcuts | ModalKind::About | ModalKind::Globe => {}
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
    SetSkin(String),
    SetBackground(String),
    SetGroupColor(String, String),
    SetGrid(String),
    ToggleLinkMode,
    ToggleMultiSelect,
    EraseLinks,
    AddSelectedToGroup,
    RemoveSelectedFromGroup,
    ToggleGroupDropdown,
    SetModalGroup(String),
}

#[derive(Message, Clone, Debug)]
pub struct ActionMessage(pub AppAction);

#[derive(Resource, Default)]
pub struct WebImportState {
    #[cfg(target_arch = "wasm32")]
    pub last_nonce: u64,
}
