use crate::startup::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_spawn_system, enemy_movement_system));
    }
}

#[derive(Component)]
pub struct EnemyMarker;

#[derive(Component, Debug)]
pub struct EnemyPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource)]
pub struct EnemyState(pub HashMap<Entity, EnemyPos>);

pub fn eliminate_enemy(
    commands: &mut Commands,
    enemy: Entity,
    enemy_state: &mut ResMut<EnemyState>,
) {
    let Some((enemy, _)) = enemy_state.0.remove_entry(&enemy) else {
        return;
    };
    enemy_state.0.remove(&enemy);
    commands.entity(enemy).despawn_recursive();
}

// make them strafe to make them appear they're dodging
pub fn enemy_movement_system(
    mut enem_pos: Query<&mut Transform, With<EnemyMarker>>,
    time: Res<Time>,
) {
    for mut pos in enem_pos.iter_mut() {
        pos.translation.z += 10. * time.delta_seconds();
    }
}

pub fn enemy_spawn_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut enemy_state: ResMut<EnemyState>,
) {
    if enemy_state.0.len() >= ENEMY_SPAWN_LIMIT {
        return;
    }

    let mut unique_pos: HashSet<(i32, i32)> = enemy_state
        .0
        .values()
        .map(|enemy| (enemy.x, enemy.y))
        .collect();

    let mut rng = thread_rng();
    while unique_pos.len() != ENEMY_SPAWN_LIMIT {
        let (x, y) = (rng.gen_range(-5..=5), rng.gen_range(3..=6));
        unique_pos.insert((x, y));
    }

    for (x, y) in unique_pos {
        if enemy_state
            .0
            .iter()
            .any(|(_, enemy)| enemy.x == x && enemy.y == y)
        {
            continue;
        }

        let sphere = Sphere { radius: 1. };
        let sphere_bundle = MaterialMeshBundle {
            mesh: mesh.add(sphere),
            transform: Transform::from_xyz(x as f32, y as f32, -50.),
            material: material.add(Color::WHITE),
            ..default()
        };

        let enemy_id = commands
            .spawn((EnemyMarker, sphere_bundle))
            .insert(Sensor)
            .insert(Collider::ball(1.))
            .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1))
            .id();
        enemy_state.0.insert(enemy_id, EnemyPos { x, y });
    }
}

pub fn reset_system(
    mut commands: Commands,
    enemies: Query<Entity, With<EnemyMarker>>,
    mut enemy_state: ResMut<EnemyState>,
) {
    for enemy in &enemies {
        eliminate_enemy(&mut commands, enemy, &mut enemy_state);
    }
}
