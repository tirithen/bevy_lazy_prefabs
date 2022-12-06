use bevy::prelude::*;
use bevy_lazy_prefabs::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
    let sprite = registry.load("sprite.prefab").unwrap();
    commands.spawn_empty().insert_prefab(sprite);

    let cam = registry.load("cam_2d.prefab").unwrap();
    commands.spawn_empty().insert_prefab(cam);
}
