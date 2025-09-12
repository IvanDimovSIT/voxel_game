use std::{collections::HashMap};

use macroquad::{
    math::{vec2, vec3, vec4, Vec3},
    models::Mesh,
    ui::Vertex,
};
use tobj::{LoadOptions, Model, load_obj};

use crate::graphics::texture_manager::TextureManager;

const BASE_PATH: &str = "assets/models/";

const MODEL_FILES: [(MeshId, &str); 1] = [(MeshId::TestModel, "test.obj")];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshId {
    TestModel,
}

pub struct MeshManager {
    models: HashMap<MeshId, Mesh>,
}
impl MeshManager {
    pub fn new(texture_manager: &TextureManager) -> Self {
        let mut models = HashMap::new();
        for (id, path) in MODEL_FILES {
            let fullpath = format!("{BASE_PATH}{path}");
            let mesh = Self::load_mesh(&fullpath, texture_manager);
            models.insert(id, mesh);
        }

        Self { models }
    }

    fn load_mesh(filepath: &str, texture_manager: &TextureManager) -> Mesh {
        let (loaded_models, _loaded_materials) = load_obj(
            filepath,
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: false,
                ignore_lines: false,
            },
        )
        .expect(&format!("Error loading model '{filepath}'"));

        assert_eq!(
            loaded_models.len(),
            1,
            "model files must contain a single model"
        );
        let loaded_model = loaded_models.into_iter().last().unwrap();
        Self::convert_loaded_model_to_mesh(loaded_model, texture_manager)
    }

    fn convert_loaded_model_to_mesh(loaded_model: Model, texture_manager: &TextureManager) -> Mesh {
        Mesh {
            vertices: Self::construct_vertices(&loaded_model),
            indices: loaded_model
                .mesh
                .indices
                .iter()
                .map(|x| *x as u16)
                .collect(),
            texture: Some(texture_manager.get(crate::model::voxel::Voxel::Boards)),
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
                vec2(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1])
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

    fn move_mesh(mesh: &mut Mesh, by: Vec3) {
        for v in &mut mesh.vertices {
            v.position += by;
        }
    }

    pub fn get(&self, id: MeshId) -> Mesh {
        let mesh = self.models.get(&id).expect("Failed to find mesh");

        Mesh {
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            texture: mesh.texture.clone(),
        }
    }

    pub fn get_at(&self, id: MeshId, at: Vec3) -> Mesh {
        let mut mesh = self.get(id);
        Self::move_mesh(&mut mesh, at);

        mesh
    }
}
