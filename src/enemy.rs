use crate::{player::KillCount, startup::Kulay};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

pub const ENEMY_SPAWN_LIMIT: usize = 10;
const KILL_LEVELS: [usize; ENEMY_SPAWN_LIMIT - 1] = [5, 10, 50, 100, 150, 250, 300, 350, 400];

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

#[derive(Bundle, Debug)]
pub struct EnemyBundle {
    pub color: Kulay,
    pub marker: Enemy,
    pub ms: MovementSpeed,
}

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Debug)]
pub struct EnemyPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug)]
pub struct MovementSpeed(pub f32);

#[derive(Resource)]
pub struct EnemyState {
    pub pos: HashMap<Entity, EnemyPos>,
    pub enemy_count: usize,
    pub enemy_count_updated: bool,
}

impl EnemyBundle {
    fn new(color: Kulay, ms: f32) -> Self {
        Self {
            color,
            marker: Enemy,
            ms: MovementSpeed(ms),
        }
    }
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
fn enemy_movement_system(
    mut enemies: Query<(&mut Transform, &MovementSpeed), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut pos, ms) in enemies.iter_mut() {
        pos.translation.z += ms.0 * time.delta_seconds();
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
    }

    println!("{}", enemy_state.enemy_count);

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

        // roll the dice
        let color = if rng.gen_range(0..=1) == 0 {
            Kulay::Pula
        } else {
            Kulay::Asul
        };

        let sphere = Sphere { radius: 1. };
        let sphere_bundle = MaterialMeshBundle {
            mesh: mesh.add(sphere),
            transform: Transform::from_xyz(x as f32, y as f32, -50.),
            material: match color {
                Kulay::Pula => material.add(StandardMaterial {
                    base_color: Color::hsl(0., 0.5, 0.5),
                    reflectance: 0.,
                    ..default()
                }),
                Kulay::Asul => material.add(StandardMaterial {
                    base_color: Color::hsl(240., 0.8, 0.5),
                    reflectance: 0.,
                    ..default()
                }),
            },
            ..default()
        };

        let enemy_id = commands
            .spawn((
                EnemyBundle::new(color, pick_ms(enemy_state.enemy_count)),
                sphere_bundle,
            ))
            .insert(Sensor)
            .insert(Collider::ball(1.))
            .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1))
            .id();
        enemy_state.pos.insert(enemy_id, EnemyPos { x, y });
    }
}

fn pick_ms(enemy_count: usize) -> f32 {
    let mut rng = thread_rng();
    let ms_threshold = match enemy_count {
        5 | 6 => 10,
        7 | 8 => rng.gen_range(10..11),
        9 | 10 => rng.gen_range(10..=12),
        _ => rng.gen_range(7..=9),
    };
    ms_threshold as f32
}
