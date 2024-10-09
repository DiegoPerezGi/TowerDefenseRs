use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::{app::Startup, window::PrimaryWindow, DefaultPlugins};

mod fps;
use fps::*;

fn main() {
    App::new()
        .init_resource::<MyWorldCoords>()
        .add_systems(Startup, setup)
        .add_systems(Update, (my_cursor_system, restarting_handle_system))
        .add_systems(Update, animate_sprite_system)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_plugins(LogDiagnosticsPlugin::default())
        .run();
}

#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

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

#[derive(Component)]
struct Orc;

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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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
        // eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        if buttons.just_pressed(MouseButton::Left) {
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            let animation_indices = AnimationIndices { first: 1, last: 5 };
            let id = commands
                .spawn((
                    Orc,
                    SpriteBundle {
                        texture: asset_server.load("orc/orc.png"),
                        transform: Transform {
                            translation: Vec3::new(world_position.x, world_position.y, 6.0),

                            scale: Vec3::splat(6.0),
                            ..default()
                        },
                        ..default()
                    },
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: animation_indices.first,
                    },
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ))
                .id();
            println!("Orc spawned id {}", id)
        }
    }
}

fn restarting_handle_system(
    q_orcs: Query<Entity, With<Orc>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    let mut count = 0;
    if buttons.just_pressed(MouseButton::Middle) {
        for _ in q_orcs.iter() {
            count += 1
        }
        eprintln!("Orcs count: {}", count)
    }
    if buttons.just_pressed(MouseButton::Right) {
        for entity_id in q_orcs.iter() {
            commands.entity(entity_id).despawn();
        }
        eprintln!("Orcs despawned!")
    }
}
