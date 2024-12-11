use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use handgun_heroes::startup::GameStartUp;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(GameStartUp)
        .run();
}
