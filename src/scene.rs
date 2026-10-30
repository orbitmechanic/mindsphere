use std::collections::HashMap;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::skins::SceneTextures;
use crate::state::{AppState, ViewMode};
use crate::util::{
    BOTTOM_DOCK_H, GROUP_ANGULAR_RADIUS, LEFT_PANEL_W, NODE_LIFT, NODE_RADIUS, RIGHT_PANEL_W,
    TOP_BAR_H, alpha, color_from_hex,
};

#[derive(Component)]
pub struct OrbitCamera;

#[derive(Component)]
pub struct Globe;

#[derive(Component)]
pub struct SkySphere;

#[derive(Component)]
pub struct StarfieldRoot;

#[derive(Component)]
pub struct ThoughtVisual;

#[derive(Resource)]
pub struct SceneAssets {
    pub node_mesh: Handle<Mesh>,
}

#[derive(Resource)]
pub struct GlobeHandles {
    pub globe_material: Handle<StandardMaterial>,
    pub sky_material: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct SceneMap {
    pub nodes: HashMap<String, Entity>,
}

#[derive(Resource, Default)]
pub struct DragState {
    pub orbiting: bool,
    pub node: Option<String>,
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let textures = SceneTextures::load(&asset_server);
    let earth = textures.skin("earth");
    let stars = textures.background("stars");
    let globe_mesh = meshes.add(Sphere::new(1.0).mesh().uv(96, 48));
    let globe_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.94, 0.96, 0.99, 1.0),
        base_color_texture: Some(earth),
        metallic: 0.02,
        perceptual_roughness: 0.82,
        emissive: Color::srgb(0.006, 0.012, 0.018).to_linear(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(globe_mesh),
        MeshMaterial3d(globe_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Globe,
    ));

    let sky_mesh = meshes.add(Sphere::new(1.0).mesh().uv(64, 32));
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(stars),
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        ..default()
    });
    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(60.0)),
        NoFrustumCulling,
        SkySphere,
    ));

    let star_mesh = meshes.add(Sphere::new(1.0).mesh().ico(2).expect("star sphere"));
    let star_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.97, 1.0),
            unlit: true,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.76, 1.0),
            unlit: true,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.84, 0.62),
            unlit: true,
            ..default()
        }),
    ];
    let mut rng = SimpleRng::new(0x51A7_2026);
    commands
        .spawn((StarfieldRoot, Transform::default(), Visibility::Inherited))
        .with_children(|parent| {
            for _ in 0..460 {
                let z = rng.range(-1.0, 1.0);
                let angle = rng.range(0.0, std::f32::consts::TAU);
                let r = (1.0 - z * z).sqrt();
                let direction = Vec3::new(r * angle.cos(), z, r * angle.sin());
                let material =
                    star_materials[rng.next_u32() as usize % star_materials.len()].clone();
                parent.spawn((
                    Mesh3d(star_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(direction * 45.0)
                        .with_scale(Vec3::splat(rng.range(0.045, 0.13))),
                ));
            }
        });

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.2, 8.8).looking_at(Vec3::ZERO, Vec3::Y),
        AmbientLight {
            color: Color::srgb(0.55, 0.68, 0.88),
            brightness: 110.0,
            ..default()
        },
        IsDefaultUiCamera,
        OrbitCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4_200.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 7.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.82, 0.95),
            intensity: 38_000.0,
            range: 14.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.5, 2.0, 5.0),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.45, 0.25, 0.95),
            intensity: 22_000.0,
            range: 12.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, -2.0, -4.0),
    ));

    commands.insert_resource(SceneAssets {
        node_mesh: meshes.add(Sphere::new(NODE_RADIUS).mesh().ico(4).expect("node sphere")),
    });
    commands.insert_resource(GlobeHandles {
        globe_material,
        sky_material,
    });
    commands.insert_resource(textures);
}

