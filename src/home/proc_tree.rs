use bevy::prelude::*;
use bevy::app::{App, Plugin};
use bevy::math::vec3;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::reflect::TypeUuid;
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy_easings::{Ease, EasingType, EaseFunction, EasingsPlugin};
use chrono::{DateTime, Utc};
use crate::data::{CurrentQuestInfo, CurrentTree, Health, QuestCompletedEvent, QuestMissedEvent, TreeInfo};

pub struct ProcTreePlugin;

impl Plugin for ProcTreePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(Material2dPlugin::<CustomMaterial>::default())
            .add_startup_system(setup_proc_tree)
            .add_system(update_proc_tree)
            .add_system(handle_quest_events);
    }
}

// fn setup_proc_tree(mut commands: Commands,
//                    asset_server: Res<AssetServer>,
//                    mut meshes: ResMut<Assets<Mesh>>,
//                    mut materials: ResMut<Assets<CustomMaterial>>,
//                    mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
//     let texture_handle = asset_server.load("sprites/48x48_trees.png");
//     let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 4, 1);
//     let texture_atlas_handle = texture_atlases.add(texture_atlas);
//     commands.spawn().insert_bundle(MaterialMesh2dBundle {
//         mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
//         transform: Transform::from_scale(Vec3::splat(400.0)),
//         material: materials.add(CustomMaterial {
//             color: Color::RED,
//             color_texture: Some(asset_server.load("sprites/48x48_trees.png")),
//             alpha_mode: AlphaMode::Blend,
//         }),
//         ..default()
//     });
// }

#[derive(Component)]
struct ProcTree;

fn setup_proc_tree(mut commands: Commands,
                   asset_server: Res<AssetServer>,
                   mut meshes: ResMut<Assets<Mesh>>,
                   mut materials: ResMut<Assets<CustomMaterial>>,
                   mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    let texture_handle = asset_server.load("sprites/48x48_trees.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 4, 1);
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
    mut current_tree: Res<CurrentTree>,
    mut trees_info: Query<&TreeInfo>,
) {
    let current_tree = trees_info.get(current_tree.0).unwrap();
    for mut tree_sprite in proc_trees.iter_mut() {
        tree_sprite.index =
            match current_tree.health {
                Health::Good => 1,
                Health::Moderate => 3,
                Health::Bad => 0
            }
    }
}

fn handle_quest_events(
    mut commands: Commands,
    mut quest_completed_events: EventReader<QuestCompletedEvent>,
    mut quest_missed_events: EventReader<QuestMissedEvent>,
    asset_server: Res<AssetServer>,
) {
    let good_popup_handle = asset_server.load("sprites/emote_heart.png");
    let bad_popup_handle = asset_server.load("sprites/emote_broken_heart.png");

    let quest_completed = !quest_completed_events.is_empty();
    let quest_missed = !quest_missed_events.is_empty();

    if quest_completed {
        spawn_popup(&mut commands, good_popup_handle);
    } else if quest_missed {
        spawn_popup(&mut commands, bad_popup_handle);
    }
}

fn spawn_popup(commands: &mut Commands, popup_texture_handle: Handle<Image>) {
    commands
        .spawn_bundle(
            SpriteBundle {
                texture: popup_texture_handle,
                ..default()
            },
        )
        .insert(
            Transform {
                translation: vec3(0.0, 0.0, 1.0),
                rotation: Quat::default(),
                scale: Vec3::splat(1.0),
            }.ease_to(
                Transform {
                    translation: vec3(0.0, 50.0, 1.0),
                    rotation: Quat::default(),
                    scale: Vec3::splat(1.0),
                },
                EaseFunction::QuadraticIn,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(500),
                },
            )
        )
        .insert(
            Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                ..default()
            }.ease_to(
                Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                },
                EaseFunction::QuadraticIn,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(500),
                },
            )
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