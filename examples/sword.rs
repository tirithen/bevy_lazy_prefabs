use bevy::{prelude::*, render::camera::OrthographicProjection};
use bevy_lazy_prefabs::*;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Equippable;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Item;

#[derive(Default, Component, Reflect, Debug)]
#[reflect(Component)]
struct DealsDamage {
    value: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LazyPrefabsPlugin)
        .add_startup_system(setup)
        .add_startup_system_to_stage(StartupStage::PostStartup, read_damage)
        .run();
}

fn setup(mut commands: Commands, mut registry: ResMut<PrefabRegistry>) {
    registry.register_type::<Item>();
    registry.register_type::<Equippable>();
    registry.register_type::<DealsDamage>();

    let sword = registry.load("sharp_sword.prefab").unwrap();
    commands.spawn_empty().insert_prefab(sword);

    let cam = registry.load("cam_2d.prefab").unwrap();
    commands
        .spawn_empty()
        .insert_prefab(cam)
        .insert(OrthographicProjection {
            scale: 0.1,
            ..Default::default()
        });
}

fn read_damage(q_sword: Query<&DealsDamage>) {
    let damage = q_sword.single();
    println!("{:#?}", damage);
}