pub fn sync_globe(
    mut state: ResMut<AppState>,
    textures: Res<SceneTextures>,
    handles: Res<GlobeHandles>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut globe: Query<&mut Transform, With<Globe>>,
) {
    if !state.ui.globe {
        return;
    }
    let Some(sphere) = state.current().cloned() else {
        return;
    };
    state.ui.globe = false;
    let transparency = sphere.transparency.clamp(0.15, 1.0);
    let skin = textures.skin(&sphere.skin);
    let sky = textures.background(&sphere.background);
    if let Some(mut material) = materials.get_mut(&handles.globe_material) {
        material.base_color_texture = Some(skin);
        material.base_color = Color::srgba(0.94, 0.96, 0.99, transparency);
        material.alpha_mode = if transparency < 0.999 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        };
    }
    if let Some(mut material) = materials.get_mut(&handles.sky_material) {
        material.base_color_texture = Some(sky);
    }
    for mut transform in &mut globe {
        transform.scale = Vec3::splat(sphere.radius);
    }
}

pub fn update_camera(state: Res<AppState>, mut camera: Query<&mut Transform, With<OrbitCamera>>) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let cos_pitch = state.camera_pitch.cos();
    let offset = Vec3::new(
        state.camera_yaw.sin() * cos_pitch,
        state.camera_pitch.sin(),
        state.camera_yaw.cos() * cos_pitch,
    ) * state.camera_distance;
    transform.translation = offset;
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

pub fn camera_controls(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut state: ResMut<AppState>,
    mut drag: ResMut<DragState>,
) {
    let delta = motion
        .read()
        .fold(Vec2::ZERO, |acc, event| acc + event.delta);
    let wheel_delta = wheel
        .read()
        .fold(0.0, |acc, event| acc + event.y * wheel_scale(event.unit));

    if state.modal.is_some() || state.mode == ViewMode::Board {
        drag.orbiting = false;
        drag.node = None;
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let width = window.resolution.width();
    let height = window.resolution.height();
    let over_scene = cursor.x > LEFT_PANEL_W
        && cursor.x < width - RIGHT_PANEL_W
        && cursor.y > TOP_BAR_H
        && cursor.y < height - BOTTOM_DOCK_H;

    if over_scene && wheel_delta.abs() > 0.0 {
        state.camera_distance = (state.camera_distance - wheel_delta * 0.12).clamp(4.0, 24.0);
        state.mark_dirty();
    }

    if buttons.just_pressed(MouseButton::Left) && over_scene {
        let picked = pick_node(&state, &camera, cursor);
        if let Some(id) = picked {
            state.select_node(&id);
            drag.node = Some(id);
            drag.orbiting = false;
        } else {
            drag.orbiting = true;
            drag.node = None;
            state.clear_selection();
        }
    }

    if buttons.just_released(MouseButton::Left) {
        drag.orbiting = false;
        drag.node = None;
    }

    if drag.node.is_some() && buttons.pressed(MouseButton::Left) {
        let Some(id) = drag.node.clone() else {
            return;
        };
        if let Ok((camera, camera_transform)) = camera.single()
            && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor)
        {
            let origin = ray.origin;
            let direction = *ray.direction;
            let b = origin.dot(direction);
            let drag_radius = state
                .current()
                .map(|sphere| sphere.radius + NODE_LIFT)
                .unwrap_or(2.9);
            let c = origin.length_squared() - drag_radius * drag_radius;
            let discriminant = b * b - c;
            if discriminant >= 0.0 {
                let t = -b - discriminant.sqrt();
                if t > 0.0 {
                    state.set_node_position(&id, ray.get_point(t));
                }
            }
        }
    } else if drag.orbiting && buttons.pressed(MouseButton::Left) {
        state.camera_yaw -= delta.x * 0.006;
        state.camera_pitch = (state.camera_pitch + delta.y * 0.006).clamp(-1.15, 1.15);
        state.mark_dirty();
    }
}

fn wheel_scale(unit: bevy::input::mouse::MouseScrollUnit) -> f32 {
    match unit {
        bevy::input::mouse::MouseScrollUnit::Line => 1.0,
        bevy::input::mouse::MouseScrollUnit::Pixel => 0.05,
    }
}

