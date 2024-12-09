use crate::enemy::{eliminate_enemy, EnemyState};
use crate::startup::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Resource)]
pub struct PlayerHealth(pub usize);

#[derive(Component)]
pub struct PlayerMarker;

pub fn player_movement_system(
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

pub fn player_shoot_system(
    mut commands: Commands,
    cam: Query<&Transform, With<CamMarker>>,
    mut enemy_state: ResMut<EnemyState>,
    rapier_context: Res<RapierContext>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    let cam = cam.single();
    let ray_context = rapier_context.cast_ray(
        cam.translation,
        *cam.forward(),
        255.,
        false,
        QueryFilter::default().groups(CollisionGroups::new(Group::default(), Group::GROUP_2)),
    );

    // since we're only dealing with spheres
    // collider, then there's no use to
    // check its groups
    if let Some((entity, _)) = ray_context {
        // plus points if clicked
        eliminate_enemy(&mut commands, entity, &mut enemy_state);
        scoreboard.0 += 100;
    } else {
        scoreboard.0 -= 100;
    }

    println!("Score: {}", scoreboard.0);
}
