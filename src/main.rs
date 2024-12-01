use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use handgun_heroes::startup::GameStartUp;

// - [x] make camera movable globally
// - [x] add crosshair ui
// - [x] add objects to shoot
//  - [x] fix target object spawn system
// - [x] raycast and mouse event button
// - [ ] add scoreboard
// - [ ] add timer
// - [ ] make camera's vertical rotation fixed

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GameStartUp)
        .run();
}