fn pick_node(
    state: &AppState,
    camera: &Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    cursor: Vec2,
) -> Option<String> {
    let Ok((camera, camera_transform)) = camera.single() else {
        return None;
    };
    let sphere = state.current()?;
    let radius = sphere.radius + NODE_LIFT;
    let mut best: Option<(f32, String)> = None;
    for node in &sphere.nodes {
        let world = Vec3::from_array(node.pos) * radius;
        let Ok(screen) = camera.world_to_viewport(camera_transform, world) else {
            continue;
        };
        let distance = screen.distance(cursor);
        if distance < 18.0 && best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((distance, node.id.clone()));
        }
    }
    best.map(|(_, id)| id)
}

pub fn sync_scene(
    mut commands: Commands,
    mut state: ResMut<AppState>,
    assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<SceneMap>,
    mut visuals: Query<
        (
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        With<ThoughtVisual>,
    >,
) {
    if !state.ui.scene {
        return;
    }
    let Some(sphere) = state.current().cloned() else {
        return;
    };
    state.ui.scene = false;
    let selected = state.selected_id.clone();

    let live_ids: std::collections::HashSet<String> =
        sphere.nodes.iter().map(|node| node.id.clone()).collect();
    let stale: Vec<String> = map
        .nodes
        .keys()
        .filter(|id| !live_ids.contains(*id))
        .cloned()
        .collect();
    for id in stale {
        if let Some(entity) = map.nodes.remove(&id) {
            commands.entity(entity).despawn();
        }
    }

    for node in &sphere.nodes {
        let color = color_from_hex(sphere.group_color(&node.group_id));
        let is_selected = selected.as_deref() == Some(node.id.as_str());
        let emissive = if is_selected {
            color.to_linear() * 0.95
        } else {
            color.to_linear() * 0.28
        };
        let position = Vec3::from_array(node.pos) * (sphere.radius + NODE_LIFT);

        if let Some(&entity) = map.nodes.get(&node.id) {
            if let Ok((mut transform, material_handle, mut visibility)) = visuals.get_mut(entity) {
                transform.translation = position;
                *visibility = Visibility::Inherited;
                if let Some(mut material) = materials.get_mut(&material_handle.0) {
                    material.base_color = if is_selected {
                        lighten(color, 0.18)
                    } else {
                        color
                    };
                    material.emissive = emissive;
                }
            }
        } else {
            let material = materials.add(StandardMaterial {
                base_color: color,
                emissive,
                metallic: 0.1,
                perceptual_roughness: 0.28,
                ..default()
            });
            let entity = commands
                .spawn((
                    Mesh3d(assets.node_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(position),
                    ThoughtVisual,
                    Visibility::Inherited,
                ))
                .id();
            map.nodes.insert(node.id.clone(), entity);
        }
    }
}

pub fn draw_globe(mut gizmos: Gizmos, state: Res<AppState>) {
    if state.mode == ViewMode::Board {
        return;
    }
    let grid = Color::srgba(0.32, 0.62, 0.88, 0.15);
    let equator = Color::srgba(0.44, 0.82, 1.0, 0.22);
    let Some(sphere) = state.current() else {
        return;
    };
    let radius = sphere.radius + 0.012;

    for lon in (0..12).map(|i| i as f32 * 30.0) {
        let mut points = Vec::with_capacity(65);
        for step in 0..=64 {
            let lat = -90.0 + step as f32 * (180.0 / 64.0);
            points.push(spherical_vec(lat, lon) * radius);
        }
        gizmos.linestrip(points, grid);
    }
    for lat in [-60.0, -30.0, 30.0, 60.0] {
        let mut points = Vec::with_capacity(65);
        for step in 0..=64 {
            let lon = step as f32 * (360.0 / 64.0);
            points.push(spherical_vec(lat, lon) * radius);
        }
        gizmos.linestrip(points, grid);
    }
    {
        let mut points = Vec::with_capacity(65);
        for step in 0..=64 {
            let lon = step as f32 * (360.0 / 64.0);
            points.push(spherical_vec(0.0, lon) * radius);
        }
        gizmos.linestrip(points, equator);
    }

    for link in &sphere.links {
        let (Some(a), Some(b)) = (sphere.node(&link.a), sphere.node(&link.b)) else {
            continue;
        };
        let a = Vec3::from_array(a.pos);
        let b = Vec3::from_array(b.pos);
        let selected = state.selected_id.as_deref();
        let lit = selected == Some(link.a.as_str()) || selected == Some(link.b.as_str());
        let color = if lit {
            Color::srgba(0.43, 0.94, 1.0, 0.82)
        } else {
            Color::srgba(0.35, 0.70, 0.92, 0.34)
        };
        let mut points = Vec::with_capacity(25);
        for step in 0..=24 {
            let t = step as f32 / 24.0;
            let point = a.lerp(b, t).normalize_or_zero() * (sphere.radius + 0.075);
            points.push(point);
        }
        gizmos.linestrip(points, color);
    }

    for group in &sphere.groups {
        let mut nodes: Vec<Vec3> = sphere
            .nodes
            .iter()
            .filter(|node| node.group_id == group.id)
            .map(|node| Vec3::from_array(node.pos).normalize_or_zero())
            .filter(|position| position.length_squared() > 0.5)
            .collect();
        if nodes.is_empty() {
            continue;
        }
        nodes.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        let theta = GROUP_ANGULAR_RADIUS;
        let inside_cos = (theta - 0.0035).cos();
        let outline_radius = sphere.radius + 0.082;
        let base = color_from_hex(&group.color);
        let outline_color = alpha(base, 0.72);
        let samples = 96usize;
        let circles: Vec<Vec<Vec3>> = nodes
            .iter()
            .map(|center| circle_on_sphere(*center, theta, samples))
            .collect();
        for (index, circle) in circles.iter().enumerate() {
            let keep: Vec<bool> = circle
                .iter()
                .map(|point| {
                    !circles.iter().enumerate().any(|(other_index, _)| {
                        other_index != index && point.dot(nodes[other_index]) > inside_cos
                    })
                })
                .collect();
            for step in 0..samples {
                let next = (step + 1) % samples;
                if keep[step] && keep[next] {
                    gizmos.line(
                        circle[step] * outline_radius,
                        circle[next] * outline_radius,
                        outline_color,
                    );
                }
            }
        }
    }

    if let Some(id) = state.selected_id.as_deref()
        && let Some(node) = sphere.node(id)
    {
        let center = Vec3::from_array(node.pos);
        let rotation = Quat::from_rotation_arc(Vec3::Z, center);
        gizmos
            .circle(
                Isometry3d::new(center * (sphere.radius + NODE_LIFT), rotation),
                NODE_RADIUS * 1.9,
                Color::srgba(0.55, 0.98, 1.0, 0.8),
            )
            .resolution(40);
    }
}

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 & 0xffff_ffff) as u32
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        let value = self.next_u32() as f32 / u32::MAX as f32;
        min + (max - min) * value
    }
}

fn circle_on_sphere(center: Vec3, angular_radius: f32, samples: usize) -> Vec<Vec3> {
    let reference = if center.y.abs() < 0.95 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent = reference.cross(center).normalize_or_zero();
    if tangent.length_squared() < 0.5 {
        return Vec::new();
    }
    let bitangent = center.cross(tangent).normalize_or_zero();
    let sin_r = angular_radius.sin();
    let cos_r = angular_radius.cos();
    (0..samples)
        .map(|index| {
            let angle = std::f32::consts::TAU * index as f32 / samples as f32;
            (center * cos_r + (tangent * angle.cos() + bitangent * angle.sin()) * sin_r)
                .normalize_or_zero()
        })
        .collect()
}

fn spherical_vec(lat_deg: f32, lon_deg: f32) -> Vec3 {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    Vec3::new(lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos())
}

fn lighten(color: Color, amount: f32) -> Color {
    let c = color.to_srgba();
    Color::srgba(
        (c.red + (1.0 - c.red) * amount).min(1.0),
        (c.green + (1.0 - c.green) * amount).min(1.0),
        (c.blue + (1.0 - c.blue) * amount).min(1.0),
        c.alpha,
    )
}
