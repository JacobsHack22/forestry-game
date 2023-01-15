use crate::data::{CurrentQuestInfo, QuestAppearedEvent, QuestCompletedEvent, QuestMissedEvent};
use bevy::math::{ivec3, vec2, vec3};
use bevy::prelude::*;
use bevy::text::Text2dBounds;
use bevy_easings::{custom_ease_system, CustomComponentEase, EaseFunction, EasingType, Lerp};
use bevy_simple_tilemap::plugin::SimpleTileMapPlugin;
use bevy_simple_tilemap::prelude::*;
use chrono::{DateTime, Duration, Local};

pub struct QuestPanelPlugin;

impl Plugin for QuestPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimpleTileMapPlugin)
            .add_startup_system(setup_quest_panel)
            .add_system(custom_ease_system::<QuestPanel>)
            .add_system(update_quest_panel_content)
            .add_system(update_quest_panel_ui)
            .add_system(handle_quest_events);
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct TiledPanel {
    pub tiled_size: IVec2,
    pub top_left_tile: u32,
    pub top_right_tile: u32,
    pub top_tile: u32,
    pub left_tile: u32,
    pub right_tile: u32,
    pub fill_tile: u32,
}

#[derive(Component, Default)]
struct QuestPanel {
    pub draggable: bool,
    pub expansion_fraction: f32,
    pub dragged_from: Option<Vec2>,
    pub tiles_info: TiledPanel,
}

#[derive(Component, Default, Clone, Copy)]
struct QuestHeader;

#[derive(Component, Default, Clone, Copy)]
struct QuestDescription;

#[derive(Component, Default, Clone, Copy)]
struct QuestButton {
    is_pressed: bool,
}

impl Lerp for QuestPanel {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        QuestPanel {
            expansion_fraction: self
                .expansion_fraction
                .lerp(&other.expansion_fraction, scalar),
            ..*other
        }
    }
}

fn generate_panel_tilemap(tiles_info: TiledPanel) -> TileMap {
    let mut tiles = Vec::<(IVec3, Option<Tile>)>::new();

    let tiled_x = tiles_info.tiled_size.x;
    tiles.push((
        ivec3(-tiled_x, 0, 0),
        Some(Tile {
            sprite_index: tiles_info.top_left_tile,
            ..default()
        }),
    ));
    tiles.push((
        ivec3(tiled_x, 0, 0),
        Some(Tile {
            sprite_index: tiles_info.top_right_tile,
            ..default()
        }),
    ));
    for x in (-tiled_x + 1)..=(tiled_x - 1) {
        tiles.push((
            ivec3(x, 0, 0),
            Some(Tile {
                sprite_index: tiles_info.top_tile,
                ..default()
            }),
        ));
        for y in 1..tiles_info.tiled_size.y {
            tiles.push((
                ivec3(x, -y, 0),
                Some(Tile {
                    sprite_index: tiles_info.fill_tile,
                    ..default()
                }),
            ));
        }
    }
    for y in 1..tiles_info.tiled_size.y {
        tiles.push((
            ivec3(-tiled_x, -y, 0),
            Some(Tile {
                sprite_index: tiles_info.left_tile,
                ..default()
            }),
        ));
        tiles.push((
            ivec3(tiled_x, -y, 0),
            Some(Tile {
                sprite_index: tiles_info.right_tile,
                ..default()
            }),
        ));
    }

    let mut tilemap = TileMap::default();
    tilemap.set_tiles(tiles);
    return tilemap;
}

