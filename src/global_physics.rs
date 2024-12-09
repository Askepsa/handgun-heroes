use crate::enemy::{eliminate_enemy, EnemyMarker, EnemyState};
use crate::player::{PlayerHealth, PlayerMarker};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct GlobalPhysicsPlugin;

impl Plugin for GlobalPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_enemy_collider_system);
    }
}

pub fn player_enemy_collider_system(
    mut commands: Commands,
    player_collider: Query<Entity, With<PlayerMarker>>,
    enemies: Query<Entity, With<EnemyMarker>>,
    rapier_context: Res<RapierContext>,
    mut player_health: ResMut<PlayerHealth>,
    mut enemy_state: ResMut<EnemyState>,
) {
    let player = player_collider.single();
    for enemy in &enemies {
        // TEMP FIX
        if rapier_context.intersection_pair(player, enemy).is_some() {
            if player_health.0 != 0 {
                player_health.0 -= 1;
            }
            println!("Health: {}", player_health.0);
            eliminate_enemy(&mut commands, enemy, &mut enemy_state);
        }
    }
}
