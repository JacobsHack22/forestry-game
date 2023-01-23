use bevy::prelude::*;
use rand::{RngCore, SeedableRng, Rng, rngs::StdRng, seq::SliceRandom};

use crate::data::TreeInfo;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct SeedStructure {
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
    pub full_light_exposure: f32,
    pub base_branch_width: f32,

    pub tropism_weight: f32,
    pub current_direction_weight: f32,
    pub optimal_growth_direction_weight: f32,

    pub environment_size: u64,
    pub environment_points_count: u32,
    pub iterations_count: u32,
}

impl From<TreeInfo> for SeedStructure {
    fn from(tree_info: TreeInfo) -> Self {
        SeedStructure {
            seed: tree_info.seed,
            iterations_count: 5,
            ..default()
        }
    }
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

#[derive(Debug)]
pub enum BudFate {
    Dormant,
    Shoot,
    Dead,
    Flower,
    Leaf,
}

impl Default for BudFate {
    fn default() -> Self {
        BudFate::Dormant
    }
}

#[derive(Default, Debug)]
pub struct Bud {
    pub direction: Vec3,
    pub bud_id: BudId,
    pub branch_node: Option<Box<MetamerNode>>,
    pub bud_fate: BudFate,
}

#[derive(Default, Debug)]
pub struct MetamerNode {
    pub global_position: Vec3,
    pub width: f32,
    pub main_bud: Box<Bud>,
    pub axillary_bud: Box<Bud>
}

impl MetamerNode {
    pub fn distance_to(&self, environment_point: &Vec3) -> f32 {
        f32::sqrt( (self.global_position.x - environment_point.x).powi(2) + 
                    (self.global_position.y - environment_point.y).powi(2) +
                    (self.global_position.z - environment_point.z).powi(2) )
    }
}

impl From<Box<MetamerNode>> for TreeNode {
    fn from(node: Box<MetamerNode>) -> Self {
        TreeNode {
            global_position: node.global_position,
            width: node.width,
            main_branch: match node.main_bud.branch_node {
                Some(branch) => Some(Box::new(TreeNode::from(branch))),
                None => None,
            },
            lateral_branch: match node.axillary_bud.branch_node {
                Some(branch) => Some(Box::new(TreeNode::from(branch))),
                None => None,
            },
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct BudId(usize);

#[derive(Default, Debug)]
pub struct Environment {
    pub points: Vec<Vec3>,
    pub bud_id_counter: BudId,
}

impl Environment {
    pub fn get_next_bud_id(&mut self) -> BudId {
        let tmp = self.bud_id_counter;
        self.bud_id_counter.0 += 1;
        tmp
    }  

    pub fn get_number_of_buds(&self) -> usize {
        self.bud_id_counter.0
    }
}

pub fn generate_environment(seed: u64, environment_size: u64, environment_points_count: u32) -> Environment {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut points = vec![];

    for _ in 0..environment_points_count {
        let x = rng.gen_range(-(environment_size as f32)..=environment_size as f32);
        let y = rng.gen_range(-(environment_size as f32)..=environment_size as f32);
        let z = rng.gen_range(-(environment_size as f32)..=environment_size as f32);

        points.push(Vec3::new(x as f32, y as f32, z as f32));
    }

    Environment { points, bud_id_counter: BudId(0) }
}

#[derive(Default, Debug, Clone)]
pub struct BudLocalEnvironment {
    pub optimal_growth_direction: Vec3,
    pub resource: f32,
    pub light_exposure: f32,
}

pub fn find_candidates_for_environment_point(node: &Box<MetamerNode>, associated_bud_candidates: &mut Vec<BudId>, current_minimal_distance: &mut f32, environment_point: &Vec3) {
    for bud in [&node.main_bud, &node.axillary_bud] {
        match bud.bud_fate {
            BudFate::Dormant => {
                let distance = node.distance_to(environment_point);
                if associated_bud_candidates.is_empty() || distance < *current_minimal_distance {
                    *current_minimal_distance = distance;
                    associated_bud_candidates.clear();
                    associated_bud_candidates.push(bud.bud_id);
                } else if distance == *current_minimal_distance {
                    associated_bud_candidates.push(bud.bud_id);
                }
            },
            BudFate::Shoot => {
                find_candidates_for_environment_point(bud.branch_node.as_ref().expect("Shoot should have branch node"),
                 associated_bud_candidates, current_minimal_distance, environment_point)
            }
            _ => (),
        }
    }
}

pub fn calculate_optimal_growth_direction(bud_info: &mut Vec<BudLocalEnvironment>, environment: &Environment, root: &Box<MetamerNode>, rng: &mut StdRng) {
    let mut associated_sets: Vec<Vec<Vec3>> = vec![vec![]; environment.get_number_of_buds()];

    for environment_point in &environment.points {
        let mut associated_bud_candidates: Vec<BudId> = vec![];
        let mut current_minimal_distance = f32::MAX;
        find_candidates_for_environment_point(root, &mut associated_bud_candidates, &mut current_minimal_distance, environment_point);

        let associated_bud = associated_bud_candidates.choose(rng);

        if let Some(BudId(bud_id)) = associated_bud {
            associated_sets[*bud_id].push(*environment_point);
        }
    }

    for (i, set) in associated_sets.iter().enumerate() {
        bud_info[i].optimal_growth_direction = set.iter().sum::<Vec3>().normalize();
    }
}

pub fn calculate_local_environment(bud_info: &mut Vec<BudLocalEnvironment>, environment: &Environment, root: &Box<MetamerNode>, rng: &mut StdRng) {
    calculate_optimal_growth_direction(bud_info, environment, root, rng);
}

pub fn generate(tree_info: TreeInfo) -> TreeStructure {
    let args = SeedStructure::from(tree_info);

    let mut environment = generate_environment(args.seed, args.environment_size, args.environment_points_count);
    let mut rng = StdRng::seed_from_u64(args.seed);

    let mut root = Box::new(MetamerNode {
        global_position: Vec3::ZERO,
        width: args.base_branch_width,
        main_bud: Box::new(Bud {
            direction: Vec3::new(0.0, 1.0, 0.0),
            bud_id: environment.bud_id_counter,
            branch_node: None,
            bud_fate: BudFate::Dormant,
        }),
        axillary_bud: Box::new(Bud {
            direction: Vec3::ZERO,
            bud_id: BudId(0),
            branch_node: None,
            bud_fate: BudFate::Dead,
        })
    });

    for _ in 0..args.iterations_count {
       let mut bud_info: Vec<BudLocalEnvironment> = vec![BudLocalEnvironment::default(); environment.get_number_of_buds()];

       calculate_local_environment(&mut bud_info, &environment, &root, &mut rng);
    }

    TreeStructure { root: TreeNode::from(root) }
}