fn setup_quest_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let panel_texture_handle = asset_server.load("sprites/8x8_quest_panel.png");
    let button_texture_handle = asset_server.load("sprites/button.png");
    let font = asset_server.load("fonts/at01.ttf");

    let header_text_style = TextStyle {
        font: font.clone(),
        font_size: 130.0,
        color: Color::BLACK,
    };
    let description_text_style = TextStyle {
        font,
        font_size: 100.0,
        color: Color::DARK_GRAY,
    };

    let tile_size = 8.0;

    let panel_atlas = TextureAtlas::from_grid(
        panel_texture_handle,
        vec2(tile_size, tile_size),
        3,
        3,
        None,
        None,
    );
    let panel_atlas_handle = texture_atlases.add(panel_atlas);

    let button_atlas =
        TextureAtlas::from_grid(button_texture_handle, vec2(32.0, 16.0), 1, 2, None, None);
    let button_atlas_handle = texture_atlases.add(button_atlas);

    let tiles_info = TiledPanel {
        tiled_size: IVec2::new(3, 7),
        top_left_tile: 0,
        top_right_tile: 2,
        top_tile: 1,
        left_tile: 3,
        right_tile: 5,
        fill_tile: 4,
    };

    let text_scale = 0.05;
    let panel_tiled_width = tiles_info.tiled_size.x * 2 + 1;
    let panel_tiled_height = tiles_info.tiled_size.y;
    let text_box_width = tile_size * (panel_tiled_width as f32 - 1.0);
    let text_box_height = tile_size * (panel_tiled_height as f32);
    let text_box_scaled_width = text_box_width / text_scale;
    let text_box_scaled_height = text_box_height / text_scale;

    let text_top_margin = tile_size * 0.4;

    commands
        .spawn(TileMapBundle {
            texture_atlas: panel_atlas_handle,
            tilemap: generate_panel_tilemap(tiles_info),
            ..Default::default()
        })
        .insert(QuestPanel {
            tiles_info,
            ..default()
        })
        .insert(Name::from("Panel"))
        .with_children(|parent| {
            parent
                .spawn(Text2dBundle {
                    text: Text::from_section("Header", header_text_style.clone())
                        .with_alignment(TextAlignment::CENTER),
                    transform: Transform {
                        translation: vec3(0.0, -text_top_margin, 1.0),
                        rotation: Quat::default(),
                        scale: Vec3::splat(text_scale),
                    },
                    ..default()
                })
                .insert(QuestHeader);
            parent
                .spawn(Text2dBundle {
                    text: Text::from_section("Description", description_text_style.clone())
                        .with_alignment(TextAlignment::TOP_LEFT),
                    transform: Transform {
                        translation: vec3(
                            -text_box_width / 2.0,
                            -tile_size / 2.0 - text_top_margin * 1.5,
                            1.0,
                        ),
                        rotation: Quat::default(),
                        // scale: Vec3::splat(1.0)
                        scale: Vec3::splat(text_scale),
                    },
                    text_2d_bounds: Text2dBounds {
                        size: vec2(text_box_scaled_width, text_box_scaled_height),
                    },
                    ..default()
                })
                .insert(QuestDescription);

            let button_pos = vec3(0.0, -text_box_height + tile_size * 2.0, 1.0);
            parent
                .spawn(SpriteSheetBundle {
                    texture_atlas: button_atlas_handle,
                    transform: Transform::from_translation(button_pos),
                    ..default()
                })
                .insert(QuestButton::default());
        });
}

fn cursor_to_world(window: &Window, cam_transform: &Transform, cursor_pos: Vec2) -> Vec2 {
    // get the size of the window
    let size = Vec2::new(window.width() as f32, window.height() as f32);

    // the default orthographic projection is in pixels from the center;
    // just undo the translation
    let screen_pos = cursor_pos - size / 2.0;

    // apply the camera transform
    let out = cam_transform.compute_matrix() * screen_pos.extend(0.0).extend(1.0);
    Vec2::new(out.x, out.y)
}

fn format_duration(duration: Duration) -> String {
    if duration >= Duration::days(365) {
        let years = duration.num_days() / 365;
        years.to_string() + " years"
    } else if duration >= Duration::weeks(1) {
        duration.num_weeks().to_string() + "w"
    } else if duration >= Duration::days(1) {
        duration.num_days().to_string() + "d"
    } else if duration >= Duration::hours(1) {
        duration.num_days().to_string() + "h"
    } else {
        duration.num_seconds().max(0).to_string() + "s"
    }
}

fn update_quest_panel_content(
    mut headers: Query<&mut Text, (With<QuestHeader>, Without<QuestDescription>)>,
    mut descriptions: Query<&mut Text, (With<QuestDescription>, Without<QuestHeader>)>,
    current_quest: Res<CurrentQuestInfo>,
) {
    let mut header_text = headers.single_mut();
    let mut description_text = descriptions.single_mut();

    if let Some(quest) = current_quest.current_quest.as_ref() {
        let time_remaining = quest.deadline - DateTime::from(Local::now());
        let title = quest.quest.name.clone();
        let description = quest.quest.description.clone();

        header_text.sections.first_mut().unwrap().value =
            title + " in " + format_duration(time_remaining).as_str();
        description_text.sections.first_mut().unwrap().value = description;
    }
}

fn handle_quest_events(
    mut commands: Commands,
    mut panel: Query<(Entity, &mut QuestPanel)>,
    quest_completed_events: EventReader<QuestCompletedEvent>,
    quest_missed_events: EventReader<QuestMissedEvent>,
    quest_appeared_events: EventReader<QuestAppearedEvent>,
) {
    let (panel_entity, mut panel) = panel.single_mut();

    let quest_completed = !quest_completed_events.is_empty();
    let quest_missed = !quest_missed_events.is_empty();
    let quest_appeared = !quest_appeared_events.is_empty();

    if quest_appeared {
        panel.draggable = true;
        animate_panel(&mut commands, panel_entity, &panel, -0.2, 0.0);
    } else if quest_completed || quest_missed {
        panel.draggable = false;
        animate_panel(
            &mut commands,
            panel_entity,
            &panel,
            panel.expansion_fraction,
            -0.2,
        );
    }
}

