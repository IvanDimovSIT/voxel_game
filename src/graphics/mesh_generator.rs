use std::rc::Rc;

use macroquad::{
    math::{Vec2, Vec3, Vec4},
    models::Mesh,
    ui::Vertex,
};

use crate::{
    model::{
        location::{InternalLocation, Location},
        voxel::Voxel,
    },
    service::asset_manager::AssetManager,
};

use super::texture_manager::TextureManager;

#[derive(Debug, Clone, Copy)]
pub enum FaceDirection {
    /// z - 1
    Up,
    /// z + 1
    Down,
    /// x - 1
    Left,
    /// x + 1
    Right,
    /// y + 1
    Front,
    /// y - 1
    Back,
}

pub struct MeshGenerator {
    asset_manager: Rc<AssetManager>,
}
impl MeshGenerator {
    const COLOR: [u8; 4] = [255, 255, 255, 255];
    const INDECIES: [u16; 6] = [0, 1, 2, 0, 2, 3];
    const FRONT_NORMAL: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    const BACK_NORMAL: Vec4 = Vec4::new(0.0, -1.0, 0.0, 0.0);
    const RIGHT_NORMAL: Vec4 = Vec4::new(-1.0, 0.0, 0.0, 0.0);
    const LEFT_NORMAL: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    const DOWN_NORMAL: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    const UP_NORMAL: Vec4 = Vec4::new(0.0, 0.0, -1.0, 0.0);
    const ALL_DIRECTIONS: [FaceDirection; 6] = [
        FaceDirection::Front,
        FaceDirection::Back,
        FaceDirection::Left,
        FaceDirection::Right,
        FaceDirection::Up,
        FaceDirection::Down,
    ];
    const VERTICES_PER_FACE: usize = 4;

