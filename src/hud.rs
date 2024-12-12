use bevy::prelude::*;

use crate::{
    globals::DamageEvent,
    player::{PlayerHealth, PlayerWeapon},
    startup::Kulay,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreBoard(0))
            .insert_resource(HealthBarState(Vec::new()))
            .add_systems(Startup, (init_crosshair_ui_system, init_scoreboard_system))
            .add_systems(Startup, init_healthbar_hud)
            .add_systems(
                Update,
                (refresh_scoreboard_system, refresh_crosshair_color_system),
            )
            .add_systems(Update, animate_health_deplete_system);
    }
}

#[derive(Component)]
pub struct ScoreBoardMarker;

#[derive(Resource)]
pub struct ScoreBoard(pub i32);

#[derive(Component)]
pub struct CrossHairMarker;

#[derive(Component)]
struct HealthDepleteMarker;

#[derive(Resource)]
struct HealthBarState(pub Vec<Entity>);

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

fn animate_health_deplete_system(
    mut commands: Commands,
    mut query: Query<(&mut Transform, Entity), With<HealthDepleteMarker>>,
    mut damage_observer: EventReader<DamageEvent>,
    mut health_bar_state: ResMut<HealthBarState>,
    time: Res<Time>,
    mut hearts_to_animate: Local<Vec<Entity>>,
) {
    for _ in damage_observer.read() { // osu how?!?
        let Some(health_bar_entity) = health_bar_state.0.pop() else {
            return;
        };
        let Ok((_, entity)) = query.get(health_bar_entity) else {
            return;
        };
        hearts_to_animate.push(entity);
    }

    hearts_to_animate.retain_mut(|health_bar_entity| {
        let Ok((mut transform, entity)) = query.get_mut(*health_bar_entity) else {
            return false;
        };
        if transform.scale.x >= 0. {
            transform.scale -= Vec3::splat(1.) * time.delta_seconds();
        } else {
            commands.entity(entity).despawn(); // should also despawn its parent
            return false;
        }
        true
    });
}

fn init_healthbar_hud(
    mut commands: Commands,
    player_health: Res<PlayerHealth>,
    asset_server: Res<AssetServer>,
    mut health_bar_state: ResMut<HealthBarState>,
) {
    let container = NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::End,
            position_type: PositionType::Absolute,
            display: Display::Flex,
            padding: UiRect::all(Val::Px(50.)),
            ..default()
        },
        ..default()
    };
    let container = commands.spawn(container).id();
    let heart = UiImage::new(asset_server.load("pixel_heart.png")); // change img
    let mut stack: Vec<Entity> = vec![];
    for _ in 0..player_health.0 {
        let heart_container = NodeBundle {
            style: Style {
                width: Val::Px(32.),
                height: Val::Px(32.),
                margin: UiRect::all(Val::Px(5.)),
                ..default()
            },
            ..default()
        };
        let heart_hud = commands
            .spawn((heart_container, heart.clone(), HealthDepleteMarker))
            .id();
        commands.entity(container).push_children(&[heart_hud]);
        stack.push(heart_hud);
    }

    health_bar_state.0 = stack;
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
