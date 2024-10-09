use bevy::prelude::*;
use bevy::{app::Startup, window::PrimaryWindow, DefaultPlugins};

fn main() {
    App::new()
        .init_resource::<MyWorldCoords>()
        .add_systems(Startup, setup)
        .add_systems(Update, (my_cursor_system, restarting_handle_system))
        .add_systems(Update, animate_sprite_system)
        .add_systems(Update, update_health_bar)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .run();
}

// HealthBar
//
#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct EmptyHealthBar;

#[derive(Component)]
struct NameTag;

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

#[derive(Component, Clone)]
struct Orc {
    name: String,
}

#[derive(Component)]
struct Health {
    actual: f32,
    max: f32,
}

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
        if buttons.just_pressed(MouseButton::Left) {
            let orc = Orc {
                name: "Grom".to_string(),
            };

            let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            let animation_indices = AnimationIndices { first: 1, last: 5 };
            commands
                .spawn((
                    orc.clone(),
                    Health {
                        actual: 50.0,
                        max: 100.0,
                    },
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
                .with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Srgba::rgb(1.0, 0.0, 0.0).into(),
                                custom_size: Some(Vec2::new(100.0, 10.0)),
                                ..default()
                            },
                            transform: Transform {
                                translation: (Vec3::new(0.0, -13.0, 1.0)),
                                scale: (Vec3::splat(1.0 / 6.0)),
                                ..default()
                            },
                            ..default()
                        },
                        HealthBar,
                    ));
                    parent.spawn((
                        EmptyHealthBar,
                        SpriteBundle {
                            sprite: Sprite {
                                color: Srgba::rgb(0.0, 0.0, 0.0).into(),
                                custom_size: Some(Vec2::new(100.0, 10.0)),
                                ..default()
                            },
                            transform: Transform {
                                translation: (Vec3::new(0.0, -13.0, 0.0)),
                                scale: (Vec3::splat(1.0 / 6.0)),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                    let text_style = TextStyle {
                        color: Srgba::rgb(1.0, 1.0, 1.0).into(),
                        ..default()
                    };
                    parent.spawn((
                        Text2dBundle {
                            text: Text::from_sections([TextSection::new(
                                orc.name.clone(),
                                text_style.clone(),
                            )]),
                            transform: Transform {
                                translation: (Vec3::new(0.0, -16.0, 0.0)),
                                scale: (Vec3::splat(1.0 / 6.0)),
                                ..default()
                            },
                            ..default()
                        },
                        NameTag,
                    ));
                });
        }
    }
}

//fn update_health_bar(
//    q_parent: Query<(&Health, &Children)>,
//    q_child: Query<&mut Sprite, With<HealthBar>>,
//) {
//    for (health, children) in q_parent.iter() {
//        for &child in children.iter_mut() {
//            let mut Ok(sprite) = q_child.get(child);
//            let health_percentage = health.actual * 100.0 / health.max;
//            sprite.custom_size = Some(Vec2::new(health_percentage, 10.0));
//            println!("health: {}", health_percentage)
//        }
//    }
//}

fn update_health_bar(
    q_parent: Query<(&Health, &Children)>, // `Query` en lugar de `query`, y `Children` con mayúscula
    mut q_child: Query<&mut Sprite, With<HealthBar>>, // `Query` en lugar de `query`, y `With` con mayúscula
) {
    for (health, children) in q_parent.iter() {
        for &child in children.iter() {
            // Verifica si el child tiene una `Sprite` con `HealthBar`
            if let Ok(mut sprite) = q_child.get_mut(child) {
                // Calcula el porcentaje de vida
                let health_percentage = (health.actual * 100.0) / health.max;
                // Actualiza el tamaño de la barra de salud
                sprite.custom_size = Some(Vec2::new(health_percentage, 10.0));
                println!("health: {}", health_percentage);
            }
        }
    }
}

fn restarting_handle_system(
    q_orcs: Query<Entity, With<Orc>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Right) {
        for entity_id in q_orcs.iter() {
            commands.entity(entity_id).despawn_recursive();
        }
        eprintln!("Orcs despawned!")
    }
}
