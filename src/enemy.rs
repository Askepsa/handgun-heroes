use crate::player::KillCount;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

pub const ENEMY_SPAWN_LIMIT: usize = 5;
const KILL_LEVELS: [usize; ENEMY_SPAWN_LIMIT - 1] = [5, 10, 50, 100];

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyState {
            pos: HashMap::new(),
            enemy_count: 1,
            enemy_count_updated: false,
        })
        .add_systems(Update, (enemy_spawn_system, enemy_movement_system));
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
pub struct EnemyState {
    pub pos: HashMap<Entity, EnemyPos>,
    pub enemy_count: usize,
    pub enemy_count_updated: bool,
}

pub fn eliminate_enemy(
    commands: &mut Commands,
    enemy_entity: Entity,
    enemy_state: &mut ResMut<EnemyState>,
) {
    let Some((enemy, _)) = enemy_state.pos.remove_entry(&enemy_entity) else {
        return;
    };
    enemy_state.pos.remove(&enemy);
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
    player_kill_count: Res<KillCount>,
) {
    let mut unique_pos: HashSet<(i32, i32)> = enemy_state
        .pos
        .values()
        .map(|enemy| (enemy.x, enemy.y))
        .collect();

    let is_kill_count_level = KILL_LEVELS.contains(&player_kill_count.0);
    let can_update_enemy_count =
        enemy_state.enemy_count <= ENEMY_SPAWN_LIMIT && !enemy_state.enemy_count_updated;

    if !is_kill_count_level && enemy_state.enemy_count_updated {
        enemy_state.enemy_count_updated = false;
    }

    if is_kill_count_level && can_update_enemy_count {
        enemy_state.enemy_count += 1;
        enemy_state.enemy_count_updated = true;

        println!("+1 {}", enemy_state.enemy_count);
        println!("{}", enemy_state.enemy_count);
    }

    let mut rng = thread_rng();
    while unique_pos.len() < enemy_state.enemy_count {
        let (x, y) = (rng.gen_range(-9..=9), rng.gen_range(3..=6));
        unique_pos.insert((x, y));
    }

    for (x, y) in unique_pos {
        if enemy_state
            .pos
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
        enemy_state.pos.insert(enemy_id, EnemyPos { x, y });
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
