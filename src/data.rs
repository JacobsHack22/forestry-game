use std::ops::Add;
use std::time::SystemTime;
use bevy::app::{App, Plugin};
use bevy::prelude::{Bundle, Commands, Component, Entity};
use chrono::{DateTime, Local, Duration, FixedOffset};

pub enum TreeKind {
    Birch,
    Oak,
}

#[derive(Component)]
pub struct TreeInfo {
    pub name: String,
    pub seed: u64,
    pub health: u8,
    pub kind: TreeKind,
}

impl Default for TreeInfo {
    fn default() -> Self {
        TreeInfo {
            name: "John".to_string(),
            seed: 0,
            health: 5,
            kind: TreeKind::Oak,
        }
    }
}

pub struct Quest {
    pub name: String,
    pub description: String,
    pub time_to_complete: Duration,
}

pub struct ActiveQuest {
    pub quest: Quest,
    pub deadline: DateTime<Local>,
}

#[derive(Bundle, Default)]
pub struct TreeItem {
    pub info: TreeInfo,
}

pub struct CurrentQuest {
    pub quest: Option<ActiveQuest>
}

impl Default for CurrentQuest {
    fn default() -> Self {
        CurrentQuest {
            quest: Some(ActiveQuest {
                quest: Quest {
                    name: "Tidy up".to_string(),
                    description: "Clean your place after submitting your JacobsHack entry".to_string(),
                    time_to_complete: Duration::hours(1)
                },
                deadline: DateTime::from(Local::now()).add(Duration::hours(1))
            })
        }
    }
}

pub struct CurrentTree(Entity);

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(data_setup);
    }
}

fn data_setup(mut commands: Commands) {
    let default_tree = commands
        .spawn_bundle(TreeItem::default())
        .id();

    commands.insert_resource(CurrentTree(default_tree));
    commands.insert_resource(CurrentQuest::default())
}