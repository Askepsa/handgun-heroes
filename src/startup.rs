use crate::enemy::*;
use crate::global_physics::*;
use crate::player::*;
use crate::ui::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;

pub const ENEMY_SPAWN_LIMIT: usize = 3;

pub struct GameStartUp;

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyState(HashMap::new()))
            .insert_resource(ScoreBoard(0))
            .insert_resource(PlayerHealth(5)) // get hit 3 times and ur ded
            .add_systems(
                Startup,
                (
                    startup_system,
                    init_crosshair_ui_system,
                    init_scoreboard_system,
                ),
            )
            .add_systems(Update, refresh_scoreboard_system)
            .add_systems(Update, player_movement_system)
            .add_systems(Update, debug_system)
            .add_systems(Update, (enemy_spawn_system, enemy_movement_system))
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

fn debug_system(input: Res<ButtonInput<MouseButton>>, cam_pos: Query<&Transform, With<CamMarker>>) {
    let cam_pos = cam_pos.single();
    if input.just_pressed(MouseButton::Left) {
        info!("{:?}", cam_pos.translation);
    }
}
