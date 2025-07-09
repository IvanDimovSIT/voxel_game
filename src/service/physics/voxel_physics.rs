use bincode::{Decode, Encode};
use macroquad::{
    camera::{Camera3D, set_camera},
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
};

use crate::{
    graphics::{mesh_generator::MeshGenerator, renderer::Renderer},
    model::{area::AREA_HEIGHT, location::Location, voxel::Voxel, world::World},
    service::camera_controller::CameraController,
    utils::{StackVec, vector_to_location},
};

const FALLING_VOXELS: [Voxel; 3] = [Voxel::Sand, Voxel::Dirt, Voxel::Grass];
const MAX_FALL_SPEED: f32 = 3.0;
const GRAVITY: f32 = 0.2;
const VIEW_CULLING_COEFFICIENT: f32 = 0.7;

struct SimulatedVoxel {
    voxel_type: Voxel,
    mesh: Mesh,
    position: Vec3,
    velocity: f32,
}
impl SimulatedVoxel {
    fn from_dto(dto: SimulatedVoxelDTO, mesh_generator: &MeshGenerator) -> Self {
        let position = vec3(dto.position[0], dto.position[1], dto.position[2]);
        let mesh = mesh_generator.generate_mesh_for_falling_voxel(dto.voxel_type, position);

        Self {
            voxel_type: dto.voxel_type,
            mesh,
            position,
            velocity: dto.velocity,
        }
    }

    fn create_dto(&self) -> SimulatedVoxelDTO {
        SimulatedVoxelDTO {
            voxel_type: self.voxel_type,
            position: [self.position.x, self.position.y, self.position.z],
            velocity: self.velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct SimulatedVoxelDTO {
    voxel_type: Voxel,
    position: [f32; 3],
    velocity: f32,
}

pub struct VoxelSimulator {
    simulated_voxels: Vec<SimulatedVoxel>,
}
impl VoxelSimulator {
    pub fn new(
        simulated_voxel_dtos: Vec<SimulatedVoxelDTO>,
        mesh_generator: &MeshGenerator,
    ) -> Self {
        let simulated_voxels = simulated_voxel_dtos
            .into_iter()
            .map(|dto| SimulatedVoxel::from_dto(dto, mesh_generator))
            .collect();

        Self { simulated_voxels }
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

    /// checks if the voxels around the input one should be simulated for falling
    pub fn update_voxels(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        location_to_check: Location,
    ) {
        let mut to_check = StackVec::new();
        Self::get_voxels_to_check(&mut to_check, location_to_check);

        for location in to_check {
            let voxel = world.get(location);
            if !FALLING_VOXELS.contains(&voxel) || location.z + 1 >= AREA_HEIGHT as i32 {
                continue;
            }
            let lower = Location {
                z: location.z + 1,
                ..location
            };
            if world.get(lower) != Voxel::None {
                continue;
            }

            let position = location.into();
            world.set(location, Voxel::None);
            renderer.update_location(world, location);
            self.simulated_voxels.push(SimulatedVoxel {
                voxel_type: voxel,
                mesh: renderer
                    .get_mesh_generator()
                    .generate_mesh_for_falling_voxel(voxel, position),
                position,
                velocity: 0.0,
            });

            // recursive call
            let up_location = Location {
                z: location.z - 1,
                ..location
            };
            if up_location.z >= 0 && FALLING_VOXELS.contains(&world.get(up_location)) {
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
            voxel.mesh.vertices.iter_mut().for_each(|v| {
                v.position.z += voxel.velocity;
            });
        }

        self.simulated_voxels
            .retain(|voxel| Self::retain_or_place_voxel(voxel, world, renderer));
    }

    pub fn draw(&self, camera: &Camera3D) {
        if self.simulated_voxels.is_empty() {
            return;
        }
        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);
        let look = (camera.target - camera.position).normalize();
        let culled_voxels = Self::cull_voxels(self.simulated_voxels.iter(), look, camera.position);

        for voxel in culled_voxels {
            draw_mesh(&voxel.mesh);
        }
    }

    pub fn location_has_voxel(&self, location: Location) -> bool {
        self.simulated_voxels.iter().any(|voxel| {
            let voxel_location = vector_to_location(voxel.position);
            voxel_location == location
        })
    }

    pub fn create_simulated_voxel_dtos(&self) -> Vec<SimulatedVoxelDTO> {
        self.simulated_voxels
            .iter()
            .map(SimulatedVoxel::create_dto)
            .collect()
    }

    /// returns true if the voxel should continue falling, otherwise returns false and places it in the world
    fn retain_or_place_voxel(
        voxel: &SimulatedVoxel,
        world: &mut World,
        renderer: &mut Renderer,
    ) -> bool {
        let location = vector_to_location(voxel.position + vec3(0.0, 0.0, 0.5));
        if location.z >= AREA_HEIGHT as i32 {
            return false;
        }
        let world_voxel = world.get(location);
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
        let up_voxel = world.get(up_location);
        if up_voxel == Voxel::None {
            world.set(up_location, voxel.voxel_type);
            renderer.update_location(world, up_location);
        }

        false
    }

    fn cull_voxels<'a, T>(
        iter: T,
        look: Vec3,
        camera_position: Vec3,
    ) -> impl Iterator<Item = &'a SimulatedVoxel>
    where
        T: Iterator<Item = &'a SimulatedVoxel>,
    {
        iter.filter(move |simulated_voxel| {
            let direction_to_voxel = (simulated_voxel.position - camera_position).normalize();
            let dot_product = direction_to_voxel.dot(look);
            dot_product > VIEW_CULLING_COEFFICIENT
        })
    }
}
