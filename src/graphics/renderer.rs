use std::collections::HashMap;

use macroquad::{
    camera::Camera3D, math::{vec2, vec3, Vec2, Vec3}, models::{draw_mesh, Mesh}, prelude::debug
};
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{model::{
    area::{AreaLocation, AREA_HEIGHT, AREA_SIZE},
    location::{InternalLocation, Location, LOCATION_OFFSET},
    voxel::Voxel,
    world::World,
}, service::camera_controller::{self, CameraController}};

use super::mesh_generator::{self, MeshGenerator};

type Meshes = HashMap<AreaLocation, HashMap<InternalLocation, Vec<Mesh>>>;

pub struct Renderer {
    meshes: Meshes,
    mesh_generator: MeshGenerator,
}
impl Renderer {
    pub async fn new() -> Self {
        Self {
            meshes: Meshes::new(),
            mesh_generator: MeshGenerator::new().await,
        }
    }

    pub fn unload_area(&mut self, area_location: AreaLocation) {
        let remove_result = self.meshes.remove(&area_location);
        if remove_result.is_none() {
            debug!("Area {:?} is already unloaded", area_location);
        }
    }

    fn generate_meshes_for_voxel(
        &mut self,
        world: &mut World,
        global_location: InternalLocation,
        voxel: Voxel,
    ) -> (AreaLocation, Vec<Mesh>) {
        let area_location = World::convert_global_to_area_location(global_location);

        if voxel == Voxel::None {
            return (area_location, vec![]);
        }

        let mut meshes = vec![];

        //TODO: Extract into seperate function
        if Voxel::None == world.get(global_location.offset_x(1)) {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Left,
            ));
        }
        if Voxel::None == world.get(global_location.offset_x(-1)) {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Right,
            ));
        }
        if Voxel::None == world.get(global_location.offset_y(1)) {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Front,
            ));
        }
        if Voxel::None == world.get(global_location.offset_y(-1)) {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Back,
            ));
        }
        if global_location.z + 1 < AREA_HEIGHT
            && Voxel::None == world.get(global_location.offset_z(1))
        {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Down,
            ));
        }
        if global_location.z > 0 && Voxel::None == world.get(global_location.offset_z(-1)) {
            meshes.push(self.mesh_generator.generate_mesh(
                voxel,
                global_location,
                mesh_generator::FaceDirection::Up,
            ));
        }

        (area_location, meshes)
    }

    fn set_meshes(
        &mut self,
        area_location: AreaLocation,
        location: InternalLocation,
        meshes: Vec<Mesh>,
    ) {
        let mut area = self.meshes.get_mut(&area_location);
        if area.is_none() {
            self.meshes.insert(area_location, HashMap::new());
            area = self.meshes.get_mut(&area_location);
        }
        let area = area.unwrap();
        if meshes.is_empty() {
            area.remove(&location);
            return;
        }

        area.insert(location, meshes);
    }

    fn update_meshes_for_voxel(
        &mut self,
        world: &mut World,
        global_location: InternalLocation,
        voxel: Voxel,
    ) {
        let (area_location, meshes) = self.generate_meshes_for_voxel(world, global_location, voxel);
        self.set_meshes(area_location, global_location, meshes);
    }

    pub fn update_location(&mut self, world: &mut World, location: InternalLocation) {
        if let Some(voxel) = world.get_without_loading(location) {
            self.update_meshes_for_voxel(world, location, voxel);
        }

        let mut neighbors = vec![
            InternalLocation::new(location.x + 1, location.y, location.z),
            InternalLocation::new(location.x - 1, location.y, location.z),
            InternalLocation::new(location.x, location.y + 1, location.z),
            InternalLocation::new(location.x, location.y - 1, location.z),
        ];
        if location.z > 0 {
            neighbors.push(InternalLocation::new(
                location.x,
                location.y,
                location.z - 1,
            ));
        }
        if location.z < (AREA_HEIGHT - 1) {
            neighbors.push(InternalLocation::new(
                location.x,
                location.y,
                location.z + 1,
            ));
        }

        for neighbor in neighbors {
            if let Some(neighbour_voxel) = world.get_without_loading(neighbor) {
                self.update_meshes_for_voxel(world, neighbor, neighbour_voxel);
            }
        }
    }

    pub fn load_full_area(&mut self, world: &mut World, area_location: AreaLocation) {
        if self.meshes.contains_key(&area_location) {
            return;
        }

        let voxels = world.get_renderable_voxels_for_area(area_location);

        for (location, voxel) in voxels {
            self.update_meshes_for_voxel(world, location, voxel);
        }
    }

    fn is_area_visible(area_location: AreaLocation, camera: &Camera3D, look: Vec3) -> bool {
        let area_middle = [
            (area_location.x * AREA_SIZE + AREA_SIZE/2) as i32 - LOCATION_OFFSET,
            (area_location.y * AREA_SIZE + AREA_SIZE/2) as i32 - LOCATION_OFFSET,
        ]; 
        let area_vec = vec3(area_middle[0] as f32, area_middle[1] as f32, camera.target.z);
        if camera.position.distance(area_vec) <= AREA_SIZE as f32 {
            return true;
        }

        let area_look = (area_vec - camera.position)
            .normalize_or_zero();

        area_look.dot(look) >= -0.1
    }

    /// Returns the number of rendered areas and faces
    pub fn render_voxels(&self, camera: &Camera3D) -> (usize, usize) {
        let position = camera.position;
        let look = (camera.target - position).normalize_or_zero();

        let visible_areas: Vec<_> = self.meshes.iter()
            .filter(|(area, _meshes)| 
                Self::is_area_visible(**area, camera, look))
            .collect();

        let mut faces_visible = 0;
        
        for (_, areas) in &visible_areas {
            for meshes in areas.values() {
                debug_assert!(meshes.len() > 0, "Meshes map is storing empty voxels");
                faces_visible += meshes.len();
                for mesh in meshes {
                    draw_mesh(mesh);
                }
            }
        }

        (visible_areas.len(), faces_visible)
    }

    pub fn get_voxel_face_count(&self) -> usize {
        self.meshes
            .values()
            .flat_map(|areas| areas.values())
            .map(|voxel_meshes| voxel_meshes.len())
            .sum()
    }

    pub fn update_loaded_areas(&mut self, world: &mut World, areas: &[AreaLocation]) {
        for area_location in areas {
            self.load_full_area(world, *area_location);
        }

        let areas_to_unload: Vec<_> = self
            .meshes
            .keys()
            .filter(|loaded| !areas.contains(loaded))
            .copied()
            .collect();

        for area_location in areas_to_unload {
            self.unload_area(area_location);
        }
    }
}
