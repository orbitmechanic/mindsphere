use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "mindsphere.sphere";
pub const VERSION: u32 = 1;
pub const GROUP_COLORS: [&str; 8] = [
    "#5ad2ff", "#ffb454", "#ff7a7a", "#6ee7b7", "#b69cff", "#f38bd0", "#8ed16f", "#e6d36b",
];

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn now_millis() -> u128 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u128
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }
}

pub fn now_secs() -> i64 {
    (now_millis() / 1000) as i64
}

pub fn new_id(prefix: &str) -> String {
    format!(
        "{prefix}_{}_{}",
        now_millis(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn default_format() -> String {
    FORMAT.to_owned()
}

fn default_version() -> u32 {
    VERSION
}

fn default_position() -> [f32; 3] {
    [0.0, 0.0, 1.0]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CameraData {
    #[serde(default)]
    pub yaw: f32,
    #[serde(default = "default_pitch")]
    pub pitch: f32,
    #[serde(default = "default_distance")]
    pub distance: f32,
}

fn default_pitch() -> f32 {
    8.0
}

fn default_distance() -> f32 {
    8.8
}

fn default_radius() -> f32 {
    crate::util::DEFAULT_GLOBE_RADIUS
}

fn default_skin() -> String {
    "earth".to_owned()
}

fn default_transparency() -> f32 {
    1.0
}

fn default_background() -> String {
    "stars".to_owned()
}

fn default_grid() -> String {
    "latlong".to_owned()
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: default_pitch(),
            distance: default_distance(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Group {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thought {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub group_id: String,
    #[serde(default = "default_position")]
    pub pos: [f32; 3],
    #[serde(default)]
    pub created_at: i64,
}

impl Default for Thought {
    fn default() -> Self {
        Self {
            id: new_id("thought"),
            title: "Untitled thought".to_owned(),
            body: String::new(),
            group_id: String::new(),
            pos: default_position(),
            created_at: now_secs(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Link {
    #[serde(default)]
    pub a: String,
    #[serde(default)]
    pub b: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {
    #[serde(default)]
    pub last_sphere: String,
    #[serde(default)]
    pub mode: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sphere {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub camera: CameraData,
    #[serde(default = "default_radius")]
    pub radius: f32,
    #[serde(default = "default_skin")]
    pub skin: String,
    #[serde(default = "default_transparency")]
    pub transparency: f32,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_grid")]
    pub grid: String,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub nodes: Vec<Thought>,
    #[serde(default)]
    pub links: Vec<Link>,
}

impl Default for Sphere {
    fn default() -> Self {
        Self::blank("Untitled sphere")
    }
}

impl Sphere {
    pub fn blank(title: &str) -> Self {
        let now = now_secs();
        let mut sphere = Self {
            format: FORMAT.to_owned(),
            version: VERSION,
            id: new_id("sphere"),
            title: cleaned_title(title),
            created_at: now,
            updated_at: now,
            camera: CameraData::default(),
            radius: default_radius(),
            skin: default_skin(),
            transparency: default_transparency(),
            background: default_background(),
            grid: default_grid(),
            groups: Vec::new(),
            nodes: Vec::new(),
            links: Vec::new(),
        };
        sphere.normalize();
        sphere
    }

    pub fn welcome() -> Self {
        let mut sphere = Self::blank("Thinking it through");
        sphere.camera = CameraData {
            yaw: 0.0,
            pitch: 8.0,
            distance: 8.8,
        };
        let start = sphere.add_group("Start here", Some("#5ad2ff"));
        let ways = sphere.add_group("Ways in", Some("#ffb454"));
        let questions = sphere.add_group("Open questions", Some("#b69cff"));
        let local = sphere.add_group("Your space", Some("#6ee7b7"));

        let a = sphere.add_thought(
            "No centre, no edge",
            &start,
            spherical(5.0, -5.0),
            "A sphere has no fixed middle and no right way up. Any thought can be brought to the centre just by turning the map.",
        );
        let b = sphere.add_thought(
            "Drag a thought",
            &start,
            spherical(11.0, -12.0),
            "Drag any orb across the surface. Ideas are allowed to move when your thinking changes.",
        );
        let c = sphere.add_thought(
            "Group related ideas",
            &ways,
            spherical(30.0, 24.0),
            "Give a cluster a name and a colour. Groups become columns in the 2D board.",
        );
        let d = sphere.add_thought(
            "Connect the dots",
            &ways,
            spherical(36.0, 31.0),
            "Draw a path between thoughts that lead into one another. Connections work across groups.",
        );
        let e = sphere.add_thought(
            "Every angle is valid",
            &questions,
            spherical(-24.0, 2.0),
            "Turn the sphere to see the same set of thoughts from another side. Perspective is part of the map.",
        );
        let f = sphere.add_thought(
            "What will you think through?",
            &questions,
            spherical(-30.0, -5.0),
            "Start with a question that matters. Add everything that comes up before you begin to sort it.",
        );
        let g = sphere.add_thought(
            "Your data stays local",
            &local,
            spherical(43.0, 2.0),
            "Spheres are plain JSON files on your device. No account, no cloud, no telemetry.",
        );
        let h = sphere.add_thought(
            "Export a snapshot",
            &local,
            spherical(48.0, 9.0),
            "Use Export to keep a portable copy of this sphere, or Import to bring one in.",
        );

        sphere.add_link(&a, &b);
        sphere.add_link(&a, &c);
        sphere.add_link(&a, &d);
        sphere.add_link(&c, &e);
        sphere.add_link(&e, &f);
        sphere.add_link(&d, &g);
        sphere.add_link(&g, &h);
        sphere.add_link(&f, &h);
        sphere
    }

    pub fn normalize(&mut self) {
        self.format = FORMAT.to_owned();
        self.version = VERSION;
        if self.id.trim().is_empty() {
            self.id = new_id("sphere");
        }
        self.title = cleaned_title(&self.title);
        if self.created_at == 0 {
            self.created_at = now_secs();
        }
        if self.updated_at == 0 {
            self.updated_at = self.created_at;
        }
        self.camera.pitch = self.camera.pitch.clamp(-1.2, 1.2);
        self.camera.distance = self.camera.distance.clamp(5.2, 24.0);
        self.radius = self.radius.clamp(1.8, 5.0);
        if self.skin.trim().is_empty() {
            self.skin = default_skin();
        }
        self.transparency = self.transparency.clamp(0.15, 1.0);
        if self.background.trim().is_empty() {
            self.background = default_background();
        }
        if !matches!(self.grid.as_str(), "latlong" | "geodesic" | "off") {
            self.grid = default_grid();
        }

        let mut group_ids = HashSet::new();
        for group in &mut self.groups {
            if group.id.trim().is_empty() || !group_ids.insert(group.id.clone()) {
                group.id = new_id("group");
                group_ids.insert(group.id.clone());
            }
            if group.name.trim().is_empty() {
                group.name = "New group".to_owned();
            }
            if group.color.trim().is_empty() {
                group.color = "#8b95b5".to_owned();
            }
        }

        let mut node_ids = HashSet::new();
        for node in &mut self.nodes {
            if node.id.trim().is_empty() || !node_ids.insert(node.id.clone()) {
                node.id = new_id("thought");
                node_ids.insert(node.id.clone());
            }
            if node.title.trim().is_empty() {
                node.title = "Untitled thought".to_owned();
            }
            if !group_ids.contains(&node.group_id) {
                node.group_id.clear();
            }
            let p = Vec3Like::from(node.pos);
            node.pos = p.normalized().to_array();
            if node.created_at == 0 {
                node.created_at = now_secs();
            }
        }

        self.links.retain(|link| {
            link.a != link.b && node_ids.contains(&link.a) && node_ids.contains(&link.b)
        });
        let mut seen = HashSet::new();
        self.links.retain(|link| {
            let key = if link.a < link.b {
                format!("{}:{}", link.a, link.b)
            } else {
                format!("{}:{}", link.b, link.a)
            };
            seen.insert(key)
        });
    }

    pub fn add_group(&mut self, name: &str, color: Option<&str>) -> String {
        let id = new_id("group");
        let color = color
            .map(str::to_owned)
            .unwrap_or_else(|| GROUP_COLORS[self.groups.len() % GROUP_COLORS.len()].to_owned());
        self.groups.push(Group {
            id: id.clone(),
            name: cleaned_title(name),
            color,
        });
        id
    }

    pub fn add_thought(
        &mut self,
        title: &str,
        group_id: &str,
        pos: [f32; 3],
        body: &str,
    ) -> String {
        let id = new_id("thought");
        self.nodes.push(Thought {
            id: id.clone(),
            title: if title.trim().is_empty() {
                "Untitled thought".to_owned()
            } else {
                title.trim().to_owned()
            },
            body: body.to_owned(),
            group_id: group_id.to_owned(),
            pos: Vec3Like::from(pos).normalized().to_array(),
            created_at: now_secs(),
        });
        id
    }

    pub fn add_link(&mut self, a: &str, b: &str) -> bool {
        if a.is_empty() || b.is_empty() || a == b {
            return false;
        }
        if self
            .links
            .iter()
            .any(|link| (link.a == a && link.b == b) || (link.a == b && link.b == a))
        {
            return false;
        }
        self.links.push(Link {
            a: a.to_owned(),
            b: b.to_owned(),
        });
        true
    }

    pub fn remove_link(&mut self, a: &str, b: &str) -> bool {
        let before = self.links.len();
        self.links
            .retain(|link| !((link.a == a && link.b == b) || (link.a == b && link.b == a)));
        self.links.len() != before
    }

    pub fn remove_thought(&mut self, id: &str) {
        self.nodes.retain(|node| node.id != id);
        self.links.retain(|link| link.a != id && link.b != id);
    }

    pub fn node(&self, id: &str) -> Option<&Thought> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut Thought> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn group(&self, id: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.id == id)
    }

    pub fn group_name(&self, id: &str) -> &str {
        self.group(id)
            .map(|group| group.name.as_str())
            .unwrap_or("Unfiled")
    }

    pub fn group_color(&self, id: &str) -> &str {
        self.group(id)
            .map(|group| group.color.as_str())
            .unwrap_or("#8b95b5")
    }

    pub fn links_for(&self, id: &str) -> Vec<&str> {
        self.links
            .iter()
            .filter_map(|link| {
                if link.a == id {
                    Some(link.b.as_str())
                } else if link.b == id {
                    Some(link.a.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn duplicate(&self) -> Self {
        let mut copy = self.clone();
        copy.id = new_id("sphere");
        copy.title = format!("{} copy", self.title);
        copy.created_at = now_secs();
        copy.updated_at = copy.created_at;
        copy.normalize();
        copy
    }
}

pub fn parse_sphere(text: &str) -> Option<Sphere> {
    let mut sphere: Sphere = serde_json::from_str(text).ok()?;
    sphere.id = new_id("sphere");
    sphere.created_at = now_secs();
    sphere.updated_at = sphere.created_at;
    sphere.normalize();
    Some(sphere)
}

pub fn parse_library(text: &str) -> Vec<Sphere> {
    if let Ok(mut spheres) = serde_json::from_str::<Vec<Sphere>>(text) {
        for sphere in &mut spheres {
            sphere.normalize();
        }
        return spheres;
    }
    if let Ok(map) = serde_json::from_str::<BTreeMap<String, Sphere>>(text) {
        let mut spheres: Vec<Sphere> = map.into_values().collect();
        for sphere in &mut spheres {
            sphere.normalize();
        }
        return spheres;
    }
    parse_sphere(text).into_iter().collect()
}

pub fn library_json(spheres: &[Sphere]) -> String {
    serde_json::to_string_pretty(spheres).unwrap_or_else(|_| "[]".to_owned())
}

pub fn sphere_json(sphere: &Sphere) -> String {
    serde_json::to_string_pretty(sphere).unwrap_or_else(|_| "{}".to_owned())
}

pub fn spherical(lat_deg: f32, lon_deg: f32) -> [f32; 3] {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    [lat.cos() * lon.sin(), lat.sin(), (lat.cos() * lon.cos())]
}

fn cleaned_title(title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        "Untitled sphere".to_owned()
    } else {
        title.to_owned()
    }
}

#[derive(Clone, Copy, Debug)]
struct Vec3Like {
    x: f32,
    y: f32,
    z: f32,
}

impl From<[f32; 3]> for Vec3Like {
    fn from(value: [f32; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl Vec3Like {
    fn normalized(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len < 0.0001 || !len.is_finite() {
            Self {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            }
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        }
    }

    fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_is_connected_and_normalized() {
        let sphere = Sphere::welcome();
        assert_eq!(sphere.groups.len(), 4);
        assert_eq!(sphere.nodes.len(), 8);
        assert_eq!(sphere.links.len(), 8);
        for node in &sphere.nodes {
            let p = Vec3Like::from(node.pos);
            let length = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn links_are_unique_and_removed_with_nodes() {
        let mut sphere = Sphere::welcome();
        let a = sphere.nodes[0].id.clone();
        let b = sphere.nodes[1].id.clone();
        assert!(!sphere.add_link(&a, &b));
        sphere.remove_thought(&a);
        assert!(sphere.node(&a).is_none());
        assert!(sphere.links.iter().all(|link| link.a != a && link.b != a));
    }

    #[test]
    fn duplicate_gets_a_new_id() {
        let sphere = Sphere::welcome();
        let copy = sphere.duplicate();
        assert_ne!(sphere.id, copy.id);
        assert_eq!(copy.nodes.len(), sphere.nodes.len());
    }

    #[test]
    fn visual_settings_round_trip() {
        let mut sphere = Sphere::welcome();
        sphere.radius = 3.4;
        sphere.skin = "jupiter".to_owned();
        sphere.transparency = 0.55;
        sphere.background = "skymap".to_owned();
        let parsed = parse_sphere(&sphere_json(&sphere)).expect("valid sphere");
        assert!((parsed.radius - 3.4).abs() < 0.001);
        assert_eq!(parsed.skin, "jupiter");
        assert!((parsed.transparency - 0.55).abs() < 0.001);
        assert_eq!(parsed.background, "skymap");
    }

    #[test]
    fn visual_settings_clamp() {
        let mut sphere = Sphere::welcome();
        sphere.radius = 99.0;
        sphere.transparency = -1.0;
        sphere.normalize();
        assert_eq!(sphere.radius, 5.0);
        assert_eq!(sphere.transparency, 0.15);
    }

    #[test]
    fn library_round_trips() {
        let spheres = vec![Sphere::welcome()];
        let text = library_json(&spheres);
        let parsed = parse_library(&text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].title, "Thinking it through");
    }
}
