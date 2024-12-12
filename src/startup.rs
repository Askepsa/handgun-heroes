use crate::enemy::*;
use crate::globals::*;
use crate::player::*;
use crate::hud::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub struct GameStartUp;

#[derive(Component, Debug, PartialEq)]
pub enum Kulay {
    Pula,
    Asul,
}

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiPlugin)
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

    let light = DirectionalLightBundle {
        transform: Transform::from_xyz(0., 30., 0.),
        ..default()
    };
    commands.spawn(light);

    let mut windows = windows.single_mut();
    windows.cursor.grab_mode = CursorGrabMode::Locked;
    windows.cursor.visible = false;
}

fn debug_system(input: Res<ButtonInput<MouseButton>>, cam_pos: Query<&Transform, With<CamMarker>>) {
    let cam_pos = cam_pos.single();
    if input.just_pressed(MouseButton::Right) {
        info!("{:?}", cam_pos.translation);
    }
}

// this should not belong here
pub fn reset_system(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut enemy_state: ResMut<EnemyState>,
) {
    for enemy in &enemies {
        eliminate_enemy(&mut commands, enemy, &mut enemy_state);
    }
}
