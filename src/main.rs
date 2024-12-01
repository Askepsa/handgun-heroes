use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};

// - [x] make camera movable globally
// - [x] add crosshair ui
// - [x] add objects to shoot
//  - [x] fix target object spawn system
// - [x] raycast and mouse event button
// - [ ] add scoreboard
// - [ ] add timer
// - [ ] make camera's vertical rotation fixed

const TARGET_LIMIT: usize = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(TargetState(HashMap::new()))
        .add_systems(Startup, (startup_system, startup_ui))
        .add_systems(Update, camera_movement_system)
        .add_systems(Update, debug_system)
        .add_systems(Update, target_spawn_system)
        .add_systems(Update, reset_system.run_if(input_pressed(KeyCode::KeyR)))
        .add_systems(
            Update,
            target_shoot_system.run_if(input_just_pressed(KeyCode::KeyV)),
        )
        .run();
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
    // spawn ui
    let ui_screen = NodeBundle {
        style: Style {
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };
    let crosshair = NodeBundle {
        style: Style {
            height: Val::Px(5.),
            width: Val::Px(5.),
            ..default()
        },
        background_color: BackgroundColor(Color::WHITE),
        ..default()
    };
    let ui = commands.spawn(ui_screen).id();
    let crosshair = commands.spawn(crosshair).id();
    commands.entity(ui).push_children(&[crosshair]);
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
        println!("noice");
    } else {
        println!("missed lol");
    }
}

// x: -15 -> 15, y: 3 -> 5, z: -10 (constant)
fn target_spawn_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut target_state: ResMut<TargetState>,
) {
    if target_state.0.len() >= TARGET_LIMIT {
        return;
    }

    let mut unique_pos: HashSet<(i32, i32)> = target_state
        .0
        .iter()
        .map(|(_, target)| (target.x, target.y))
        .collect();

    let mut rng = thread_rng();
    while unique_pos.len() != TARGET_LIMIT {
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
