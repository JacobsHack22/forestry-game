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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
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
