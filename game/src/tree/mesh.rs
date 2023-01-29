use bevy::{
    math::vec3,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use super::gen::{TreeNode, TreeStructure};

const NODE_CIRCLE_SEGMENTS: usize = 16;

#[derive(Copy, Clone, PartialEq, Debug)]
struct OrientedVertex {
    position: Vec3,
    normal: Vec3,
}

fn gen_circle(radius: f32, segments: usize) -> Vec<OrientedVertex> {
    let mut vertices = vec![];
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
        let vec = vec3(angle.cos() * radius, 0.0, angle.sin() * radius);
        vertices.push(OrientedVertex {
            position: vec,
            normal: vec.normalize(),
        });
    }
    vertices
}

fn gen_oriented_circle(
    radius: f32,
    segments: usize,
    position: Vec3,
    direction: Vec3,
) -> impl Iterator<Item = OrientedVertex> {
    gen_circle(radius, segments).into_iter().map(move |vertex| {
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);
        OrientedVertex {
            position: rotation.mul_vec3(vertex.position) + position,
            normal: rotation.mul_vec3(vertex.normal),
        }
    })
}

struct TreeMeshBuilder<'a> {
    vertices: &'a mut Vec<[f32; 3]>,
    normals: &'a mut Vec<[f32; 3]>,
    indices: &'a mut Vec<u32>,
}

impl TreeMeshBuilder<'_> {
    fn build_child(
        &mut self,
        current: &TreeNode,
        child: &TreeNode,
        current_circle_indices: &[u32],
    ) {
        let child_circle_indices = self.build_recursive(Some(current), child);
        self.build_branch(current_circle_indices, &child_circle_indices);
    }

    fn build_branch(&mut self, current_circle_indices: &[u32], child_circle_indices: &[u32]) {
        let current_circle_windows = current_circle_indices.array_windows::<2>();
        let child_circle_windows = child_circle_indices.array_windows::<2>();

        current_circle_windows.zip(child_circle_windows).for_each(
            |(&[cur_fst, cur_snd], &[child_fst, child_snd])| {
                self.indices
                    .extend_from_slice(&[cur_snd, cur_fst, child_fst]);
                self.indices
                    .extend_from_slice(&[child_fst, child_snd, cur_snd]);
            },
        );
    }

    pub fn build_recursive(&mut self, parent: Option<&TreeNode>, current: &TreeNode) -> Vec<u32> {
        let mut direction = Vec3::ZERO;
        if let Some(parent) = parent {
            direction += current.global_position - parent.global_position;
        }
        if let Some(main_branch) = &current.main_branch {
            direction += main_branch.global_position - current.global_position;
        }
        if let Some(lateral_branch) = &current.lateral_branch {
            direction += lateral_branch.global_position - current.global_position;
        }
        if direction == Vec3::ZERO {
            direction = Vec3::Y;
        }
        let direction = direction.normalize();

        let position = current.global_position;

        let current_circle_indices =
            gen_oriented_circle(current.width, NODE_CIRCLE_SEGMENTS, position, direction)
                .map(|vertex| {
                    let idx = self.vertices.len();
                    self.vertices.push(vertex.position.into());
                    self.normals.push(vertex.normal.into());
                    idx as u32
                })
                .collect::<Vec<_>>();

        if let Some(main_branch) = &current.main_branch {
            self.build_child(current, main_branch, &current_circle_indices);
        }
        if let Some(lateral_branch) = &current.lateral_branch {
            self.build_child(current, lateral_branch, &current_circle_indices);
        }

        current_circle_indices
    }
}

pub fn build(tree: &TreeStructure) -> Mesh {
    let mut vertices = vec![];
    let mut normals = vec![];
    let mut indices = vec![];

    TreeMeshBuilder {
        vertices: &mut vertices,
        normals: &mut normals,
        indices: &mut indices,
    }
    .build_recursive(None, &tree.root);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh
}
