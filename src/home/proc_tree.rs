use crate::data::{CurrentTree, Health, QuestCompletedEvent, QuestMissedEvent, TreeInfo};
use bevy::app::{App, Plugin};
use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy_easings::{Ease, EaseFunction, EasingType};

pub struct ProcTreePlugin;

impl Plugin for ProcTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<CustomMaterial>::default())
            .insert_resource(ProcTreeSystem::default())
            .add_startup_system(setup_proc_tree)
            .add_startup_system(setup_proc_tree_plugin)
            .add_system(update_proc_tree)
            .add_system(handle_quest_events);
    }
}

#[derive(Component)]
struct ProcTree;

#[derive(Default)]
struct ProcTreeSystem {
    pub good_popup_handle: Handle<Image>,
    pub bad_popup_handle: Handle<Image>,
}

fn setup_proc_tree_plugin(
    asset_server: Res<AssetServer>,
    mut proc_tree_system: ResMut<ProcTreeSystem>,
) {
    proc_tree_system.good_popup_handle = asset_server.load("sprites/heart.png");
    proc_tree_system.bad_popup_handle = asset_server.load("sprites/emote_broken_heart.png");
}

fn setup_proc_tree(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/season-trees-spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 40, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        })
        .insert(ProcTree);
}

fn update_proc_tree(
    mut proc_trees: Query<&mut TextureAtlasSprite, With<ProcTree>>,
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

fn handle_quest_events(
    mut commands: Commands,
    quest_completed_events: EventReader<QuestCompletedEvent>,
    quest_missed_events: EventReader<QuestMissedEvent>,
    proc_tree_system: Res<ProcTreeSystem>,
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
        .spawn_bundle(SpriteBundle {
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

#[derive(AsBindGroup, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/proc_tree.wgsl".into()
    }
}
