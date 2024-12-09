use crate::player::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

pub const ENEMY_SPAWN_LIMIT: usize = 3;

pub struct GameStartUp;

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyState(HashMap::new()))
            .insert_resource(ScoreBoard(0))
            .insert_resource(PlayerHealth(5)) // get hit 3 times and ur ded
            .add_systems(Startup, (startup_system, startup_ui, startup_scoreboard_ui))
            .add_systems(Update, scoreboard_system)
            .add_systems(Update, player_movement_system)
            .add_systems(Update, debug_system)
            .add_systems(Update, (enemy_spawn_system, move_enemy_system))
            .add_systems(Update, player_enemy_collide_system)
            .add_systems(
                Update,
                reset_system.run_if(input_just_pressed(KeyCode::KeyR)),
            )
            .add_systems(
                Update,
                player_shoot_system.run_if(input_just_pressed(KeyCode::KeyV)),
            );
    }
}

#[derive(Component)]
pub struct CamMarker;

#[derive(Component)]
pub struct EnemyMarker;

#[derive(Component, Debug)]
struct EnemyPos {
    x: i32,
    y: i32,
}

#[derive(Resource)]
pub struct EnemyState(HashMap<Entity, EnemyPos>);

#[derive(Component)]
struct ScoreBoardMarker;

#[derive(Resource)]
pub struct ScoreBoard(pub i32);

fn startup_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let player_collider = commands
        .spawn(Collider::capsule(
            Vect::new(0., 0., 0.),
            Vect::new(0., 5., 0.),
            5.,
        ))
        .insert(PlayerMarker)
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2))
        .id();
    let cam = Camera3dBundle {
        transform: Transform::from_xyz(0., 3., 0.),
        ..Default::default()
    };
    commands.spawn((CamMarker, cam)).add_child(player_collider);

    let plane = Plane3d::new(Vec3::new(0., 1., 0.), Vec2::new(20., 20.));
    let floor = MaterialMeshBundle {
        mesh: mesh.add(plane),
        material: material.add(Color::WHITE),
        ..Default::default()
    };
    commands.spawn(floor);

    let mut windows = windows.single_mut();
    windows.cursor.grab_mode = CursorGrabMode::Locked;
    windows.cursor.visible = false;
}

fn startup_ui(mut commands: Commands) {
    let ui_screen = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };
    let ui_entity = commands.spawn(ui_screen).id();

    let crosshair_bundle = NodeBundle {
        style: Style {
            height: Val::Px(5.),
            width: Val::Px(5.),
            ..default()
        },
        background_color: BackgroundColor(Color::WHITE),
        ..default()
    };
    let crosshair_entity = commands.spawn(crosshair_bundle).id();
    commands
        .entity(ui_entity)
        .push_children(&[crosshair_entity]);
}

fn startup_scoreboard_ui(mut commands: Commands, score_board: Res<ScoreBoard>) {
    let scoreboard_ui = NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(50.)),
            ..default()
        },
        ..default()
    };
    let scoreboard = commands.spawn(scoreboard_ui).id();
    let text_bundle = TextBundle::from_section(
        &format!("Score: {}", score_board.0),
        TextStyle {
            font_size: 50.,
            ..default()
        },
    );
    let text_entity = commands.spawn((ScoreBoardMarker, text_bundle)).id();
    let scoreboard_entity = commands
        .entity(scoreboard)
        .push_children(&[text_entity])
        .id();

    let ui_screen = NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    };
    let ui_entity = commands.spawn(ui_screen).id();

    commands
        .entity(ui_entity)
        .push_children(&[scoreboard_entity]);
}

fn scoreboard_system(
    scoreboard_points: Res<ScoreBoard>,
    mut scoreboard_ui: Query<&mut Text, With<ScoreBoardMarker>>,
) {
    let mut score_ui = scoreboard_ui.single_mut();
    *score_ui = Text::from_section(
        format!("Score: {}", scoreboard_points.0),
        TextStyle {
            font_size: 50.,
            ..default()
        },
    );
}

fn debug_system(input: Res<ButtonInput<MouseButton>>, cam_pos: Query<&Transform, With<CamMarker>>) {
    let cam_pos = cam_pos.single();
    if input.just_pressed(MouseButton::Left) {
        info!("{:?}", cam_pos.translation);
    }
}

fn player_enemy_collide_system(
    mut commands: Commands,
    player_collider: Query<Entity, With<PlayerMarker>>,
    enemies: Query<Entity, With<EnemyMarker>>,
    rapier_context: Res<RapierContext>,
    mut player_health: ResMut<PlayerHealth>,
    mut enemy_state: ResMut<EnemyState>,
) {
    let player = player_collider.single();
    for enemy in &enemies {
        // TEMP FIX
        if let Some(_) = rapier_context.intersection_pair(player, enemy) {
            if player_health.0 != 0 {
                player_health.0 -= 1;
            }
            println!("Health: {}", player_health.0);
            eliminate_enemy(&mut commands, enemy, &mut enemy_state);
        }
    }
}

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
fn move_enemy_system(mut enem_pos: Query<&mut Transform, With<EnemyMarker>>, time: Res<Time>) {
    for mut pos in enem_pos.iter_mut() {
        pos.translation.z += 10. * time.delta_seconds();
    }
}

fn enemy_spawn_system(
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
        .iter()
        .map(|(_, enemy)| (enemy.x, enemy.y))
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

fn reset_system(
    mut commands: Commands,
    enemies: Query<Entity, With<EnemyMarker>>,
    mut enemy_state: ResMut<EnemyState>,
) {
    for enemy in &enemies {
        eliminate_enemy(&mut commands, enemy, &mut enemy_state);
    }
}
