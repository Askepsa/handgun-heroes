use crate::enemy::*;
use crate::global_physics::*;
use crate::player::*;
use crate::ui::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use std::collections::HashMap;

pub const ENEMY_SPAWN_LIMIT: usize = 3;

pub struct GameStartUp;

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyState(HashMap::new()))
            .insert_resource(ScoreBoard(0))
            .insert_resource(PlayerHealth(5))
            .add_plugins(UiPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(EnemyPlugin)
            .add_plugins(GlobalPhysicsPlugin)
            .add_systems(Startup, init_world_system)
            .add_systems(Update, debug_system)
            .add_systems(
                Update,
                reset_system.run_if(input_just_pressed(KeyCode::KeyR)),
            );
    }
}

#[derive(Component)]
pub struct CamMarker;

fn init_world_system(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
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
