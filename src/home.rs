use bevy::prelude::*;
use crate::home::quest_panel::QuestPanelPlugin;
use crate::NORMAL_BUTTON;
use super::data::*;

mod quest_panel;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(home_setup)
            .add_plugin(QuestPanelPlugin);
    }
}

fn setup_tree(commands: &mut Commands,
              asset_server: &Res<AssetServer>,
              texture_atlases: &mut ResMut<Assets<TextureAtlas>>) {
    let texture_handle = asset_server.load("sprites/48x48_trees.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        });
}

fn home_setup(mut commands: Commands,
              asset_server: Res<AssetServer>,
              mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    setup_tree(&mut commands, &asset_server, &mut texture_atlases);
}