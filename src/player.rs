use crate::enemy::{eliminate_enemy, EnemyState};
use crate::globals::Kulay;
use crate::hud::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerHealth(5))
            .insert_resource(PlayerWeapon(Kulay::Asul))
            .insert_resource(KillCount(0))
            .add_systems(Startup, init_player)
            .add_systems(Update, (player_movement_system, switch_weapon_system))
            .add_systems(
                Update,
                (player_shoot_system).run_if(input_just_pressed(MouseButton::Left)),
            );
    }
}

#[derive(Component)]
pub struct CamMarker;

#[derive(Resource)]
pub struct PlayerHealth(pub usize);

#[derive(Resource)]
pub struct PlayerWeapon(pub Kulay);

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Resource)]
pub struct KillCount(pub usize);

fn init_player(mut commands: Commands) {
    let player_collider = commands
        .spawn(Collider::cuboid(10., 10., 1.))
        .insert(PlayerMarker)
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2))
        .id();
    let cam = Camera3dBundle {
        transform: Transform::from_xyz(0., 4.6, 0.),
        ..Default::default()
    };
    let fog = FogSettings {
        color: Color::srgb(0.25, 0.25, 0.25),
        falloff: FogFalloff::Linear {
            start: 15.0,
            end: 100.0,
        },
        ..default()
    };
    commands
        .spawn((CamMarker, cam, fog))
        .add_child(player_collider);
}

fn switch_weapon_system(keys: Res<ButtonInput<KeyCode>>, mut weapon: ResMut<PlayerWeapon>) {
    if keys.just_pressed(KeyCode::Digit1) {
        weapon.0 = Kulay::Asul;
    } else if keys.just_pressed(KeyCode::Digit2) {
        weapon.0 = Kulay::Pula;
    }
}

fn player_movement_system(
    mut mouse_evt: EventReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<CamMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut cam = cam.single_mut();
    for mouse_motion in mouse_evt.read() {
        let delta_yaw = -mouse_motion.delta.x * 0.0005;
        let delta_pitch = -mouse_motion.delta.y * 0.0005;
        let (yaw, pitch, roll) = cam.rotation.to_euler(EulerRot::YXZ);
        let yaw = (yaw + delta_yaw).clamp(-0.1, 0.1);
        let pitch = (pitch + delta_pitch).clamp(-0.1, 0.1);
        cam.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
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

fn player_shoot_system(
    mut commands: Commands,
    cam: Query<(&GlobalTransform, &Camera), With<CamMarker>>,
    mut enemy_state: ResMut<EnemyState>,
    rapier_context: Res<RapierContext>,
    mut scoreboard: ResMut<Score>,
    mut kill_count: ResMut<KillCount>,
    enemies: Query<&Kulay>,
    player_weapon: Res<PlayerWeapon>,
    win: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let (cam_transform, cam) = cam.single();

    let Some(cursor_position) = win.single().cursor_position() else {
        return;
    };

    let Some(ray) = cam.viewport_to_world(cam_transform, cursor_position) else {
        return;
    };

    let ray_context = rapier_context.cast_ray(
        ray.origin,
        *ray.direction,
        255.,
        false,
        QueryFilter::default().groups(CollisionGroups::new(Group::default(), Group::GROUP_2)),
    );

    let Some((entity, _)) = ray_context else {
        return;
    };

    if let Ok(color) = enemies.get(entity) {
        if *color == player_weapon.0 {
            eliminate_enemy(&mut commands, entity, &mut enemy_state);
            scoreboard.0 += 100;
            kill_count.0 += 1;
            commands.spawn(AudioBundle {
                source: asset_server.load("attack.ogg"),
                settings: PlaybackSettings::DESPAWN,
            });
        } else {
            scoreboard.0 -= 100;
            commands.spawn(AudioBundle {
                source: asset_server.load("damage.ogg"),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    } else {
        scoreboard.0 -= 100;
    }
}
