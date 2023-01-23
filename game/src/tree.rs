use crate::data::{CurrentTree, TreeInfo};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;

mod gen;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TreePluginData::default())
            .add_plugin(DebugLinesPlugin::default())
            .add_startup_system(setup_tree)
            .add_system(update_tree_structure)
            .add_system(draw_trees);
    }
}

#[derive(Component)]
struct Tree;

#[derive(Default, Resource)]
struct TreePluginData {
    tree_structure: gen::TreeStructure,
}

fn setup_tree(mut commands: Commands) {
    commands
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(Tree);
}


fn update_tree_structure(
    mut data: ResMut<TreePluginData>,
    current_tree: Res<CurrentTree>,
    trees_info: Query<(&TreeInfo, ChangeTrackers<TreeInfo>)>,
) {
    let (current_tree_info, change_trackers) = trees_info.get(current_tree.0).unwrap();
    if change_trackers.is_changed() || current_tree.is_changed() {
        data.tree_structure = gen::generate(current_tree_info.clone());
    }
}

fn draw_trees(
    data: ResMut<TreePluginData>,
    tree_transforms: Query<&Transform, With<Tree>>,
    mut lines: ResMut<DebugLines>,
) {
    for transform in tree_transforms.iter() {
        draw_tree(&data.tree_structure.root, *transform, &mut lines);
    }
}

fn draw_tree(node: &gen::TreeNode, transform: Transform, lines: &mut DebugLines) {
    let children = [node.main_branch.as_deref(), node.lateral_branch.as_deref()];
    for child in children.iter() {
        if let Some(child) = child {
            let current_pos = transform.transform_point(node.global_position);
            let child_pos = transform.transform_point(child.global_position);
            lines.line(current_pos, child_pos, 0.0);
            draw_tree(child, transform, lines);
        }
    }
}
