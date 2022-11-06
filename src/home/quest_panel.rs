use bevy::math::{ivec3, vec2, vec3};
use bevy::prelude::*;
use bevy_simple_tilemap::prelude::TileMapBundle;
use bevy_simple_tilemap::{Tile, TileMap};
use bevy_simple_tilemap::plugin::SimpleTileMapPlugin;

pub struct QuestPanelPlugin;

impl Plugin for QuestPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(SimpleTileMapPlugin)
            .add_startup_system(setup_quest_panel)
            .add_system(update_quest_panel);
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
    pub expansion_fraction: f32,
    pub dragged_from: Option<Vec2>,
    pub tiles_info: TiledPanel,
}

#[derive(Default)]
struct QuestPanelInfo {
    pub mouse_pos: Vec2,
}

fn generate_panel_tilemap(tiles_info: TiledPanel) -> TileMap {
    let mut tiles = Vec::<(IVec3, Option<Tile>)>::new();

    let tiled_x = tiles_info.tiled_size.x;
    tiles.push((
        ivec3(-tiled_x, 0, 0),
        Some(Tile { sprite_index: tiles_info.top_left_tile, ..default() })
    ));
    tiles.push((
        ivec3(tiled_x, 0, 0),
        Some(Tile { sprite_index: tiles_info.top_right_tile, ..default() })
    ));
    for x in (-tiled_x + 1)..=(tiled_x - 1) {
        tiles.push((
            ivec3(x, 0, 0),
            Some(Tile { sprite_index: tiles_info.top_tile, ..default() })
        ));
        for y in 1..tiles_info.tiled_size.y {
            tiles.push((
                ivec3(x, -y, 0),
                Some(Tile { sprite_index: tiles_info.fill_tile, ..default() })
            ));
        }
    }
    for y in 1..tiles_info.tiled_size.y {
        tiles.push((
            ivec3(-tiled_x, -y, 0),
            Some(Tile { sprite_index: tiles_info.left_tile, ..default() })
        ));
        tiles.push((
            ivec3(tiled_x, -y, 0),
            Some(Tile { sprite_index: tiles_info.right_tile, ..default() })
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
    let texture_handle = asset_server.load("sprites/8x8_quest_panel.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, vec2(8.0, 8.0), 3, 3);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let tiles_info = TiledPanel {
        tiled_size: IVec2::new(3, 6),
        top_left_tile: 0,
        top_right_tile: 2,
        top_tile: 1,
        left_tile: 3,
        right_tile: 5,
        fill_tile: 4,
    };

    commands
        .spawn_bundle(TileMapBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_scale(Vec3::splat(3.0)),
            tilemap: generate_panel_tilemap(tiles_info),
            ..Default::default()
        })
        .insert(QuestPanel {
            tiles_info,
            ..default()
        })
        .insert(Name::from("Panel"));
    commands.insert_resource(QuestPanelInfo {
        ..default()
    })
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

fn panel_y_from_expansion_fraction(
    panel_height: f32,
    window_height: f32,
    tile_height: f32,
    expansion_fraction: f32
) -> f32 {
    return (panel_height - tile_height) * expansion_fraction + tile_height * 0.5 - window_height * 0.5;
}

fn expansion_fraction_from_panel_y(
    panel_height: f32,
    window_height: f32,
    tile_height: f32,
    panel_y: f32
) -> f32 {
    return (panel_y + window_height * 0.5 - tile_height * 0.5) / (panel_height - tile_height);
}

fn update_quest_panel(
    mut panels: Query<(&mut Transform, &mut QuestPanel)>,
    cameras: Query<&Transform, (With<Camera>, Without<QuestPanel>)>,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
) {
    let window = windows.get_primary().unwrap();
    let cam_transform = cameras.single();

    let (panel_transform, panel) = panels.single_mut();
    let mut panel_transform: Mut<Transform> = panel_transform;
    let mut panel: Mut<QuestPanel> = panel;

    let tile_size = 8.0;
    let panel_tiled_width = panel.tiles_info.tiled_size.x * 2 + 1;
    let panel_width = window.width().min(window.height() * 0.5);
    let panel_scale = panel_width / tile_size / (panel_tiled_width as f32);
    panel_transform.scale = Vec3::splat(panel_scale);

    let panel_height = tile_size * panel_scale * (panel.tiles_info.tiled_size.y as f32);
    panel_transform.translation.y =
        panel_y_from_expansion_fraction(
            panel_height,
            window.height(),
            tile_size * panel_scale,
            panel.expansion_fraction,
        );
    // panel_transform.translation.z = 5.0;

    if !buttons.pressed(MouseButton::Left) {
        panel.dragged_from = None;
    }

    if let Some(cursor_pos) = window.cursor_position() {
        let cursor_pos = cursor_to_world(window, cam_transform, cursor_pos);

        if buttons.just_pressed(MouseButton::Left) {
            let panel_top = panel_transform.translation.y + tile_size * panel_scale / 2.0;
            if cursor_pos.y < panel_top {
                panel.dragged_from = Some(cursor_pos);
            }
        } else if let Some(dragged_from) = panel.dragged_from.as_mut() {
            let max_y = panel_y_from_expansion_fraction(
                panel_height,
                window.height(),
                tile_size * panel_scale,
                1.0
            );
            let min_y = panel_y_from_expansion_fraction(
                panel_height,
                window.height(),
                tile_size * panel_scale,
                0.0
            );
            panel_transform.translation.y += cursor_pos.y - dragged_from.y;
            panel_transform.translation.y = panel_transform.translation.y.min(max_y).max(min_y);
            dragged_from.y = cursor_pos.y;
            panel.expansion_fraction =
                expansion_fraction_from_panel_y(
                    panel_height,
                    window.height(),
                    tile_size * panel_scale,
                    panel_transform.translation.y,
                );
        }
    } else {
        panel.dragged_from = None;
    }
}