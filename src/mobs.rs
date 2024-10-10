use bevy::prelude::*;

use crate::animate_sprite_system;

pub fn mob_plugin(app: &mut App) {
    app.add_event::<DespawnOrcsEvent>();
    app.add_event::<SpawnOrcEvent>();
    app.add_systems(
        Update,
        (
            spawn_orc_system,
            despawn_orc_system,
            update_health_bar,
            animate_sprite_system,
        ),
    );
}

#[derive(Event)]
pub struct SpawnOrcEvent {
    pub world_position: Vec2,
}

#[derive(Event)]
pub struct DespawnOrcsEvent;

#[derive(Component, Clone)]
pub struct Orc {
    name: String,
}
#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct EmptyHealthBar;

#[derive(Component)]
struct NameTag;

#[derive(Component)]
struct Health {
    actual: f32,
    max: f32,
}

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn despawn_orc_system(
    q_orcs: Query<Entity, With<Orc>>,
    mut commands: Commands,
    mut ev_despawn_orcs: EventReader<DespawnOrcsEvent>,
) {
    for _ in ev_despawn_orcs.read() {
        for entity_id in q_orcs.iter() {
            commands.entity(entity_id).despawn_recursive();
        }
        eprintln!("Orcs despawned!")
    }
}

fn spawn_orc_system(
    mut commands: Commands,
    mut ev_spawn_orc: EventReader<SpawnOrcEvent>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for ev in ev_spawn_orc.read() {
        let orc = Orc {
            name: "Orc".to_string(),
        };

        let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let animation_indices = AnimationIndices { first: 1, last: 5 };
        commands
            .spawn((
                orc.clone(),
                Health {
                    actual: 100.0,
                    max: 100.0,
                },
                SpriteBundle {
                    texture: asset_server.load("orc/orc.png"),
                    transform: Transform {
                        translation: Vec3::new(ev.world_position.x, ev.world_position.y, 6.0),

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
                parent
                    .spawn((
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
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Srgba::rgb(1.0, 0.0, 0.0).into(),
                                    custom_size: Some(Vec2::new(0.0, 10.0)),
                                    ..default()
                                },
                                transform: Transform {
                                    translation: (Vec3::new(0.0, 0.0, 1.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            HealthBar,
                        ));
                    });
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
fn update_health_bar(
    q_parent: Query<(&Health, &Children)>,
    q_empty_health_bar: Query<(&Children, &Transform), (With<EmptyHealthBar>, Without<HealthBar>)>,
    mut q_health_bar: Query<(&mut Sprite, &mut Transform), With<HealthBar>>,
) {
    for (health, children) in q_parent.iter() {
        for &child in children.iter() {
            if let Ok((grandchildren, empty_bar_transform)) = q_empty_health_bar.get(child) {
                for &grandson in grandchildren.iter() {
                    // Verifica si el child tiene una `Sprite` con `HealthBar`
                    if let Ok((mut sprite, mut transform)) = q_health_bar.get_mut(grandson) {
                        let health_percentage = (health.actual) / health.max;
                        let full_width = 100.0;
                        let new_width = health_percentage * full_width;

                        // Actualiza el tamaño de la barra de salud
                        sprite.custom_size = Some(Vec2::new(new_width, 10.0));
                        transform.translation.x = empty_bar_transform.translation.x
                            - (full_width / 2.0)
                            + (new_width / 2.0);
                        println!("health: {}", new_width);
                    }
                }
            }
        }
    }
}
