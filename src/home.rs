use bevy::prelude::*;

mod proc_tree;
mod quest_panel;

use proc_tree::ProcTreePlugin;
use quest_panel::QuestPanelPlugin;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuestPanelPlugin).add_plugin(ProcTreePlugin);
    }
}
