use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

// add floor
// add camera
// make camera movable
// add objects lol

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup_system)
        .add_systems(Update, camera_movement_system)
        .run();
}

#[derive(Component)]
struct Cam;

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

fn camera_movement_system(
    mut mouse_evt: EventReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<Cam>>,
) {
    let mut cam = cam.single_mut();
    for mouse_motion in mouse_evt.read() {
        let yaw = -mouse_motion.delta.x * 0.003;
        let pitch = -mouse_motion.delta.y * 0.003;
        cam.rotate_y(yaw);
        cam.rotate_local_x(pitch);
    }
}