    // UVs:
    const UV_REPEATING: [Vec2; 4] = [
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];
    /// gap size in px between textures
    const GAP_SIZE: f32 = 32.0;
    /// single texture size
    const TEXTURE_SIZE: f32 = 64.0;
    /// whole texture map size
    const TEXTURE_HEIGHT: f32 = Self::TEXTURE_SIZE * 3.0 + Self::GAP_SIZE * 2.0;
    const PIXEL_SIZE: f32 = 1.0 / Self::TEXTURE_HEIGHT;
    const UV_OFFSET1: f32 = Self::TEXTURE_SIZE * Self::PIXEL_SIZE;
    const UV_OFFSET2: f32 =
        Self::TEXTURE_SIZE * Self::PIXEL_SIZE + Self::GAP_SIZE * Self::PIXEL_SIZE;
    const UV_OFFSET3: f32 =
        2.0 * Self::TEXTURE_SIZE * Self::PIXEL_SIZE + Self::GAP_SIZE * Self::PIXEL_SIZE;
    const UV_OFFSET4: f32 =
        2.0 * Self::TEXTURE_SIZE * Self::PIXEL_SIZE + 2.0 * Self::GAP_SIZE * Self::PIXEL_SIZE;
    const TOP_UV: [Vec2; 4] = [
        Vec2::new(0.0, Self::UV_OFFSET3),
        Vec2::new(1.0, Self::UV_OFFSET3),
        Vec2::new(1.0, Self::UV_OFFSET2),
        Vec2::new(0.0, Self::UV_OFFSET2),
    ];
    const SIDE_UV: [Vec2; 4] = [
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, Self::UV_OFFSET4),
        Vec2::new(0.0, Self::UV_OFFSET4),
    ];
    const BOTTOM_UV: [Vec2; 4] = [
        Vec2::new(0.0, Self::UV_OFFSET1),
        Vec2::new(1.0, Self::UV_OFFSET1),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];

    pub fn new(asset_manager: Rc<AssetManager>) -> Self {
        Self { asset_manager }
    }

    /// generates a mesh for the voxel only with the side faces from the diretions slice
    pub fn generate_mesh(
        &self,
        voxel: Voxel,
        location: InternalLocation,
        directions: &[FaceDirection],
    ) -> Mesh {
        debug_assert!(
            !directions.is_empty(),
            "Need at least one face direction to generate mesh"
        );

        let location: Location = location.into();
        let middle_x = location.x as f32;
        let middle_y = location.y as f32;
        let middle_z = location.z as f32;

        let mut vertices = Vec::with_capacity(Self::VERTICES_PER_FACE);
        let mut indices = Vec::with_capacity(Self::INDECIES.len());
        let mut index_offset = 0;

        for direction in directions {
            let face_verticies =
                Self::get_verticies_for_voxel(voxel, *direction, middle_x, middle_y, middle_z);
            let face_indecies: Vec<_> = Self::INDECIES
                .iter()
                .map(|ind| ind + index_offset)
                .collect();

            index_offset += face_verticies.len() as u16;
            vertices.extend(face_verticies);
            indices.extend(face_indecies);
        }

        Mesh {
            vertices,
            indices,
            texture: Some(self.asset_manager.texture_manager.get(voxel)),
        }
    }

    pub fn generate_mesh_for_falling_voxel(&self, voxel: Voxel, position: Vec3) -> Mesh {
        let vertices: Vec<_> = Self::ALL_DIRECTIONS
            .iter()
            .flat_map(|dir| {
                Self::get_verticies_for_voxel(voxel, *dir, position.x, position.y, position.z)
            })
            .collect();

        let indices: Vec<_> = (0..Self::ALL_DIRECTIONS.len() as u16)
            .flat_map(|offset| {
                Self::INDECIES
                    .into_iter()
                    .map(move |ind| ind + offset * Self::VERTICES_PER_FACE as u16)
            })
            .collect();

        Mesh {
            vertices,
            indices,
            texture: Some(self.asset_manager.texture_manager.get(voxel)),
        }
    }

    fn get_verticies_for_voxel(
        voxel: Voxel,
        direction: FaceDirection,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
    ) -> Vec<Vertex> {
        let (top_uv, sides_uv, bottom_uv) =
            if TextureManager::VOXELS_WITH_DIFFERENT_FACES.contains(&voxel) {
                (Self::TOP_UV, Self::SIDE_UV, Self::BOTTOM_UV)
            } else {
                (Self::UV_REPEATING, Self::UV_REPEATING, Self::UV_REPEATING)
            };

        match direction {
            FaceDirection::Front => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: sides_uv[0],
                        color: Self::COLOR,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: sides_uv[1],
                        color: Self::COLOR,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: sides_uv[2],
                        color: Self::COLOR,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: sides_uv[3],
                        color: Self::COLOR,
                        normal: Self::FRONT_NORMAL,
                    },
                ]
            }
            FaceDirection::Back => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: sides_uv[3],
                        color: Self::COLOR,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: sides_uv[2],
                        color: Self::COLOR,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: sides_uv[1],
                        color: Self::COLOR,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: sides_uv[0],
                        color: Self::COLOR,
                        normal: Self::BACK_NORMAL,
                    },
                ]
            }
            FaceDirection::Right => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: sides_uv[3],
                        color: Self::COLOR,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: sides_uv[0],
                        color: Self::COLOR,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: sides_uv[1],
                        color: Self::COLOR,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: sides_uv[2],
                        color: Self::COLOR,
                        normal: Self::RIGHT_NORMAL,
                    },
                ]
            }
            FaceDirection::Left => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: sides_uv[1],
                        color: Self::COLOR,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: sides_uv[2],
                        color: Self::COLOR,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: sides_uv[3],
                        color: Self::COLOR,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: sides_uv[0],
                        color: Self::COLOR,
                        normal: Self::LEFT_NORMAL,
                    },
                ]
            }
            FaceDirection::Down => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: bottom_uv[0],
                        color: Self::COLOR,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: bottom_uv[1],
                        color: Self::COLOR,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: bottom_uv[2],
                        color: Self::COLOR,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: bottom_uv[3],
                        color: Self::COLOR,
                        normal: Self::DOWN_NORMAL,
                    },
                ]
            }
            FaceDirection::Up => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: top_uv[0],
                        color: Self::COLOR,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: top_uv[1],
                        color: Self::COLOR,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: top_uv[2],
                        color: Self::COLOR,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: top_uv[3],
                        color: Self::COLOR,
                        normal: Self::UP_NORMAL,
                    },
                ]
            }
        }
    }

    /// checks if the face should be generated based on the current voxel and its neighbour
    pub fn should_generate_face(current_voxel: Voxel, neighbour_voxel: Voxel) -> bool {
        neighbour_voxel == Voxel::None
            || (current_voxel != Voxel::Glass && neighbour_voxel == Voxel::Glass)
    }

    /// generates an untextured quad mesh at the origin (0,0,0)
    pub fn generate_quad_mesh(size: f32) -> Mesh {
        let vertices = [
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                uv: Self::UV_REPEATING[0],
                color: Self::COLOR,
                normal: Self::DOWN_NORMAL,
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                uv: Self::UV_REPEATING[1],
                color: Self::COLOR,
                normal: Self::DOWN_NORMAL,
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.0),
                uv: Self::UV_REPEATING[2],
                color: Self::COLOR,
                normal: Self::DOWN_NORMAL,
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.0),
                uv: Self::UV_REPEATING[3],
                color: Self::COLOR,
                normal: Self::DOWN_NORMAL,
            },
        ]
        .into_iter()
        .map(|v| Vertex {
            position: Vec3::new(
                v.position.x * size,
                v.position.y * size,
                v.position.z * size,
            ),
            ..v
        })
        .collect();

        Mesh {
            vertices,
            indices: Self::INDECIES.as_slice().to_vec(),
            texture: None,
        }
    }
}
