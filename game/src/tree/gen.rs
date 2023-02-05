use std::f32::consts::PI;

use bevy::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

use crate::data::TreeInfo;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct SeedStructure {
    pub seed: u64,
    pub main_branching_angle: f32,
    pub lateral_branching_angle: f32,
    pub apical_dominance: f32,
    pub bud_light_sensitivity: f32,
    pub branch_self_pruning: f32,
    pub maximum_shoot_length: f32,
    pub tropism_angle: f32,
    pub bud_perception_angle: f32,
    pub bud_perception_distance_coef: f32,
    pub occupancy_radius_coef: f32,
    pub resource_coef: f32,
    pub full_light_exposure: f32,
    pub base_branch_width: f32,
    pub shadow_volume_angle: f32,
    pub shadow_adjustment_coef: f32,
    pub shadow_adjustment_base: f32,
    pub trunk_length: f32,

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
            main_branching_angle: 5.0 * PI / 180.0,
            lateral_branching_angle: 70.0 * PI / 180.0,
            apical_dominance: 0.5,
            bud_light_sensitivity: 1.0,
            branch_self_pruning: 0.05,
            maximum_shoot_length: 1.0,
            tropism_angle: -PI / 4.0,
            bud_perception_angle: PI / 4.0,
            bud_perception_distance_coef: 5.0,
            occupancy_radius_coef: 2.0,
            resource_coef: 0.5,
            full_light_exposure: 4.0,
            base_branch_width: 1e-7,
            shadow_volume_angle: PI / 4.0,
            shadow_adjustment_coef: 1.0,
            shadow_adjustment_base: 1.0,
            trunk_length: 0.3,

            tropism_weight: 0.1,
            current_direction_weight: 0.5,
            optimal_growth_direction_weight: 0.3,

            environment_size: 40,
            environment_points_count: 100000,
            iterations_count: 20,
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
}

impl Default for BudFate {
    fn default() -> Self {
        BudFate::Dormant
    }
}

#[derive(Default, Debug)]
pub struct Bud {
    pub direction: Vec3,
    pub id: BudId,
    pub branch_node: Option<Box<MetamerNode>>,
    pub fate: BudFate,
}

#[derive(Default, Debug)]
pub struct MetamerNode {
    pub global_position: Vec3,
    pub width: f32,
    pub main_bud: Bud,
    pub axillary_bud: Bud,
}

impl MetamerNode {
    pub fn distance_to(&self, environment_point: &Vec3) -> f32 {
        self.global_position.distance(environment_point.clone())
    }
}

