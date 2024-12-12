use crate::enemy::{eliminate_enemy, Enemy, EnemyState};
use crate::player::{PlayerHealth, PlayerMarker};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct GlobalPhysicsPlugin;

impl Plugin for GlobalPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(Update, player_enemy_collider_system);
    }
}

#[derive(Event)]
pub struct DamageEvent;

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
            }
            eliminate_enemy(&mut commands, enemy, &mut enemy_state);
        }
    }
}

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
