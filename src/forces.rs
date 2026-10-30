use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::model::Sphere;
use crate::scene::DragState;
use crate::state::AppState;
use crate::util::{GROUP_ANGULAR_RADIUS, MIN_THOUGHT_DIST};

#[derive(Resource, Default)]
pub struct ForceState {
    velocities: HashMap<String, Vec3>,
    quiet_steps: u32,
    current_sphere: String,
}

pub fn simulate(
    time: Res<Time<Fixed>>,
    mut state: ResMut<AppState>,
    mut force: ResMut<ForceState>,
    drag: Res<DragState>,
) {
    if !state.sim_active {
        return;
    }
    let Some(mut sphere) = state.current().cloned() else {
        return;
    };
    if force.current_sphere != sphere.id {
        force.velocities.clear();
        force.quiet_steps = 0;
        force.current_sphere = sphere.id.clone();
    }
    if sphere.nodes.is_empty() {
        return;
    }

    let dt = time.delta_secs().clamp(0.001, 1.0 / 30.0) / 6.0;
    let mut max_step = 0.0_f32;
    for _ in 0..6 {
        max_step = max_step.max(step(&mut sphere, &mut force, drag.node.as_deref(), dt));
    }

    if max_step > 0.00012 {
        if let Some(current) = state.current_mut() {
            current.nodes = sphere.nodes;
        }
        force.quiet_steps = 0;
        state.sim_active = true;
        state.ui.scene = true;
        if !state.dirty {
            state.save_timer = 1.4;
        }
        state.dirty = true;
    } else {
        force.quiet_steps += 1;
        if force.quiet_steps > 45 {
            state.sim_active = false;
            state.dirty = true;
            state.save_timer = 0.4;
        }
    }
}

fn step(sphere: &mut Sphere, force: &mut ForceState, dragged: Option<&str>, dt: f32) -> f32 {
    let radius = sphere.radius.max(0.5);
    let count = sphere.nodes.len();
    let mut positions: Vec<Vec3> = sphere
        .nodes
        .iter()
        .map(|node| Vec3::from_array(node.pos).normalize_or_zero())
        .collect();
    for position in &mut positions {
        if position.length_squared() < 0.5 {
            *position = Vec3::Z;
        }
    }
    let mut acceleration = vec![Vec3::ZERO; count];

    // Thought-to-thought repulsion: keep centres at least two thought diameters apart.
    for i in 0..count {
        for j in (i + 1)..count {
            let angle = positions[i].dot(positions[j]).clamp(-1.0, 1.0).acos();
            let world_distance = angle * radius;
            if world_distance >= MIN_THOUGHT_DIST {
                continue;
            }
            let Some(direction) = tangent_towards(positions[i], positions[j]) else {
                continue;
            };
            let strength = (MIN_THOUGHT_DIST - world_distance) / radius * 20.0 + 0.12;
            acceleration[i] -= direction * strength;
            acceleration[j] += direction * strength;
        }
    }

    // Groups: attract members to their surface centroid.
    let mut members: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, node) in sphere.nodes.iter().enumerate() {
        members
            .entry(node.group_id.clone())
            .or_default()
            .push(index);
    }
    let mut groups: Vec<(String, Vec3, f32, Vec<usize>)> = Vec::new();
    for (group_id, indices) in &members {
        let centroid = indices
            .iter()
            .fold(Vec3::ZERO, |sum, index| sum + positions[*index]);
        let center = centroid.normalize_or_zero();
        if center.length_squared() < 0.5 {
            continue;
        }
        let mut extent: f32 = 0.0;
        for index in indices {
            let angle = center.dot(positions[*index]).clamp(-1.0, 1.0).acos();
            extent = extent.max(angle);
        }
        extent += GROUP_ANGULAR_RADIUS;
        for index in indices {
            let Some(direction) = tangent_towards(positions[*index], center) else {
                continue;
            };
            let angle = positions[*index].dot(center).clamp(-1.0, 1.0).acos();
            acceleration[*index] += direction * (angle * 4.5).min(2.5);
        }
        groups.push((group_id.clone(), center, extent, indices.clone()));
    }

    // Group-to-group repulsion: keep the circle unions from overlapping.
    let group_margin = 0.04;
    for i in 0..groups.len() {
        for j in (i + 1)..groups.len() {
            let (_, center_a, extent_a, members_a) = &groups[i];
            let (_, center_b, extent_b, members_b) = &groups[j];
            let angle = center_a.dot(*center_b).clamp(-1.0, 1.0).acos();
            let minimum = extent_a + extent_b + group_margin;
            if angle >= minimum {
                continue;
            }
            let Some(direction) = tangent_towards(*center_a, *center_b) else {
                continue;
            };
            let strength = (minimum - angle) * 13.0;
            for index in members_a {
                acceleration[*index] -= direction * strength;
            }
            for index in members_b {
                acceleration[*index] += direction * strength;
            }
        }
    }

    let mut max_step = 0.0_f32;
    let mut active_ids = HashSet::new();
    for (index, node) in sphere.nodes.iter().enumerate() {
        active_ids.insert(node.id.clone());
        if dragged == Some(node.id.as_str()) {
            force.velocities.remove(&node.id);
            continue;
        }
        let mut velocity = force
            .velocities
            .get(&node.id)
            .copied()
            .unwrap_or(Vec3::ZERO);
        velocity += acceleration[index] * dt;
        if velocity.length() > 0.9 {
            velocity = velocity.normalize() * 0.9;
        }
        velocity *= 0.55;
        if velocity.length() < 0.0006 {
            velocity = Vec3::ZERO;
        }
        let mut step = velocity * dt;
        if step.length() > 0.018 {
            step = step.normalize() * 0.018;
        }
        let next = (positions[index] + step).normalize_or_zero();
        if next.length_squared() > 0.5 {
            max_step = max_step.max(next.distance(positions[index]));
            positions[index] = next;
        }
        force.velocities.insert(node.id.clone(), velocity);
    }
    force.velocities.retain(|id, _| active_ids.contains(id));

    for (node, position) in sphere.nodes.iter_mut().zip(positions) {
        node.pos = [position.x, position.y, position.z];
    }
    max_step
}

fn tangent_towards(from: Vec3, to: Vec3) -> Option<Vec3> {
    let dot = from.dot(to).clamp(-1.0, 1.0);
    let tangent = to - from * dot;
    if tangent.length_squared() < 0.0000001 {
        None
    } else {
        Some(tangent.normalize())
    }
}
