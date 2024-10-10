use bevy::prelude::*;
use bevy::{app::Startup, window::PrimaryWindow, DefaultPlugins};
use mobs::*;

mod mobs;

fn main() {
    App::new()
        .init_resource::<MyWorldCoords>()
        .add_plugins(mobs::mob_plugin)
        .add_systems(Startup, setup)
        .add_systems(Update, my_cursor_system)
        .add_systems(Update, animate_sprite_system)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .run();
}

// HealthBar
//

#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Make sure to add the marker component when you set up your camera
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn my_cursor_system(
    mut mycoords: ResMut<MyWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut ev_spawn_orc: EventWriter<SpawnOrcEvent>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        if buttons.just_pressed(MouseButton::Left) {
            ev_spawn_orc.send(SpawnOrcEvent { world_position });
        };
    }
}
