use std::collections::VecDeque;
use bevy::prelude::*;
use std::ops::Add;
use std::time::SystemTime;
use bevy::app::{App, Plugin};
use chrono::{DateTime, Local, Duration, FixedOffset, Utc};

pub enum TreeKind {
    Birch,
    Oak,
}

#[derive(Default)]
pub enum Health {
    Bad,
    Moderate,
    #[default]
    Good,
}

impl Health {
    pub fn decrement(&mut self) {
        *self = match self {
            Health::Bad => Health::Bad,
            Health::Moderate => Health::Bad,
            Health::Good => Health::Moderate
        };
    }

    pub fn increment(&mut self) {
        *self = match self {
            Health::Bad => Health::Moderate,
            Health::Moderate => Health::Good,
            Health::Good => Health::Good
        };
    }

    pub fn restore(&mut self) {
        *self = Health::Good;
    }
}

#[derive(Component)]
pub struct TreeInfo {
    pub name: String,
    pub seed: u64,
    pub health: Health,
    pub kind: TreeKind,
}

impl Default for TreeInfo {
    fn default() -> Self {
        TreeInfo {
            name: "John".to_string(),
            seed: 0,
            health: Health::default(),
            kind: TreeKind::Oak,
        }
    }
}

#[derive(Clone)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub time_to_complete: Duration,
}

#[derive(Clone)]
pub struct ActiveQuest {
    pub quest: Quest,
    pub deadline: DateTime<Utc>,
}

impl From<Quest> for ActiveQuest {
    fn from(quest: Quest) -> Self {
        ActiveQuest {
            quest: quest.clone(),
            deadline: DateTime::<Utc>::from(Local::now()) + quest.time_to_complete,
        }
    }
}

#[derive(Clone, Default)]
pub struct QuestPool {
    pub queue: VecDeque<Quest>,
}

#[derive(Default)]
pub struct CurrentQuestInfo {
    pub current_quest: Option<ActiveQuest>,
    pub last_quest_finished: DateTime<Utc>,
}

#[derive(Bundle, Default)]
pub struct TreeItem {
    pub info: TreeInfo,
}

pub struct CurrentTree(pub Entity);

pub struct QuestAppearedEvent;

pub struct QuestCompletedEvent;

pub struct QuestMissedEvent;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(current_tree_setup)
            .insert_resource(CurrentQuestInfo::default())
            .insert_resource(QuestPool::default())
            .add_startup_system(fill_quest_pool)
            .add_event::<QuestCompletedEvent>()
            .add_event::<QuestMissedEvent>()
            .add_event::<QuestAppearedEvent>()
            .add_system(handle_events)
            .add_system(check_deadline)
            .add_system(check_next_quest);
    }
}

fn current_tree_setup(mut commands: Commands) {
    let default_tree = commands
        .spawn_bundle(TreeItem::default())
        .id();

    commands.insert_resource(CurrentTree(default_tree));
}

fn handle_events(
    mut quest_completed_events: EventReader<QuestCompletedEvent>,
    mut quest_missed_events: EventReader<QuestMissedEvent>,
    mut current_quest_info: ResMut<CurrentQuestInfo>,
    current_tree: Res<CurrentTree>,
    mut tree_items: Query<&mut TreeInfo>,
) {
    let mut current_tree_item = tree_items.get_mut(current_tree.0).unwrap();
    let quest_completed = !quest_completed_events.is_empty();
    let quest_missed = !quest_missed_events.is_empty();

    if quest_completed || quest_missed {
        current_quest_info.current_quest = None;
        current_quest_info.last_quest_finished = DateTime::from(Utc::now());

        if quest_completed {
            current_tree_item.health.increment()
        } else {
            current_tree_item.health.decrement()
        }
    }
}

fn check_deadline(
    mut current_quest: ResMut<CurrentQuestInfo>,
    mut quest_missed_events: EventWriter<QuestMissedEvent>,
) {
    if let Some(active_quest) = current_quest.current_quest.as_ref() {
        if DateTime::<Utc>::from(Local::now()) > active_quest.deadline {
            quest_missed_events.send(QuestMissedEvent)
        }
    }
}

fn check_next_quest(
    mut current_quest_info: ResMut<CurrentQuestInfo>,
    mut quest_appeared_events: EventWriter<QuestAppearedEvent>,
    mut quest_pool: ResMut<QuestPool>,
) {
    let since_last_quest_finished =
        DateTime::from(Utc::now()) - current_quest_info.last_quest_finished;
    if current_quest_info.current_quest.is_none() &&
        since_last_quest_finished > Duration::seconds(5) {
        if let Some(quest) = quest_pool.queue.pop_front() {
            current_quest_info.current_quest = Some(quest.into());
            quest_appeared_events.send(QuestAppearedEvent);
        }
    }
}

fn fill_quest_pool(
    mut quest_pool: ResMut<QuestPool>,
    mut current_quest_info: ResMut<CurrentQuestInfo>,
    mut quest_appeared_events: EventWriter<QuestAppearedEvent>,
) {
    quest_pool.queue.push_back(Quest {
        name: "Test".to_string(),
        description: "... test".to_string(),
        time_to_complete: Duration::seconds(30),
    });
    quest_pool.queue.push_back(Quest {
        name: "Cheer up".to_string(),
        description: "No ecological revolution can be done in a bad mood".to_string(),
        time_to_complete: Duration::seconds(8),
    });
    quest_pool.queue.push_back(Quest {
        name: "Tidy up".to_string(),
        description: "Clean your place after submitting your JacobsHack entry".to_string(),
        time_to_complete: Duration::seconds(30),
    });
    current_quest_info.current_quest = Some(quest_pool.queue.pop_front().unwrap().into());
    quest_appeared_events.send(QuestAppearedEvent);
}
