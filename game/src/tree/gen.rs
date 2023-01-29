use bevy::prelude::*;
use rand::{RngCore, SeedableRng};

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct SeedStructure {
    pub seed: u64,
    pub main_branching_angle: f32,
    pub lateral_branching_angle: f32,
    pub apical_dominance: f32,
    pub bud_light_sensitivity: f32,
    pub branch_self_pruning: f32,
    pub maximum_shoot_lenght: f32,
    pub tropism_angle: f32,
    pub bud_perception_angle: f32,
    pub bud_perception_distance_coef: f32,
    pub occupancy_radius_coef: f32,
    pub resource_coef: f32,

    pub tropism_weight: f32,
    pub current_direction_weight: f32,
    pub optimal_growth_direction_weight: f32,

    pub iterations_count: u32,
}

#[derive(Default, Debug)]
pub struct TreeNode {
    pub global_position: Vec3,
    pub width: f32,
    pub main_branch: Option<Box<TreeNode>>,
    pub lateral_branch: Option<Box<TreeNode>>,
}

#[derive(Default, Debug)]
pub struct TreeStructure {
    pub root: TreeNode,
}

pub fn generate(args: SeedStructure) -> TreeStructure {
    let mut root = TreeNode {
        global_position: Vec3::ZERO,
        width: 0.7,
        main_branch: None,
        lateral_branch: None,
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(args.seed);

    let mut stack = vec![&mut root];

    for _ in 0..args.iterations_count {
        let mut new_stack = vec![];

        for node in stack {
            let main_x: f32 = ((((rng.next_u32() % 20) as i32) - 10) as f32) / 10.0;
            let main_z: f32 = ((((rng.next_u32() % 20) as i32) - 10) as f32) / 10.0;

            let main = TreeNode {
                global_position: node.global_position + Vec3::new(main_x, 1.0, main_z),
                width: node.width * 0.6,
                main_branch: None,
                lateral_branch: None,
            };

            let lateral_x: f32 = ((((rng.next_u32() % 20) as i32) - 10) as f32) / 10.0;
            let lateral_z: f32 = ((((rng.next_u32() % 20) as i32) - 10) as f32) / 10.0;

            let lateral = TreeNode {
                global_position: node.global_position + Vec3::new(lateral_x, 0.5, lateral_z),
                width: node.width * 0.5,
                main_branch: None,
                lateral_branch: None,
            };

            node.main_branch = Some(Box::new(main));
            node.lateral_branch = Some(Box::new(lateral));

            new_stack.push(node.main_branch.as_deref_mut().unwrap());
            new_stack.push(node.lateral_branch.as_deref_mut().unwrap());
        }

        stack = new_stack;
    }

    TreeStructure { root }
}
