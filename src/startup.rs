use crate::enemy::*;
use crate::globals::*;
use crate::hud::*;
use crate::player::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub struct GameStartUp;

impl Plugin for GameStartUp {
    fn build(&self, app: &mut App) {
        app.add_plugins(HudPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(EnemyPlugin)
            .add_plugins(Global)
            .add_systems(Startup, (init_world_system, init_bgm))
            .add_systems(Update, debug_system)
            .add_systems(
                Update,
                reset_system.run_if(input_just_pressed(KeyCode::KeyR)),
            );

        app.insert_state(GameState::InGame);
    }
}

fn init_world_system(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<StandardMaterial>>,
) {
    // look at me
    let light = DirectionalLightBundle {
        transform: Transform::from_xyz(0., 25., 0.),
        ..default()
    };
    commands.spawn(light);

    let sphere = MaterialMeshBundle {
        mesh: mesh.add(Sphere { radius: 300. }),
        transform: Transform::from_xyz(0., 10., -1000.),
        material: material.add(StandardMaterial::default()),
        ..default()
    };
    commands.spawn(sphere);

    let mut windows = windows.single_mut();
    windows.cursor.grab_mode = CursorGrabMode::Locked;
    windows.cursor.visible = false;
}

fn init_bgm(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioBundle {
        source: asset_server.load("zenith.ogg"),
        settings: PlaybackSettings::LOOP,
    });
}

fn debug_system(input: Res<ButtonInput<MouseButton>>, cam_pos: Query<&Transform, With<CamMarker>>) {
    let cam_pos = cam_pos.single();
    if input.just_pressed(MouseButton::Right) {
        info!("{:#?}", cam_pos);
    }
}