impl From<MetamerNode> for TreeNode {
    fn from(node: MetamerNode) -> Self {
        TreeNode {
            global_position: node.global_position,
            width: node.width,
            main_branch: match node.main_bud.fate {
                BudFate::Shoot => Some(Box::new(TreeNode::from(
                    *node.main_bud.branch_node.unwrap(),
                ))),
                _ => None,
            },
            lateral_branch: match node.axillary_bud.fate {
                BudFate::Shoot => Some(Box::new(TreeNode::from(
                    *node.axillary_bud.branch_node.unwrap(),
                ))),
                _ => None,
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

pub fn is_in_occupancy_zone(
    point: &Vec3,
    node: &MetamerNode,
    occupancy_radius_coef: f32,
    internode_length: f32,
) -> bool {
    if node.distance_to(point) <= occupancy_radius_coef * internode_length {
        return true;
    }
    for bud in [&node.main_bud, &node.axillary_bud] {
        match bud.fate {
            BudFate::Dormant => {
                if let Some(branch) = &bud.branch_node {
                    if is_in_occupancy_zone(
                        point,
                        branch,
                        occupancy_radius_coef,
                        node.distance_to(&branch.global_position),
                    ) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
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

    pub fn clear_occupancy_zones(&mut self, root: &MetamerNode, occupancy_radius_coef: f32) {
        self.points = self
            .points
            .iter()
            .filter(|&point| !is_in_occupancy_zone(point, root, occupancy_radius_coef, 0.0))
            .cloned()
            .collect()
    }
}

pub fn is_in_perception_volume(
    point: &Vec3,
    bud_position: &Vec3,
    bud_direction: &Vec3,
    perception_angle: f32,
    perception_radius: f32,
) -> bool {
    point.distance(bud_position.clone()) <= perception_radius
        && (point.clone() - bud_position.clone()).angle_between(bud_direction.clone())
            <= perception_angle
}

pub fn generate_environment(
    seed: u64,
    environment_size: u64,
    environment_points_count: u32,
) -> Environment {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut points = vec![];

    for _ in 0..environment_points_count {
        let x = rng.gen_range(-(environment_size as f32)..=environment_size as f32);
        let y = rng.gen_range((0.0)..=environment_size as f32);
        let z = rng.gen_range(-(environment_size as f32)..=environment_size as f32);

        points.push(Vec3::new(x as f32, y as f32, z as f32));
    }

    Environment {
        points,
        bud_id_counter: BudId(0),
    }
}

#[derive(Default, Debug, Clone)]
pub struct BudLocalEnvironment {
    pub optimal_growth_direction: Vec3,
    pub resource: f32,
    pub light_exposure: f32,
    pub subtree_size: usize,
}

pub fn find_candidates_for_environment_point(
    node: &MetamerNode,
    associated_bud_candidates: &mut Vec<BudId>,
    current_minimal_distance: &mut f32,
    environment_point: &Vec3,
    perception_angle: f32,
    perception_distance_coef: f32,
    internode_length: f32,
) {
    for bud in [&node.main_bud, &node.axillary_bud] {
        match bud.fate {
            BudFate::Dormant => {
                if is_in_perception_volume(
                    environment_point,
                    &node.global_position,
                    &bud.direction,
                    perception_angle,
                    perception_distance_coef * internode_length,
                ) {
                    let distance = node.distance_to(environment_point);
                    if associated_bud_candidates.is_empty() || distance < *current_minimal_distance
                    {
                        *current_minimal_distance = distance;
                        associated_bud_candidates.clear();
                        associated_bud_candidates.push(bud.id);
                    } else if distance == *current_minimal_distance {
                        associated_bud_candidates.push(bud.id);
                    }
                }
            }
            BudFate::Shoot => find_candidates_for_environment_point(
                bud.branch_node
                    .as_ref()
                    .expect("Shoot should have branch node"),
                associated_bud_candidates,
                current_minimal_distance,
                environment_point,
                perception_angle,
                perception_distance_coef,
                node.distance_to(&bud.branch_node.as_ref().unwrap().global_position),
            ),
            _ => (),
        }
    }
}

pub fn set_default_optimal_growth_direction(
    bud_info: &mut Vec<BudLocalEnvironment>,
    root: &MetamerNode,
) {
    for bud in [&root.main_bud, &root.axillary_bud] {
        match bud.fate {
            BudFate::Dormant => {
                bud_info[bud.id.0].optimal_growth_direction = bud.direction;
            }
            BudFate::Shoot => {
                bud_info[bud.id.0].optimal_growth_direction = bud.direction;
                set_default_optimal_growth_direction(bud_info, bud.branch_node.as_ref().unwrap());
            }
            _ => (),
        }
    }
}

pub fn calculate_optimal_growth_direction(
    bud_info: &mut Vec<BudLocalEnvironment>,
    environment: &Environment,
    root: &MetamerNode,
    rng: &mut StdRng,
    perception_angle: f32,
    perception_distance_coef: f32,
    trunk_length: f32,
) {
    set_default_optimal_growth_direction(bud_info, root);
    let mut associated_sets: Vec<Vec<Vec3>> = vec![vec![]; environment.get_number_of_buds()];

    for environment_point in &environment.points {
        let mut associated_bud_candidates: Vec<BudId> = vec![];
        let mut current_minimal_distance = f32::MAX;
        find_candidates_for_environment_point(
            root,
            &mut associated_bud_candidates,
            &mut current_minimal_distance,
            environment_point,
            perception_angle,
            perception_distance_coef,
            trunk_length,
        );

        let associated_bud = associated_bud_candidates.choose(rng);

        if let Some(BudId(bud_id)) = associated_bud {
            associated_sets[*bud_id].push(*environment_point);
        }
    }

    for (i, set) in associated_sets.iter().enumerate() {
        if i == 0 {
            bud_info[i].optimal_growth_direction = Vec3::new(0.0, 1.0, 0.0);
            continue;
        }
        if !set.is_empty() {
            bud_info[i].optimal_growth_direction = set.iter().sum::<Vec3>().normalize();
        }
    }
}

pub fn calculate_shadow_exposure_for_one_node(
    bud_info: &mut Vec<BudLocalEnvironment>,
    exposure_node: &MetamerNode,
    node: &MetamerNode,
    shadow_volume_angle: f32,
    shadow_adjustment_coef: f32,
    shadow_adjustment_base: f32,
) {
    for bud in [&node.main_bud, &node.axillary_bud] {
        let angle_between = Vec3::new(0.0, -1.0, 0.0)
            .angle_between(node.global_position.clone() - exposure_node.global_position.clone());
        match bud.fate {
            BudFate::Dormant => {
                if node.global_position.clone() == exposure_node.global_position.clone()
                    || angle_between <= shadow_volume_angle
                {
                    let bud_id = bud.id.0;
                    bud_info[bud_id].light_exposure -= shadow_adjustment_coef
                        * (shadow_adjustment_base)
                            .powf(-node.distance_to(&exposure_node.global_position));
                    if bud_info[bud_id].light_exposure < 0.0 {
                        bud_info[bud_id].light_exposure = 0.0;
                    }
                }
            }
            BudFate::Shoot => {
                if node.global_position.clone() == exposure_node.global_position.clone()
                    || angle_between <= shadow_volume_angle
                {
                    let bud_id = bud.id.0;
                    bud_info[bud_id].light_exposure -= shadow_adjustment_coef
                        * (shadow_adjustment_base)
                            .powf(-node.distance_to(&exposure_node.global_position));
                    if bud_info[bud_id].light_exposure < 0.0 {
                        bud_info[bud_id].light_exposure = 0.0;
                    }
                }
                calculate_shadow_exposure_for_one_node(
                    bud_info,
                    exposure_node,
                    bud.branch_node
                        .as_ref()
                        .expect("Shoot should have branch node"),
                    shadow_volume_angle,
                    shadow_adjustment_coef,
                    shadow_adjustment_base,
                );
            }
            _ => (),
        }
    }
}

pub fn calculate_shadow_exposure(
    bud_info: &mut Vec<BudLocalEnvironment>,
    node: &MetamerNode,
    root: &MetamerNode,
    shadow_volume_angle: f32,
    shadow_adjustment_coef: f32,
    shadow_adjustment_base: f32,
) {
    calculate_shadow_exposure_for_one_node(
        bud_info,
        node,
        root,
        shadow_volume_angle,
        shadow_adjustment_coef,
        shadow_adjustment_base,
    );
    for bud in [&node.main_bud, &node.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                calculate_shadow_exposure(
                    bud_info,
                    bud.branch_node
                        .as_ref()
                        .expect("Shoot should have branch node"),
                    root,
                    shadow_volume_angle,
                    shadow_adjustment_coef,
                    shadow_adjustment_base,
                );
            }
            _ => (),
        }
    }
}

pub fn assign_default_exposure(
    bud_info: &mut Vec<BudLocalEnvironment>,
    root: &MetamerNode,
    full_light_exposure: f32,
    shadow_adjustment_coef: f32,
) {
    for bud in [&root.main_bud, &root.axillary_bud] {
        match bud.fate {
            BudFate::Dormant => {
                bud_info[bud.id.0].light_exposure = full_light_exposure + shadow_adjustment_coef;
            }
            BudFate::Shoot => {
                bud_info[bud.id.0].light_exposure = full_light_exposure + shadow_adjustment_coef;
                assign_default_exposure(
                    bud_info,
                    bud.branch_node
                        .as_ref()
                        .expect("Shoot should have branch node"),
                    full_light_exposure,
                    shadow_adjustment_coef,
                );
            }
            _ => (),
        }
    }
}

pub fn push_light_to_the_root(bud_info: &mut Vec<BudLocalEnvironment>, bud: &Bud) -> f32 {
    match bud.fate {
        BudFate::Dormant => bud_info[bud.id.0].light_exposure,
        BudFate::Shoot => {
            let light = push_light_to_the_root(
                bud_info,
                &bud.branch_node
                    .as_ref()
                    .expect("Shoot should have branch node")
                    .main_bud,
            ) + push_light_to_the_root(
                bud_info,
                &bud.branch_node
                    .as_ref()
                    .expect("Shoot should have branch node")
                    .axillary_bud,
            );
            bud_info[bud.id.0].light_exposure = light;
            light
        }
        BudFate::Dead => 0.0,
    }
}

pub fn calculate_light_exposure(
    bud_info: &mut Vec<BudLocalEnvironment>,
    root: &MetamerNode,
    full_light_exposure: f32,
    shadow_volume_angle: f32,
    shadow_adjustment_coef: f32,
    shadow_adjustment_base: f32,
) {
    assign_default_exposure(bud_info, root, full_light_exposure, shadow_adjustment_coef);
    calculate_shadow_exposure(
        bud_info,
        root,
        root,
        shadow_volume_angle,
        shadow_adjustment_coef,
        shadow_adjustment_base,
    );
    push_light_to_the_root(bud_info, &root.main_bud);
}

pub fn calculate_resource_for_each_bud(
    bud_info: &mut Vec<BudLocalEnvironment>,
    node: &MetamerNode,
    resource: f32,
    apical_dominance: f32,
) {
    let denominator = apical_dominance * bud_info[node.main_bud.id.0].light_exposure
        + (1.0 - apical_dominance) * bud_info[node.axillary_bud.id.0].light_exposure;
    match node.main_bud.fate {
        BudFate::Shoot => {
            bud_info[node.main_bud.id.0].resource =
                resource * apical_dominance * bud_info[node.main_bud.id.0].light_exposure
                    / denominator;
            calculate_resource_for_each_bud(
                bud_info,
                node.main_bud
                    .branch_node
                    .as_ref()
                    .expect("Shoot should have branch node"),
                bud_info[node.main_bud.id.0].resource,
                apical_dominance,
            );
        }
        BudFate::Dormant => {
            bud_info[node.main_bud.id.0].resource =
                resource * apical_dominance * bud_info[node.main_bud.id.0].light_exposure
                    / denominator;
        }
        _ => (),
    }

    match node.axillary_bud.fate {
        BudFate::Shoot => {
            bud_info[node.axillary_bud.id.0].resource = resource
                * (1.0 - apical_dominance)
                * bud_info[node.axillary_bud.id.0].light_exposure
                / denominator;
            calculate_resource_for_each_bud(
                bud_info,
                node.axillary_bud
                    .branch_node
                    .as_ref()
                    .expect("Shoot should have branch node"),
                bud_info[node.axillary_bud.id.0].resource,
                apical_dominance,
            );
        }
        BudFate::Dormant => {
            bud_info[node.axillary_bud.id.0].resource = resource
                * (1.0 - apical_dominance)
                * bud_info[node.axillary_bud.id.0].light_exposure
                / denominator;
        }
        _ => (),
    }
}

pub fn calculate_resources(
    bud_info: &mut Vec<BudLocalEnvironment>,
    root: &MetamerNode,
    resource_coef: f32,
    bud_light_sensitivity: f32,
    apical_dominance: f32,
) {
    let current_resource = resource_coef
        * bud_info[root.main_bud.id.0]
            .light_exposure
            .powf(bud_light_sensitivity);
    bud_info[root.main_bud.id.0].resource = current_resource;
    calculate_resource_for_each_bud(bud_info, root, current_resource, apical_dominance);
}

pub fn calculate_local_environment(
    bud_info: &mut Vec<BudLocalEnvironment>,
    environment: &Environment,
    root: &MetamerNode,
    rng: &mut StdRng,
    perception_angle: f32,
    perception_distance_coef: f32,
    full_light_exposure: f32,
    shadow_volume_angle: f32,
    shadow_adjustment_coef: f32,
    shadow_adjustment_base: f32,
    resource_coef: f32,
    bud_light_sensitivity: f32,
    apical_dominance: f32,
    trunk_length: f32,
) {
    calculate_optimal_growth_direction(
        bud_info,
        environment,
        root,
        rng,
        perception_angle,
        perception_distance_coef,
        trunk_length,
    );
    calculate_light_exposure(
        bud_info,
        root,
        full_light_exposure,
        shadow_volume_angle,
        shadow_adjustment_coef,
        shadow_adjustment_base,
    );
    calculate_resources(
        bud_info,
        root,
        resource_coef,
        bud_light_sensitivity,
        apical_dominance,
    );
}

pub fn get_highest_tree_vigor(bud_info: &Vec<BudLocalEnvironment>, root: &MetamerNode) -> f32 {
    let mut highest_tree_vigor = 0.0;

    for bud in [&root.main_bud, &root.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                let branch_node = bud
                    .branch_node
                    .as_ref()
                    .expect("Shoot should have branch node");
                let vigor = get_highest_tree_vigor(bud_info, branch_node);
                if vigor > highest_tree_vigor {
                    highest_tree_vigor = vigor;
                }
            }
            BudFate::Dormant => {
                if bud_info[bud.id.0].resource > highest_tree_vigor {
                    highest_tree_vigor = bud_info[bud.id.0].resource;
                }
            }
            _ => (),
        }
    }

    highest_tree_vigor
}

pub fn move_tropism_vector_to_growth_plane(tropism_angle: f32, growth_direction: Vec3) -> Vec3 {
    let mut tropism_vector_2_d = Vec2::from_angle(tropism_angle);
    let growth_direction_truncated = Vec2::new(growth_direction.x, growth_direction.z);
    if growth_direction_truncated.length() != 0.0 {
        let scale = growth_direction_truncated.length() / tropism_vector_2_d.x;
        tropism_vector_2_d = tropism_vector_2_d * scale;
    }
    Vec3::new(
        growth_direction_truncated.x,
        tropism_vector_2_d.y,
        growth_direction_truncated.y,
    )
    .normalize()
}

pub fn get_random_direction_in_cone(direction: Vec3, angle: f32, rng: &mut StdRng) -> Vec3 {
    let initial_vector = Vec2::from_angle(angle);
    let cone_angle = rng.gen_range(0.0..2.0 * PI);
    let vector_yz = Vec2::from_angle(cone_angle).normalize() * angle.sin();
    let random_vector_01 = Vec3::new(initial_vector.x, vector_yz.x, vector_yz.y);
    let rotation = Quat::from_rotation_arc(Vec3::new(1.0, 0.0, 0.0), direction);
    return rotation.mul_vec3(random_vector_01);
}

pub fn add_new_shoots(
    bud: &mut Bud,
    environment: &mut Environment,
    rng: &mut StdRng,
    optimal_growth_direction: Vec3,
    tropism_angle: f32,
    number_of_internodes: usize,
    length_of_internodes: f32,
    global_position: Vec3,
    tropism_weight: f32,
    current_direction_weight: f32,
    optimal_growth_direction_weight: f32,
    main_branching_angle: f32,
    lateral_branching_angle: f32,
) {
    if number_of_internodes == 0 {
        return;
    }

    bud.fate = BudFate::Shoot;
    let mut growth_direction = current_direction_weight * bud.direction
        + optimal_growth_direction * optimal_growth_direction_weight;
    let tropism_vector =
        move_tropism_vector_to_growth_plane(tropism_angle, growth_direction.clone().normalize());
    growth_direction = growth_direction + tropism_weight * tropism_vector;
    growth_direction = growth_direction.normalize();

    bud.direction = growth_direction;

    let new_node_position = global_position + growth_direction * length_of_internodes;
    bud.branch_node = Some(Box::new(MetamerNode {
        global_position: new_node_position,
        main_bud: Bud {
            fate: BudFate::Dormant,
            direction: growth_direction,
            branch_node: None,
            id: environment.get_next_bud_id(),
        },
        axillary_bud: Bud {
            fate: BudFate::Dormant,
            direction: get_random_direction_in_cone(growth_direction, lateral_branching_angle, rng),
            branch_node: None,
            id: environment.get_next_bud_id(),
        },
        ..default()
    }));

    add_new_shoots(
        &mut bud.branch_node.as_mut().unwrap().main_bud,
        environment,
        rng,
        optimal_growth_direction,
        tropism_angle,
        number_of_internodes - 1,
        length_of_internodes,
        new_node_position,
        tropism_weight,
        current_direction_weight,
        optimal_growth_direction_weight,
        main_branching_angle,
        lateral_branching_angle,
    )
}

pub fn determine_fate_for_each_bud(
    bud_info: &Vec<BudLocalEnvironment>,
    environment: &mut Environment,
    node: &mut MetamerNode,
    rng: &mut StdRng,
    highest_tree_vigor: f32,
    tropism_angle: f32,
    tropism_weight: f32,
    current_direction_weight: f32,
    optimal_growth_direction_weight: f32,
    main_branching_angle: f32,
    lateral_branching_angle: f32,
    prunning_threshold: f32,
    maximum_shoot_length: f32,
) {
    for bud in [&mut node.main_bud, &mut node.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                if bud_info[bud.id.0].resource / (bud_info[bud.id.0].subtree_size as f32)
                    < prunning_threshold
                {
                    bud.fate = BudFate::Dead;
                    continue;
                }
                let branch_node = bud
                    .branch_node
                    .as_mut()
                    .expect("Shoot should have branch node");
                determine_fate_for_each_bud(
                    bud_info,
                    environment,
                    branch_node,
                    rng,
                    highest_tree_vigor,
                    tropism_angle,
                    tropism_weight,
                    current_direction_weight,
                    optimal_growth_direction_weight,
                    main_branching_angle,
                    lateral_branching_angle,
                    prunning_threshold,
                    maximum_shoot_length,
                );
            }
            BudFate::Dormant => {
                if bud_info[bud.id.0].resource >= 1.0 {
                    let number_of_internodes = (bud_info[bud.id.0].resource * maximum_shoot_length
                        / highest_tree_vigor)
                        .floor() as usize;
                    let length_of_internodes =
                        bud_info[bud.id.0].resource / number_of_internodes as f32;
                    add_new_shoots(
                        bud,
                        environment,
                        rng,
                        bud_info[bud.id.0].optimal_growth_direction,
                        tropism_angle,
                        number_of_internodes,
                        length_of_internodes,
                        node.global_position,
                        tropism_weight,
                        current_direction_weight,
                        optimal_growth_direction_weight,
                        main_branching_angle,
                        lateral_branching_angle,
                    );
                }
            }
            _ => (),
        }
    }
}

pub fn determine_buds_fate(
    bud_info: &Vec<BudLocalEnvironment>,
    environment: &mut Environment,
    root: &mut MetamerNode,
    rng: &mut StdRng,
    tropism_angle: f32,
    tropism_weight: f32,
    current_direction_weight: f32,
    optimal_growth_direction_weight: f32,
    main_branching_angle: f32,
    lateral_branching_angle: f32,
    prunning_threshold: f32,
    maximum_shoot_length: f32,
) {
    let hightest_tree_vigor = get_highest_tree_vigor(bud_info, root);

    determine_fate_for_each_bud(
        bud_info,
        environment,
        root,
        rng,
        hightest_tree_vigor,
        tropism_angle,
        tropism_weight,
        current_direction_weight,
        optimal_growth_direction_weight,
        main_branching_angle,
        lateral_branching_angle,
        prunning_threshold,
        maximum_shoot_length,
    );
}

pub fn calculate_branch_sizes(
    bud_info: &mut Vec<BudLocalEnvironment>,
    node: &MetamerNode,
) -> usize {
    let mut current_subtree_size = 0;
    for bud in [&node.main_bud, &node.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                let branch_node = bud
                    .branch_node
                    .as_ref()
                    .expect("Shoot should have branch node");
                bud_info[bud.id.0].subtree_size +=
                    1 + calculate_branch_sizes(bud_info, branch_node);
                current_subtree_size += bud_info[bud.id.0].subtree_size;
            }
            _ => (),
        }
    }
    current_subtree_size
}

