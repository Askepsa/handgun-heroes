use bevy::prelude::*;

use crate::{player::PlayerWeapon, startup::Kulay};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreBoard(0))
            .add_systems(Startup, (init_crosshair_ui_system, init_scoreboard_system))
            .add_systems(
                Update,
                (refresh_scoreboard_system, refresh_crosshair_color_system),
            );
    }
}

#[derive(Component)]
pub struct ScoreBoardMarker;

#[derive(Resource)]
pub struct ScoreBoard(pub i32);

#[derive(Component)]
pub struct CrossHairMarker;

// rename these shets
pub fn refresh_scoreboard_system(
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

fn refresh_crosshair_color_system(
    mut crosshair: Query<&mut BackgroundColor, With<CrossHairMarker>>,
    player_weapon: Res<PlayerWeapon>,
) {
    *crosshair.single_mut() = match player_weapon.0 {
        Kulay::Asul => BackgroundColor(Color::hsl(240., 1.0, 0.5)),
        Kulay::Pula => BackgroundColor(Color::hsl(0., 0.5, 0.5)),
    };
}

fn init_scoreboard_system(mut commands: Commands, score_board: Res<ScoreBoard>) {
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
        format!("Score: {}", score_board.0),
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

fn init_crosshair_ui_system(mut commands: Commands) {
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
        background_color: BackgroundColor(Color::hsl(240., 1.0, 0.5)),
        ..default()
    };
    let crosshair_entity = commands.spawn((CrossHairMarker, crosshair_bundle)).id();
    commands
        .entity(ui_entity)
        .push_children(&[crosshair_entity]);
}
