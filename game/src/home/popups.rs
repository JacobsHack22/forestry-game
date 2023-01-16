use crate::data::{QuestCompletedEvent, QuestMissedEvent};
use bevy::app::{App, Plugin};
use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy_easings::{Ease, EaseFunction, EasingType};

pub struct PopupsPlugin;

impl Plugin for PopupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PopupsPluginData::default())
            .add_startup_system(setup_popups)
            .add_system(handle_quest_events);
    }
}

#[derive(Default, Resource)]
struct PopupsPluginData {
    pub good_popup_handle: Handle<Image>,
    pub bad_popup_handle: Handle<Image>,
}

fn setup_popups(asset_server: Res<AssetServer>, mut proc_tree_system: ResMut<PopupsPluginData>) {
    proc_tree_system.good_popup_handle = asset_server.load("sprites/heart.png");
    proc_tree_system.bad_popup_handle = asset_server.load("sprites/emote_broken_heart.png");
}

fn handle_quest_events(
    mut commands: Commands,
    quest_completed_events: EventReader<QuestCompletedEvent>,
    quest_missed_events: EventReader<QuestMissedEvent>,
    proc_tree_system: Res<PopupsPluginData>,
) {
    let quest_completed = !quest_completed_events.is_empty();
    let quest_missed = !quest_missed_events.is_empty();

    if quest_completed {
        spawn_popup(
            &mut commands,
            proc_tree_system.good_popup_handle.clone(),
            vec2(-50., 0.),
            6.0,
        );
        spawn_popup(
            &mut commands,
            proc_tree_system.good_popup_handle.clone(),
            vec2(50., 60.),
            6.0,
        );
        spawn_popup(
            &mut commands,
            proc_tree_system.good_popup_handle.clone(),
            vec2(30., -40.),
            6.0,
        );
    } else if quest_missed {
        spawn_popup(
            &mut commands,
            proc_tree_system.bad_popup_handle.clone(),
            vec2(0., 0.),
            1.0,
        );
    }
}

fn spawn_popup(
    commands: &mut Commands,
    popup_texture_handle: Handle<Image>,
    position: Vec2,
    scale: f32,
) {
    commands
        .spawn(SpriteBundle {
            texture: popup_texture_handle,
            ..default()
        })
        .insert(
            Transform {
                translation: position.extend(0.0) + vec3(0.0, 0.0, 1.0),
                rotation: Quat::default(),
                scale: Vec3::splat(1.0 * scale),
            }
            .ease_to(
                Transform {
                    translation: position.extend(0.0) + vec3(0.0, 50.0, 1.0),
                    rotation: Quat::default(),
                    scale: Vec3::splat(1.0 * scale),
                },
                EaseFunction::QuadraticIn,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(500),
                },
            ),
        )
        .insert(
            Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                ..default()
            }
            .ease_to(
                Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                },
                EaseFunction::QuadraticIn,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(500),
                },
            ),
        );
}
