use crate::enemy::{eliminate_enemy, Enemy, EnemyState};
use crate::hud::{clean_hud_system, HudEntities, Score};
use crate::player::{KillCount, PlayerHealth, PlayerMarker};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub enum Kulay {
    Pula,
    Asul,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    InGame,
    GameOver,
}

#[derive(Event)]
pub struct DamageEvent;

pub struct Global;

impl Plugin for Global {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(Update, player_enemy_collider_system)
            .add_systems(
                Update,
                (clean_hud_system, update_game_state_to_ingame)
                    .run_if(mouse_pressed_and_not_ingame),
            );
    }
}

fn mouse_pressed_and_not_ingame(
    game_state: Res<State<GameState>>,
    input: Res<ButtonInput<MouseButton>>,
) -> bool {
    *game_state.get() != GameState::InGame && input.just_pressed(MouseButton::Left)
}

fn update_game_state_to_ingame(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        next_state.set(GameState::InGame);
    }
}

pub fn player_enemy_collider_system(
    mut commands: Commands,
    player_collider: Query<Entity, With<PlayerMarker>>,
    enemies: Query<Entity, With<Enemy>>,
    rapier_context: Res<RapierContext>,
    mut player_health: ResMut<PlayerHealth>,
    mut enemy_state: ResMut<EnemyState>,
    mut damage_event: EventWriter<DamageEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    game_state: Res<State<GameState>>,
    asset_server: Res<AssetServer>
) {
    if player_health.0 == 0 && *game_state.get() != GameState::GameOver {
        next_state.set(GameState::GameOver);
        return;
    }

    let player = player_collider.single();
    for enemy in &enemies {
        // TEMP FIX
        if rapier_context.intersection_pair(player, enemy).is_some() {
            if player_health.0 != 0 {
                player_health.0 -= 1;
                damage_event.send(DamageEvent);
                commands.spawn(AudioBundle {
                    source: asset_server.load("rizz.ogg"),
                    settings: PlaybackSettings::default(),
                });
            }
            eliminate_enemy(&mut commands, enemy, &mut enemy_state);
        }
    }
}

// this should not belong here
// move this to global module
pub fn reset_system(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut enemy_state: ResMut<EnemyState>,
    mut score: ResMut<Score>,
    mut hud_entities: ResMut<HudEntities>,
    mut player_health: ResMut<PlayerHealth>,
    mut kill_count: ResMut<KillCount>,
) {
    for enemy in &enemies {
        eliminate_enemy(&mut commands, enemy, &mut enemy_state);
    }

    score.0 = 0;
    hud_entities.0.clear();
    player_health.0 = 5;
    kill_count.0 = 0;
    *enemy_state = EnemyState::default();
}