fn animate_panel(
    commands: &mut Commands,
    panel_entity: Entity,
    panel: &QuestPanel,
    start: f32,
    finish: f32,
) {
    commands.entity(panel_entity).insert(
        QuestPanel {
            expansion_fraction: start,
            ..*panel
        }
        .ease_to(
            QuestPanel {
                expansion_fraction: finish,
                ..*panel
            },
            EaseFunction::QuadraticIn,
            EasingType::Once {
                duration: std::time::Duration::from_millis(500),
            },
        ),
    );
}

fn panel_y_from_expansion_fraction(
    panel_height: f32,
    window_height: f32,
    tile_height: f32,
    expansion_fraction: f32,
) -> f32 {
    let add_top_margin = tile_height * 0.5;
    return (panel_height - tile_height - add_top_margin) * expansion_fraction
        + add_top_margin
        + tile_height * 0.5
        - window_height * 0.5;
}

fn expansion_fraction_from_panel_y(
    panel_height: f32,
    window_height: f32,
    tile_height: f32,
    panel_y: f32,
) -> f32 {
    let add_top_margin = tile_height * 0.5;
    return (panel_y + window_height * 0.5 - tile_height * 0.5 - add_top_margin)
        / (panel_height - tile_height - add_top_margin);
}

fn update_quest_panel_ui(
    mut panels: Query<(&mut Transform, &mut QuestPanel)>,
    cameras: Query<&Transform, (With<Camera>, Without<QuestPanel>)>,
    mut buttons: Query<(&GlobalTransform, &mut QuestButton, &mut TextureAtlasSprite)>,
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut quest_completed_events: EventWriter<QuestCompletedEvent>,
) {
    let window = windows.get_primary().unwrap();
    let cam_transform = cameras.single();

    let (panel_transform, panel) = panels.single_mut();
    let mut panel_transform: Mut<Transform> = panel_transform;
    let mut panel: Mut<QuestPanel> = panel;

    let (button_global_transform, button, button_sprite) = buttons.single_mut();
    let button_global_transform: &GlobalTransform = button_global_transform;
    let mut button: Mut<QuestButton> = button;
    let mut button_sprite: Mut<TextureAtlasSprite> = button_sprite;

    // println!("{:?}", panel.expansion_fraction);

    let tile_size = 8.0;
    let panel_tiled_width = panel.tiles_info.tiled_size.x * 2 + 1;
    let panel_width = window.width().min(window.height() * 0.7);
    let panel_scale = panel_width / tile_size / (panel_tiled_width as f32);
    panel_transform.scale = Vec3::splat(panel_scale);

    let panel_height = tile_size * panel_scale * (panel.tiles_info.tiled_size.y as f32);
    panel_transform.translation.y = panel_y_from_expansion_fraction(
        panel_height,
        window.height(),
        tile_size * panel_scale,
        panel.expansion_fraction,
    );
    panel_transform.translation.z = 5.0;

    if !mouse_buttons.pressed(MouseButton::Left) || !panel.draggable {
        panel.dragged_from = None;
    }

    if let Some(cursor_pos) = window.cursor_position() {
        let cursor_pos = cursor_to_world(window, cam_transform, cursor_pos);

        let button_pos = button_global_transform.translation();

        if mouse_buttons.just_pressed(MouseButton::Left) {
            let panel_top = panel_transform.translation.y + tile_size * panel_scale / 2.0;
            let panel_content_top = panel_top - tile_size * panel_scale;
            if cursor_pos.y < panel_top && cursor_pos.y > panel_content_top {
                panel.dragged_from = Some(cursor_pos);
            }

            if cursor_pos.y < button_pos.y + panel_scale * 16.0 / 2.0
                && cursor_pos.y > button_pos.y - panel_scale * 16.0 / 2.0
                && cursor_pos.x < button_pos.x + panel_scale * 32.0 / 2.0
                && cursor_pos.x > button_pos.x - panel_scale * 32.0 / 2.0
                && !button.is_pressed
            {
                button.is_pressed = true;
                button_sprite.index = 1;
            }
        } else if let Some(dragged_from) = panel.dragged_from.as_mut() {
            let max_y = panel_y_from_expansion_fraction(
                panel_height,
                window.height(),
                tile_size * panel_scale,
                1.0,
            );
            let min_y = panel_y_from_expansion_fraction(
                panel_height,
                window.height(),
                tile_size * panel_scale,
                0.0,
            );
            panel_transform.translation.y += cursor_pos.y - dragged_from.y;
            panel_transform.translation.y = panel_transform.translation.y.min(max_y).max(min_y);
            dragged_from.y = cursor_pos.y;
            panel.expansion_fraction = expansion_fraction_from_panel_y(
                panel_height,
                window.height(),
                tile_size * panel_scale,
                panel_transform.translation.y,
            );
        }
    } else {
        panel.dragged_from = None;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if button.is_pressed {
            button.is_pressed = false;
            button_sprite.index = 0;
            quest_completed_events.send(QuestCompletedEvent);
        }
    }
}
