use std::collections::HashMap;

use macroquad::{
    math::{Vec3, vec2, vec3, vec4},
    models::Mesh,
    texture::Texture2D,
    ui::Vertex,
};
use tobj::{LoadOptions, Model, load_obj};

use crate::{
    graphics::texture_manager::TextureManager, service::creatures::creature_manager::CreatureId,
};

const BASE_PATH: &str = "assets/models/";

const MODEL_FILES: [(CreatureId, &str); 1] = [(CreatureId::Bunny, "bunny.obj")];

const MAX_COORDINATES: f32 = 4.0;

pub struct MeshManager {
    models: HashMap<CreatureId, Mesh>,
}
impl MeshManager {
    pub fn new(texture_manager: &TextureManager) -> Self {
        let mut models = HashMap::new();
        for (id, path) in MODEL_FILES {
            let fullpath = format!("{BASE_PATH}{path}");
            let texture = texture_manager.get_creature_texture(id);
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

    pub fn get(&self, id: CreatureId) -> Mesh {
        let mesh = self.models.get(&id).expect("Failed to find mesh");

        Mesh {
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            texture: mesh.texture.clone(),
        }
    }

    pub fn get_at(&self, id: CreatureId, at: Vec3) -> Mesh {
        let mut mesh = self.get(id);
        Self::move_mesh(&mut mesh, at);

        mesh
    }
}
