use macroquad::{
    camera::{Camera3D, set_camera},
    color::WHITE,
    math::{Vec3, vec3},
    models::{Mesh, draw_cube},
    texture::Texture2D,
};

use crate::{
    graphics::{falling_shader::FallingShader, renderer::Renderer},
    model::{area::AREA_HEIGHT, location::Location, voxel::Voxel, world::World},
    utils::{StackVec, vector_to_location},
};

use super::camera_controller::CameraController;

const FALLING_VOXELS: [Voxel; 3] = [
    Voxel::Sand,
    Voxel::Dirt,
    Voxel::Grass
];

#[derive(Debug)]
struct SimulatedVoxel {
    voxel_type: Voxel,
    texture: Texture2D,
    position: Vec3,
    velocity: f32,
}

const MAX_FALL_SPEED: f32 = 3.0;
const GRAVITY: f32 = 0.2;

pub struct VoxelSimulator {
    shader: FallingShader,
    simulated_voxels: Vec<SimulatedVoxel>,
}
impl VoxelSimulator {
    pub fn new() -> Self {
        Self {
            simulated_voxels: vec![],
            shader: FallingShader::new(),
        }
    }

    fn get_voxels_to_check(surrounding_locations: &mut StackVec<Location, 7>, location: Location) {
        debug_assert!(surrounding_locations.is_empty());

        surrounding_locations.push(location);
        surrounding_locations.push(Location {
            x: location.x + 1,
            ..location
        });
        surrounding_locations.push(Location {
            x: location.x - 1,
            ..location
        });
        surrounding_locations.push(Location {
            y: location.y + 1,
            ..location
        });
        surrounding_locations.push(Location {
            y: location.y - 1,
            ..location
        });
        if location.z > 0 {
            surrounding_locations.push(Location {
                z: location.z - 1,
                ..location
            });
        }
        if location.z + 1 < AREA_HEIGHT as i32 {
            surrounding_locations.push(Location {
                z: location.z + 1,
                ..location
            });
        }
    }

    /// checks if the voxels aroung the input one should be simulated for falling
    pub fn update_voxels(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        location_to_check: Location,
    ) {
        let mut to_check = StackVec::new();
        Self::get_voxels_to_check(&mut to_check, location_to_check);

        for location in to_check {
            let voxel = world.get(location.into());
            if !FALLING_VOXELS.contains(&voxel) || location.z + 1 >= AREA_HEIGHT as i32 {
                continue;
            }
            let lower = Location {
                z: location.z + 1,
                ..location
            };
            if world.get(lower.into()) != Voxel::None {
                continue;
            }

            world.set(location.into(), Voxel::None);
            renderer.update_location(world, location.into());
            self.simulated_voxels.push(SimulatedVoxel {
                voxel_type: voxel,
                texture: renderer.get_texture_manager().get(voxel),
                position: location.into(),
                velocity: 0.0,
            });

            // recursive call
            let up_location = Location {
                z: location.z - 1,
                ..location
            };
            if up_location.z >= 0 && FALLING_VOXELS.contains(&world.get(up_location.into())) {
                self.update_voxels(world, renderer, up_location);
            }
        }
    }

    /// simulates gravity for falling voxels and places them on the ground
    pub fn simulate_falling(&mut self, world: &mut World, renderer: &mut Renderer, delta: f32) {
        for voxel in &mut self.simulated_voxels {
            voxel.velocity += delta * GRAVITY;
            voxel.velocity = voxel.velocity.min(MAX_FALL_SPEED);
            voxel.position.z += voxel.velocity;
        }

        self.simulated_voxels = std::mem::take(&mut self.simulated_voxels)
            .into_iter()
            .filter(|voxel| {
                let location = vector_to_location(voxel.position + vec3(0.0, 0.0, 0.5));
                if location.z >= AREA_HEIGHT as i32 {
                    return false;
                }
                let world_voxel = world.get(location.into());
                if world_voxel == Voxel::None {
                    return true;
                }
                if location.z <= 0 {
                    return false;
                }
                let up_location = Location {
                    z: location.z - 1,
                    ..location
                };
                let up_voxel = world.get(up_location.into());
                if up_voxel == Voxel::None {
                    world.set(up_location.into(), voxel.voxel_type);
                    renderer.update_location(world, up_location.into());
                }

                false
            })
            .collect();
    }

    pub fn draw(&self, camera: &Camera3D) {
        if self.simulated_voxels.is_empty() {
            return;
        }
        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);

        self.shader.set_falling_material(camera);
        for voxel in &self.simulated_voxels {
            Self::draw_voxel(voxel);
        }
    }

    pub fn location_has_voxel(&self, location: Location) -> bool {
        self.simulated_voxels.iter().any(|voxel| {
            let voxel_location = vector_to_location(voxel.position);
            voxel_location == location
        })
    }

    fn draw_voxel(simulated_voxel: &SimulatedVoxel) {
        draw_cube(
            simulated_voxel.position,
            Vec3::ONE,
            Some(&simulated_voxel.texture),
            WHITE,
        );
    }
}
