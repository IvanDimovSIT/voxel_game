use macroquad::{
    math::{Vec2, Vec3, Vec4},
    models::Mesh,
    ui::Vertex,
};

use crate::model::{
    location::{InternalLocation, Location},
    voxel::Voxel,
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
    texture_manager: TextureManager,
}
impl MeshGenerator {
    const INDECIES: [u16; 6] = [0, 1, 2, 0, 2, 3];
    const FRONT_NORMAL: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    const BACK_NORMAL: Vec4 = Vec4::new(0.0, -1.0, 0.0, 0.0);
    const RIGHT_NORMAL: Vec4 = Vec4::new(-1.0, 0.0, 0.0, 0.0);
    const LEFT_NORMAL: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    const DOWN_NORMAL: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    const UP_NORMAL: Vec4 = Vec4::new(0.0, 0.0, -1.0, 0.0);

    pub async fn new() -> Self {
        Self {
            texture_manager: TextureManager::new().await,
        }
    }

    pub fn generate_mesh(
        &self,
        voxel: Voxel,
        location: InternalLocation,
        direction: FaceDirection,
    ) -> Mesh {
        let location: Location = location.into();
        let middle_x = location.x as f32;
        let middle_y = location.y as f32;
        let middle_z = location.z as f32;

        Mesh {
            vertices: Self::get_verticies_for_voxel(direction, middle_x, middle_y, middle_z),
            indices: Self::INDECIES.into(),
            texture: Some(self.texture_manager.get(voxel)),
        }
    }

    fn get_verticies_for_voxel(
        direction: FaceDirection,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
    ) -> Vec<Vertex> {
        // full brighbess
        let color = [255, 255, 255, 255];

        match direction {
            FaceDirection::Front => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::FRONT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::FRONT_NORMAL,
                    },
                ]
            }
            FaceDirection::Back => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::BACK_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::BACK_NORMAL,
                    },
                ]
            }
            FaceDirection::Right => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::RIGHT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::RIGHT_NORMAL,
                    },
                ]
            }
            FaceDirection::Left => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::LEFT_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::LEFT_NORMAL,
                    },
                ]
            }
            FaceDirection::Down => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::DOWN_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z + 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::DOWN_NORMAL,
                    },
                ]
            }
            FaceDirection::Up => {
                vec![
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 0.0),
                        color,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y - 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 0.0),
                        color,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x - 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(1.0, 1.0),
                        color,
                        normal: Self::UP_NORMAL,
                    },
                    Vertex {
                        position: Vec3::new(offset_x + 0.5, offset_y + 0.5, offset_z - 0.5),
                        uv: Vec2::new(0.0, 1.0),
                        color,
                        normal: Self::UP_NORMAL,
                    },
                ]
            }
        }
    }
}
