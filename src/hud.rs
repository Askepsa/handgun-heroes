use bevy::prelude::*;

use crate::{
    globals::{DamageEvent, GameState, Kulay},
    player::{PlayerHealth, PlayerWeapon},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Score(0))
            .insert_resource(HealthBarState(Vec::new()))
            .insert_resource(HudEntities(Vec::new()));

        app.add_systems(
            Startup,
            (
                init_crosshair_ui_system,
                init_scoreboard_system,
                init_healthbar_hud,
            ),
        )
        .add_systems(
            Update,
            (
                refresh_scoreboard_system,
                refresh_crosshair_color_system,
                animate_health_deplete_system,
            ),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (clean_hud_system, init_gameover_screen).chain(),
        );
    }
}

#[derive(Component)]
pub struct ScoreBoardMarker;

#[derive(Resource)]
pub struct Score(pub i32);

#[derive(Component)]
pub struct CrossHairMarker;

#[derive(Component)]
struct HealthDepleteMarker;

#[derive(Resource)]
struct HealthBarState(pub Vec<Entity>);

#[derive(Resource)]
struct HudEntities(pub Vec<Entity>);

// rename these shets
pub fn refresh_scoreboard_system(
    scoreboard_points: Res<Score>,
    mut scoreboard_ui: Query<&mut Text, With<ScoreBoardMarker>>,
) {
    for mut score_ui in scoreboard_ui.iter_mut() {
        *score_ui = Text::from_section(
            format!("Score: {}", scoreboard_points.0),
            TextStyle {
                font_size: 50.,
                ..default()
            },
        );
    }
}

fn clean_hud_system(mut commands: Commands, mut hud_entities: ResMut<HudEntities>) {
    for _ in 0..hud_entities.0.len() {
        if let Some(entity) = hud_entities.0.pop() {
            if let Some(entity) = commands.get_entity(entity) {
                entity.despawn_recursive();
            }
        }
    }
}

fn refresh_crosshair_color_system(
    mut crosshair: Query<&mut BackgroundColor, With<CrossHairMarker>>,
    player_weapon: Res<PlayerWeapon>,
) {
    for mut crosshair in crosshair.iter_mut() {
        *crosshair = match player_weapon.0 {
            Kulay::Asul => BackgroundColor(Color::hsl(240., 1.0, 0.5)),
            Kulay::Pula => BackgroundColor(Color::hsl(0., 0.5, 0.5)),
        };
    }
}

fn animate_health_deplete_system(
    mut commands: Commands,
    mut query: Query<(&mut Transform, Entity), With<HealthDepleteMarker>>,
    mut damage_observer: EventReader<DamageEvent>,
    mut health_bar_state: ResMut<HealthBarState>,
    time: Res<Time>,
    mut hearts_to_animate: Local<Vec<Entity>>,
) {
    for _ in damage_observer.read() {
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
    mut hud_entities: ResMut<HudEntities>,
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
    hud_entities.0.push(container);

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

fn init_scoreboard_system(
    mut commands: Commands,
    score_board: Res<Score>,
    mut hud_entities: ResMut<HudEntities>,
) {
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
    hud_entities.0.push(scoreboard);

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

fn init_crosshair_ui_system(mut commands: Commands, mut hud_entities: ResMut<HudEntities>) {
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
    hud_entities.0.push(ui_entity);

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

fn init_gameover_screen(
    mut commands: Commands,
    score: Res<Score>,
    mut hud_entities: ResMut<HudEntities>,
) {
    let screen = NodeBundle {
        style: Style {
            //position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.)),
            ..default()
        },
        ..default()
    };
    let screen = commands.spawn(screen).id();
    hud_entities.0.push(screen);

    let text_score = TextBundle {
        text: Text::from_section(
            format!("You Scored: {}", score.0),
            TextStyle {
                font_size: 64.,
                ..default()
            },
        ),
        ..default()
    };
    let text_score = commands.spawn(text_score).id();

    // let's hope na di na kailangan ng node bundle kapag maglalagay ng text
    let text_label = TextBundle {
        style: Style {
            margin: UiRect::top(Val::Percent(2.)),
            ..default()
        },
        text: Text::from_section(
            "Shoot to Play Again",
            TextStyle {
                font_size: 32.,
                ..default()
            },
        ),
        ..default()
    };
    let text_label = commands.spawn(text_label).id();

    commands
        .entity(screen)
        .push_children(&[text_score, text_label]);
}