pub fn update_branch_width(node: &mut MetamerNode, base_branch_width: f32) {
    let mut child_width_square_sum = 0.0;
    for bud in [&mut node.main_bud, &mut node.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                let branch_node = bud
                    .branch_node
                    .as_mut()
                    .expect("Shoot should have branch node");
                update_branch_width(branch_node, base_branch_width);
                child_width_square_sum += branch_node.width.powi(2);
            }
            BudFate::Dead => {
                if let Some(branch_node) = bud.branch_node.as_mut() {
                    update_branch_width(branch_node, base_branch_width);
                    child_width_square_sum += branch_node.width.powi(2);
                }
            }
            _ => (),
        }
    }
    node.width = child_width_square_sum.sqrt();
}

pub fn debug_tree(root: &MetamerNode) {
    for bud in [&root.main_bud, &root.axillary_bud] {
        match bud.fate {
            BudFate::Shoot => {
                let branch_node = bud
                    .branch_node
                    .as_ref()
                    .expect("Shoot should have branch node");
                debug_tree(branch_node);
            }
            _ => (),
        }
    }
}

pub fn generate(tree_info: TreeInfo) -> TreeStructure {
    let args = SeedStructure::from(tree_info);

    let mut environment = generate_environment(
        args.seed,
        args.environment_size,
        args.environment_points_count,
    );
    let mut rng = StdRng::seed_from_u64(args.seed);

    let mut root = MetamerNode {
        global_position: Vec3::ZERO,
        width: args.base_branch_width,
        main_bud: Bud {
            direction: Vec3::new(0.0, 1.0, 0.0),
            id: environment.get_next_bud_id(),
            branch_node: None,
            fate: BudFate::Dormant,
        },
        axillary_bud: Bud {
            direction: Vec3::ZERO,
            id: environment.get_next_bud_id(),
            branch_node: None,
            fate: BudFate::Dead,
        },
    };

    for _ in 0..args.iterations_count {
        let mut bud_info: Vec<BudLocalEnvironment> =
            vec![BudLocalEnvironment::default(); environment.get_number_of_buds()];

        environment.clear_occupancy_zones(&root, args.occupancy_radius_coef);
        calculate_local_environment(
            &mut bud_info,
            &environment,
            &root,
            &mut rng,
            args.bud_perception_angle,
            args.bud_perception_distance_coef,
            args.full_light_exposure,
            args.shadow_volume_angle,
            args.shadow_adjustment_coef,
            args.shadow_adjustment_base,
            args.resource_coef,
            args.bud_light_sensitivity,
            args.apical_dominance,
            args.trunk_length,
        );

        calculate_branch_sizes(&mut bud_info, &root);

        determine_buds_fate(
            &bud_info,
            &mut environment,
            &mut root,
            &mut rng,
            args.tropism_angle,
            args.tropism_weight,
            args.current_direction_weight,
            args.optimal_growth_direction_weight,
            args.main_branching_angle,
            args.lateral_branching_angle,
            args.branch_self_pruning,
            args.maximum_shoot_length,
        );

        update_branch_width(&mut root, args.base_branch_width);
    }

    let tree = TreeStructure {
        root: TreeNode::from(root),
    };
    return tree;
}
