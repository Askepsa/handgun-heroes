use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

// - [x] make camera movable globally
// - [ ] add crosshair ui
// - [ ] add objects to shoot
// - [ ] raycast and mouse event button

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup_system)
        .add_systems(Update, camera_movement_system)
        .add_systems(Update, spawn_in_yo_face)
        .add_systems(Update, rotate_cubes)
        .run();
}

#[derive(Component)]
struct Cam;

#[derive(Component)]
struct Naoxi;

fn startup_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
) {
    let cam = Camera3dBundle {
        transform: Transform::from_xyz(5., 2., 5.),
        ..Default::default()
    };
    commands.spawn((Cam, cam));

    let plane = Plane3d::new(Vec3::new(0., 1., 0.), Vec2::new(20., 20.));
    let floor = MaterialMeshBundle {
        mesh: mesh.add(plane),
        material: material.add(Color::WHITE),
        ..Default::default()
    };
    commands.spawn(floor);
}

fn spawn_in_yo_face(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mouse_evt: Res<ButtonInput<MouseButton>>,
    cam: Query<&Transform, With<Cam>>,
) {
    if mouse_evt.just_pressed(MouseButton::Left) {
        println!("spawned");
        let cam = cam.single();
        let cube = mesh.add(Cuboid::new(5., 5., 5.));
        let cube_bundle = MaterialMeshBundle {
            mesh: cube,
            material: material.add(Color::WHITE),
            transform: *cam,
            ..Default::default()
        };
        commands.spawn((Naoxi, cube_bundle));
    }
}

fn rotate_cubes(mut cubes: Query<&mut Transform, With<Naoxi>>, time: Res<Time>) {
    for mut cubes in cubes.iter_mut() {
        cubes.rotate(Quat::from_euler(
            EulerRot::YXZ,
            0.,
            1. * time.delta_seconds(),
            0.,
        ));
    }
}

fn camera_movement_system(
    mut mouse_evt: EventReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<Cam>>,
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
