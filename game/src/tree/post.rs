use bevy::prelude::*;
use bevy_easings::Lerp;

use super::gen::{TreeNode, TreeStructure};

fn subdivide_branch(
    parent: Option<&TreeNode>,
    current: &TreeNode,
    child: TreeNode,
    subdivisions_count: usize,
) -> TreeNode {
    let child_pos = child.global_position;
    let child_width = child.width;

    let mut new_current = child;
    for t in 1..subdivisions_count {
        let fraction = t as f32 / subdivisions_count as f32;

        let direction = parent.map_or(Vec3::Y, |parent| {
            current.global_position - parent.global_position
        });
        let knee_pos = current.global_position + direction / 2.0;

        let p1 = child_pos.lerp(knee_pos, fraction);
        let p2 = knee_pos.lerp(current.global_position, fraction);
        let p3 = p1.lerp(p2, fraction);

        new_current = TreeNode {
            global_position: p3,
            width: child_width.lerp(&current.width, &fraction),
            main_branch: Some(Box::new(new_current)),
            lateral_branch: None,
        }
    }
    new_current
}

fn subdivide_recursive(
    parent: Option<&TreeNode>,
    current: &TreeNode,
    subdivisions_count: usize,
) -> TreeNode {
    let main_branch = current.main_branch.as_ref().map(|child| {
        let child = subdivide_recursive(Some(current), child, subdivisions_count);
        Box::new(subdivide_branch(parent, current, child, subdivisions_count))
    });
    let lateral_branch = current.lateral_branch.as_ref().map(|child| {
        let child = subdivide_recursive(Some(current), child, subdivisions_count);
        Box::new(subdivide_branch(parent, current, child, subdivisions_count))
    });
    TreeNode {
        main_branch,
        lateral_branch,
        ..*current
    }
}

pub fn subdivide(tree: &TreeStructure, subdivisions_count: usize) -> TreeStructure {
    TreeStructure {
        root: subdivide_recursive(None, &tree.root, subdivisions_count),
    }
}
