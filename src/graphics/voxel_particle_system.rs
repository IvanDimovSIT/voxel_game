use macroquad::{
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    rand::{gen_range, rand},
};

use crate::{
    graphics::mesh_generator::MeshGenerator,
    model::{location::Location, player_info::PlayerInfo, voxel::Voxel},
    service::physics::player_physics::CollisionType,
};

const RANDOM_POSITION_OFFSET: f32 = 0.1;
const RANDOM_VELOCITY: f32 = 4.5;
const RANDOM_DESTROYED_COUNT: u32 = 10;
const MIN_DESTROYED_COUNT: u32 = 10;
const LANDING_COUNT: u32 = 15;
const LANDING_Z_OFFSET: f32 = 0.3;
const PARTICLE_LIFE: f32 = 0.4;
const GRAVITY: Vec3 = vec3(0.0, 0.0, 15.0);

struct VoxelParticle {
    position: Vec3,
    velocity: Vec3,
    delta: f32,
    mesh: Mesh,
}
impl VoxelParticle {
    fn create_random(position: Vec3, mesh: Mesh) -> Self {
        let x_velocity = gen_range(-RANDOM_VELOCITY, RANDOM_VELOCITY);
        let y_velocity = gen_range(-RANDOM_VELOCITY, RANDOM_VELOCITY);
        let z_velocity = gen_range(-RANDOM_VELOCITY, 0.0);

        Self {
            position,
            velocity: vec3(x_velocity, y_velocity, z_velocity),
            delta: 0.0,
            mesh,
        }
    }

    fn update(&mut self, delta: f32) {
        self.velocity += GRAVITY * delta;
        let delta_position = self.velocity * delta;
        self.position += delta_position;
        self.delta += delta;
        for vertex in &mut self.mesh.vertices {
            vertex.position += delta_position;
        }
    }
}

pub struct VoxelParticleSystem {
    particles: Vec<VoxelParticle>,
}
impl VoxelParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(32),
        }
    }

    pub fn add_particles_for_collision(
        &mut self,
        player_info: &PlayerInfo,
        collision: CollisionType,
        mesh_generator: &MeshGenerator,
    ) {
        if let CollisionType::Strong { voxel } = collision {
            let mut position = player_info.camera_controller.get_bottom_position();
            position.z += LANDING_Z_OFFSET;
            self.add_particles(voxel, position, LANDING_COUNT, mesh_generator);
        }
    }

    pub fn add_particles_for_destroyed(
        &mut self,
        voxel: Voxel,
        location: Location,
        mesh_generator: &MeshGenerator,
    ) {
        let count = rand() % RANDOM_DESTROYED_COUNT + MIN_DESTROYED_COUNT;
        self.add_particles(voxel, location.into(), count, mesh_generator);
    }

    pub fn update(&mut self, delta: f32) {
        for particle in &mut self.particles {
            particle.update(delta);
        }

        self.particles.retain(|p| p.delta <= PARTICLE_LIFE);
    }

    pub fn draw(&self) {
        for particle in &self.particles {
            draw_mesh(&particle.mesh);
        }
    }

    fn add_particles(
        &mut self,
        voxel: Voxel,
        position: Vec3,
        count: u32,
        mesh_generator: &MeshGenerator,
    ) {
        for _ in 0..count {
            let x_offset = gen_range(-RANDOM_POSITION_OFFSET, RANDOM_POSITION_OFFSET);
            let y_offset = gen_range(-RANDOM_POSITION_OFFSET, RANDOM_POSITION_OFFSET);
            let z_offset = gen_range(-RANDOM_POSITION_OFFSET, RANDOM_POSITION_OFFSET);
            let particle_position = position + vec3(x_offset, y_offset, z_offset);
            let mesh = mesh_generator.generate_mesh_for_particle(voxel, particle_position);
            let particle = VoxelParticle::create_random(particle_position, mesh);

            self.particles.push(particle);
        }
    }
}

#[cfg(test)]
mod tests {
    use macroquad::{
        color::WHITE,
        math::{Vec2, Vec4},
        ui::Vertex,
    };

    use super::*;

    #[test]
    fn test_update_paricle() {
        let point_mesh = Mesh {
            vertices: vec![Vertex {
                position: Vec3::ZERO,
                uv: Vec2::ZERO,
                color: WHITE.into(),
                normal: Vec4::ZERO,
            }],
            indices: vec![],
            texture: None,
        };
        let mut particle = VoxelParticle::create_random(Vec3::ZERO, point_mesh);
        let starting_position = particle.position;
        let starting_mesh_position = particle.mesh.vertices[0].position;

        particle.update(0.1);

        assert_ne!(particle.position, starting_position);
        assert_ne!(particle.mesh.vertices[0].position, starting_mesh_position);
        assert_eq!(particle.delta, 0.1);
    }
}
