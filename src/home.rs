use bevy::prelude::*;
use crate::home::proc_tree::ProcTreePlugin;
use crate::home::quest_panel::QuestPanelPlugin;

mod quest_panel;
mod proc_tree;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(home_setup)
            .add_plugin(QuestPanelPlugin)
            .add_plugin(ProcTreePlugin);
    }
}

fn home_setup(mut commands: Commands,
              asset_server: Res<AssetServer>,
              mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    // setup_tree(&mut commands, &asset_server, &mut texture_atlases);
}