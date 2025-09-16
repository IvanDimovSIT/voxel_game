use std::collections::HashMap;

use macroquad::{
    math::{Vec3, vec2, vec3, vec4},
    models::Mesh,
    texture::Texture2D,
    ui::Vertex,
};
use tobj::{LoadOptions, Model, load_obj};

use crate::graphics::texture_manager::TextureManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshId {
    Bunny,
    ButterflyDown,
    ButterflyUp,
}
impl MeshId {
    pub const VARIANTS: usize = 3;

    pub fn index(self) -> usize {
        let index = self as usize;
        debug_assert!(index < Self::VARIANTS);

        index
    }
}

const BASE_PATH: &str = "assets/models/";

const MODEL_FILES: [(MeshId, &str); MeshId::VARIANTS] = [
    (MeshId::Bunny, "bunny.obj"),
    (MeshId::ButterflyDown, "butterfly1.obj"),
    (MeshId::ButterflyUp, "butterfly2.obj"),
];

const MAX_COORDINATES: f32 = 4.0;

pub struct MeshManager {
    models: HashMap<MeshId, Mesh>,
}
impl MeshManager {
    pub fn new(texture_manager: &TextureManager) -> Self {
        let mut models = HashMap::new();
        for (id, path) in MODEL_FILES {
            let fullpath = format!("{BASE_PATH}{path}");
            let texture = texture_manager.get_mesh_texture(id);
            let mesh = Self::load_mesh(&fullpath, texture);
            models.insert(id, mesh);
        }

        Self { models }
    }

    fn load_mesh(filepath: &str, texture: Texture2D) -> Mesh {
        let (loaded_models, _loaded_materials) = load_obj(
            filepath,
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: false,
                ignore_lines: false,
            },
        )
        .unwrap_or_else(|_| panic!("Error loading model '{filepath}'"));

        assert_eq!(
            loaded_models.len(),
            1,
            "model files must contain a single model"
        );
        let loaded_model = loaded_models.into_iter().last().unwrap();
        Self::convert_loaded_model_to_mesh(loaded_model, texture)
    }

    fn convert_loaded_model_to_mesh(loaded_model: Model, texture: Texture2D) -> Mesh {
        Mesh {
            vertices: Self::construct_vertices(&loaded_model),
            indices: loaded_model
                .mesh
                .indices
                .iter()
                .map(|x| *x as u16)
                .collect(),
            texture: Some(texture),
        }
    }

    fn construct_vertices(loaded_model: &Model) -> Vec<Vertex> {
        let mesh = &loaded_model.mesh;
        let mut vertices = Vec::with_capacity(mesh.positions.len() / 3);
        for i in 0..mesh.positions.len() / 3 {
            let pos = vec3(
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            );
            debug_assert!(pos.x.abs() < MAX_COORDINATES);
            debug_assert!(pos.y.abs() < MAX_COORDINATES);
            debug_assert!(pos.z.abs() < MAX_COORDINATES);

            let norm = if !mesh.normals.is_empty() {
                vec4(
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                    0.0,
                )
            } else {
                vec4(0.0, 0.0, 1.0, 0.0)
            };

            let uv = if !mesh.texcoords.is_empty() {
                vec2(mesh.texcoords[i * 2], 1.0 - mesh.texcoords[i * 2 + 1])
            } else {
                vec2(0.0, 0.0)
            };

            vertices.push(Vertex {
                position: pos,
                normal: norm,
                uv,
                color: [255, 255, 255, 255],
            });
        }

        vertices
    }

    pub fn move_mesh(mesh: &mut Mesh, by: Vec3) {
        for v in &mut mesh.vertices {
            v.position += by;
        }
    }

    pub fn get_at(&self, id: MeshId, at: Vec3) -> Mesh {
        let mesh_ref = self.models.get(&id).expect("Failed to find mesh");

        let mut mesh = Mesh {
            vertices: mesh_ref.vertices.clone(),
            indices: mesh_ref.indices.clone(),
            texture: mesh_ref.texture.clone(),
        };

        Self::move_mesh(&mut mesh, at);

        mesh
    }

    /// rotates a mesh and it's direction around z
    pub fn rotate_around_z_with_direction(
        mesh: &mut Mesh,
        direction: &mut Vec3,
        origin: Vec3,
        angle: f32,
    ) {
        if angle <= f32::EPSILON {
            return;
        }

        let (sin_a, cos_a) = angle.sin_cos();
        Self::rotate_mesh(mesh, origin, sin_a, cos_a);
        Self::rotate_direction(direction, sin_a, cos_a);
    }

    /// rotates a mesh around z
    pub fn rotate_around_z(mesh: &mut Mesh, origin: Vec3, angle: f32) {
        if angle <= f32::EPSILON {
            return;
        }

        let (sin_a, cos_a) = angle.sin_cos();
        Self::rotate_mesh(mesh, origin, sin_a, cos_a);
    }

    fn rotate_mesh(mesh: &mut Mesh, origin: Vec3, sin: f32, cos: f32) {
        for v in &mut mesh.vertices {
            let p = &mut v.position;

            let dx = p.x - origin.x;
            let dy = p.y - origin.y;

            let new_x = dx * cos - dy * sin;
            let new_y = dx * sin + dy * cos;

            p.x = origin.x + new_x;
            p.y = origin.y + new_y;
        }
    }

    fn rotate_direction(direction: &mut Vec3, sin: f32, cos: f32) {
        debug_assert!(direction.is_normalized());
        let dir_x = direction.x;
        let dir_y = direction.y;

        direction.x = dir_x * cos - dir_y * sin;
        direction.y = dir_x * sin + dir_y * cos;
        *direction = direction.normalize_or_zero();
    }
}
