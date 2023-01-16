use bevy::prelude::*;

mod popups;
mod quest_panel;

use crate::tree::TreePlugin;
use popups::PopupsPlugin;
use quest_panel::QuestPanelPlugin;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(QuestPanelPlugin)
            .add_plugin(PopupsPlugin)
            .add_plugin(TreePlugin);
    }
}
