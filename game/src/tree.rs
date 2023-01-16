use crate::data::{CurrentTree, Health, TreeInfo};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TreePluginData::default())
            .add_startup_system(setup_tree)
            .add_system(update_tree);
    }
}

#[derive(Component)]
struct Tree;

#[derive(Default, Resource)]
struct TreePluginData {}

fn setup_tree(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/season-trees-spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 40, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        })
        .insert(Tree);
}

fn update_tree(
    mut proc_trees: Query<&mut TextureAtlasSprite, With<Tree>>,
    current_tree: Res<CurrentTree>,
    trees_info: Query<&TreeInfo>,
) {
    let current_tree = trees_info.get(current_tree.0).unwrap();
    for mut tree_sprite in proc_trees.iter_mut() {
        tree_sprite.index = match current_tree.health {
            Health::Good => 0,
            Health::Moderate => 24,
            Health::Bad => 32,
        }
    }
}
