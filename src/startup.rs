use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

pub const TARGET_SPAWN_LIMIT: usize = 3;

pub struct GameStartUp;

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.insert_resource(TargetState(HashMap::new()))
            .insert_resource(ScoreBoard(0))
            .add_systems(Startup, (startup_system, startup_ui, startup_scoreboard_ui))
            .add_systems(Update, scoreboard_system)
            .add_systems(Update, camera_movement_system)
            .add_systems(Update, debug_system)
            .add_systems(Update, target_spawn_system)
            .add_systems(Update, reset_system.run_if(input_pressed(KeyCode::KeyR)))
            .add_systems(
                Update,
                target_shoot_system.run_if(input_just_pressed(MouseButton::Left)),
            );
    }
}

#[derive(Component)]
struct CamMarker;

#[derive(Component)]
struct TargetMarker;

#[derive(Component, Debug)]
struct Target {
    x: i32,
    y: i32,
}

#[derive(Resource)]
struct TargetState(HashMap<Entity, Target>);

#[derive(Component)]
struct ScoreBoardMarker;

#[derive(Resource)]
struct ScoreBoard(i32);

fn startup_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let cam = Camera3dBundle {
        transform: Transform::from_xyz(0., 3., 0.),
        ..Default::default()
    };
    commands.spawn((CamMarker, cam));

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

fn target_shoot_system(
    mut commands: Commands,
    cam: Query<&Transform, With<CamMarker>>,
    mut target_state: ResMut<TargetState>,
    rapier_context: Res<RapierContext>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    let cam = cam.single();
    let ray_context = rapier_context.cast_ray(
        cam.translation,
        *cam.forward(),
        255.,
        true,
        bevy_rapier3d::prelude::QueryFilter {
            ..Default::default()
        },
    );

    // since we're only dealing with spheres
    // collider, then there's no use to
    // check its groups
    if let Some((entity, _)) = ray_context {
        // plus points if clicked
        let (entity, _) = target_state
            .0
            .remove_entry(&entity)
            .expect("sumabog ang entity");
        commands.entity(entity).despawn();
        scoreboard.0 += 100;
    } else {
        scoreboard.0 -= 100;
    }

    println!("Score: {}", scoreboard.0);
}

fn target_spawn_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut target_state: ResMut<TargetState>,
) {
    if target_state.0.len() >= TARGET_SPAWN_LIMIT {
        return;
    }

    let mut unique_pos: HashSet<(i32, i32)> = target_state
        .0
        .iter()
        .map(|(_, target)| (target.x, target.y))
        .collect();

    let mut rng = thread_rng();
    while unique_pos.len() != TARGET_SPAWN_LIMIT {
        let (x, y) = (rng.gen_range(-5..=5), rng.gen_range(3..=6));
        unique_pos.insert((x, y));
    }

    for (x, y) in unique_pos {
        if target_state
            .0
            .iter()
            .any(|(_, target)| target.x == x && target.y == y)
        {
            continue;
        }

        let sphere = Sphere { radius: 1. };
        let sphere_bundle = MaterialMeshBundle {
            mesh: mesh.add(sphere),
            transform: Transform::from_xyz(x as f32, y as f32, -15.),
            material: material.add(Color::WHITE),
            ..default()
        };

        let target_id = commands
            .spawn((TargetMarker, sphere_bundle))
            .insert(Collider::ball(1.))
            .id();

        target_state.0.insert(target_id, Target { x, y });
    }
}

fn reset_system(
    mut commands: Commands,
    targets: Query<Entity, With<TargetMarker>>,
    mut target_state: ResMut<TargetState>,
) {
    for target in &targets {
        commands.entity(target).despawn();
    }
    target_state.0.clear();
}

fn camera_movement_system(
    mut mouse_evt: EventReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<CamMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut cam = cam.single_mut();
    for mouse_motion in mouse_evt.read() {
        let yaw = -mouse_motion.delta.x * 0.003;
        let pitch = -mouse_motion.delta.y * 0.003;
        cam.rotate_y(yaw);
        cam.rotate_local_x(pitch);
    }

    for key in keys.get_pressed() {
        let mut movement = Vec3::ZERO;
        match key {
            KeyCode::KeyW => movement += *cam.forward(),
            KeyCode::KeyA => movement += *cam.left(),
            KeyCode::KeyS => movement += *cam.back(),
            KeyCode::KeyD => movement += *cam.right(),
            KeyCode::Space => movement += *cam.up(),
            KeyCode::ShiftLeft => movement += *cam.down(),
            _ => (),
        }
        cam.translation += movement * 10. * time.delta_seconds();
    }
}
