use std::collections::HashMap;

use macroquad::models::{draw_mesh, Mesh};

use crate::model::{area::{AreaLocation, AREA_HEIGHT}, location::{InternalLocation, Location}, voxel::Voxel, world::World};

use super::mesh_generator::{self, MeshGenerator};


type Meshes = HashMap<AreaLocation, HashMap<InternalLocation, Vec<Mesh>>>; 

pub struct Renderer {
    meshes: Meshes,
    mesh_generator: MeshGenerator
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
            println!("Area {area_location:?} is already unloaded");
            return;
        }
    }

    fn generate_meshes_for_voxel(&mut self, world: &World, global_location: InternalLocation) -> (AreaLocation, Vec<Mesh>) {
        let (area_location, _local_location) = World::convert_global_to_local_location(global_location);
        let voxel = {
            let v = world.get_without_loading(global_location);
            if Voxel::is_empty(v) {
                return (area_location, vec![]);
            }
            v.unwrap()
        };

        let mut meshes = vec![];
        
        //TODO: Extract into seperate function
        if Voxel::is_empty(world.get_without_loading(global_location.offset_x(1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Left));
        }
        if Voxel::is_empty(world.get_without_loading(global_location.offset_x(-1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Right));
        }
        if Voxel::is_empty(world.get_without_loading(global_location.offset_y(1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Front));
        }
        if Voxel::is_empty(world.get_without_loading(global_location.offset_y(-1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Back));
        }
        if global_location.z+1 >= AREA_HEIGHT || Voxel::is_empty(world.get_without_loading(global_location.offset_z(1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Down));
        }
        if global_location.z <= 0 || Voxel::is_empty(world.get_without_loading(global_location.offset_z(-1))) {
            meshes.push(self.mesh_generator.generate_mesh(voxel, global_location, mesh_generator::FaceDirection::Up));
        }
        
        
        (area_location, meshes)
    }

    fn set_meshes(&mut self, area_location: AreaLocation, location: InternalLocation, meshes: Vec<Mesh>) {
        let mut area = self.meshes.get_mut(&area_location);
        if area.is_none() {
            self.meshes.insert(area_location, HashMap::new());
            area = self.meshes.get_mut(&area_location);
        }
        let area = area.unwrap();
        if let Some(some) = area.get_mut(&location) {
            *some = meshes;
        } else {
            area.insert(location, meshes);
        }
    }

    fn update_meshes_for_voxel(&mut self, world: &World, global_location: InternalLocation, voxel: Voxel) {
        let (area_location, meshes) = self.generate_meshes_for_voxel(world, global_location);
        self.set_meshes(area_location, global_location, meshes);
    }

    pub fn update_location(&mut self, world: &World, location: InternalLocation) {
        if let Some(voxel) = world.get_without_loading(location) {
            self.update_meshes_for_voxel(world, location, voxel);
        }
    
        let mut neighbors = vec![
            InternalLocation::new(location.x + 1, location.y, location.z),
            InternalLocation::new(location.x - 1, location.y, location.z),
            InternalLocation::new(location.x, location.y, location.z + 1),
            InternalLocation::new(location.x, location.y, location.z - 1),
        ];
        if location.y > 0 {
            neighbors.push(InternalLocation::new(location.x, location.y - 1, location.z));
        }
        if location.y < (AREA_HEIGHT+1) {
            neighbors.push(InternalLocation::new(location.x, location.y + 1, location.z));
        }
    
        for neighbor in neighbors {
            if let Some(voxel) = world.get_without_loading(neighbor) {
                self.update_meshes_for_voxel(world, neighbor, voxel);
            }
        }
    }

    pub fn load_full_area(&mut self, world: &World, area_location: AreaLocation) {
        let voxels = world.get_renderable_voxels_for_area(area_location);

        for (location, voxel) in voxels {
            self.update_meshes_for_voxel(world, location, voxel);
        }
    }

    pub fn render_voxels(&self) {
        for area in self.meshes.values() {
            for meshes in area.values() {
                for mesh in meshes {
                    draw_mesh(mesh);
                }
            }
        }
    }
}