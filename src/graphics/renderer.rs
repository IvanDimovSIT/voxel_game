use std::{collections::HashMap, rc::Rc};

use macroquad::{
    camera::{Camera3D, set_camera},
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    prelude::debug,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, AreaLocation},
        location::{InternalLocation, LOCATION_OFFSET, Location},
        voxel::{MAX_VOXEL_VARIANTS, Voxel},
        world::World,
    },
    service::camera_controller::CameraController,
    utils::StackVec,
};

use super::{
    mesh_generator::{FaceDirection, MeshGenerator},
    texture_manager::TextureManager,
    voxel_shader::VoxelShader,
};

const AREA_RENDER_THRESHOLD: f32 = 0.4;
const LOOK_DOWN_RENDER_MULTIPLIER: f32 = 0.5;
const VOXEL_RENDER_THRESHOLD: f32 = 0.71;
const VOXEL_PROXIMITY_THRESHOLD: f32 = 5.5;

/// stores the face count, voxel type and mesh data
type MeshInfo = (u8, Voxel, Mesh);
type Meshes = HashMap<AreaLocation, HashMap<InternalLocation, MeshInfo>>;

struct GeneratedMeshResult {
    pub mesh: Option<Mesh>,
    pub area_location: AreaLocation,
    pub face_count: usize,
}
impl GeneratedMeshResult {
    pub fn new_empty(area_location: AreaLocation) -> Self {
        Self {
            mesh: None,
            area_location,
            face_count: 0,
        }
    }
}

pub struct Renderer {
    meshes: Meshes,
    mesh_generator: MeshGenerator,
    shader: VoxelShader,
}
impl Renderer {
    pub fn new(texture_manager: Rc<TextureManager>) -> Self {
        Self {
            meshes: Meshes::new(),
            mesh_generator: MeshGenerator::new(texture_manager),
            shader: VoxelShader::new(),
        }
    }

    pub fn unload_area(&mut self, area_location: AreaLocation) {
        let remove_result = self.meshes.remove(&area_location);
        if remove_result.is_none() {
            debug!("Area {:?} is already unloaded", area_location);
        }
    }

    fn generate_mesh_for_voxel(
        &mut self,
        world: &mut World,
        global_location: InternalLocation,
        voxel: Voxel,
    ) -> GeneratedMeshResult {
        let area_location = World::convert_global_to_area_location(global_location);

        if voxel == Voxel::None {
            return GeneratedMeshResult::new_empty(area_location);
        }

        let mut face_directions = StackVec::<FaceDirection, 6>::new();

        if Voxel::None == world.get(global_location.offset_x(1)) {
            face_directions.push(FaceDirection::Left);
        }
        if Voxel::None == world.get(global_location.offset_x(-1)) {
            face_directions.push(FaceDirection::Right);
        }
        if Voxel::None == world.get(global_location.offset_y(1)) {
            face_directions.push(FaceDirection::Front);
        }
        if Voxel::None == world.get(global_location.offset_y(-1)) {
            face_directions.push(FaceDirection::Back);
        }
        if global_location.z + 1 < AREA_HEIGHT
            && Voxel::None == world.get(global_location.offset_z(1))
        {
            face_directions.push(FaceDirection::Down);
        }
        if global_location.z > 0 && Voxel::None == world.get(global_location.offset_z(-1)) {
            face_directions.push(FaceDirection::Up);
        }

        if face_directions.is_empty() {
            return GeneratedMeshResult::new_empty(area_location);
        }

        let mesh = self
            .mesh_generator
            .generate_mesh(voxel, global_location, &face_directions);

        GeneratedMeshResult {
            mesh: Some(mesh),
            area_location,
            face_count: face_directions.len(),
        }
    }

    fn set_voxel_mesh(
        &mut self,
        area_location: AreaLocation,
        location: InternalLocation,
        mesh: Option<Mesh>,
        face_count: usize,
        voxel: Voxel,
    ) {
        debug_assert!((mesh.is_none() && face_count == 0) || (mesh.is_some() && face_count >= 1));

        let mut area = self.meshes.get_mut(&area_location);
        if area.is_none() {
            self.meshes.insert(area_location, HashMap::new());
            area = self.meshes.get_mut(&area_location);
        }
        let area = area.unwrap();

        if let Some(some_mesh) = mesh {
            area.insert(location, (face_count as u8, voxel, some_mesh));
        } else {
            area.remove(&location);
        }
    }

    fn update_meshes_for_voxel(
        &mut self,
        world: &mut World,
        global_location: InternalLocation,
        voxel: Voxel,
    ) {
        let meshing_result = self.generate_mesh_for_voxel(world, global_location, voxel);
        self.set_voxel_mesh(
            meshing_result.area_location,
            global_location,
            meshing_result.mesh,
            meshing_result.face_count,
            voxel,
        );
    }

    pub fn update_location(&mut self, world: &mut World, location: InternalLocation) {
        if let Some(voxel) = world.get_without_loading(location) {
            self.update_meshes_for_voxel(world, location, voxel);
        }

        let mut neighbors = StackVec::<InternalLocation, 6>::new();
        neighbors.push(InternalLocation::new(
            location.x + 1,
            location.y,
            location.z,
        ));
        neighbors.push(InternalLocation::new(
            location.x - 1,
            location.y,
            location.z,
        ));
        neighbors.push(InternalLocation::new(
            location.x,
            location.y + 1,
            location.z,
        ));
        neighbors.push(InternalLocation::new(
            location.x,
            location.y - 1,
            location.z,
        ));
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

    /// generates all the meshes for the area
    pub fn load_full_area(&mut self, world: &mut World, area_location: AreaLocation) {
        if self.meshes.contains_key(&area_location) {
            return;
        }

        let voxels = world.get_renderable_voxels_for_area(area_location);

        for (location, voxel) in voxels {
            self.update_meshes_for_voxel(world, location, voxel);
        }
    }

    fn is_area_visible(
        area_location: AreaLocation,
        camera: &Camera3D,
        look: Vec3,
        render_distance: f32,
    ) -> bool {
        let area_middle = [
            (area_location.x * AREA_SIZE + AREA_SIZE / 2) as i32 - LOCATION_OFFSET,
            (area_location.y * AREA_SIZE + AREA_SIZE / 2) as i32 - LOCATION_OFFSET,
        ];
        let area_vec = vec3(
            area_middle[0] as f32,
            area_middle[1] as f32,
            camera.target.z,
        );
        if camera.position.distance(area_vec) <= render_distance {
            return true;
        }

        let area_look = (area_vec - camera.position).normalize_or_zero();

        area_look.dot(look) >= AREA_RENDER_THRESHOLD
    }

    fn calculate_render_distance(look: Vec3, render_size: u32) -> f32 {
        const DOWN: Vec3 = vec3(0.0, 0.0, 1.0);
        let dot_product = look.dot(DOWN).abs();
        // 1 - (1 - dot_product)^2
        let dot_product_smooth = 1.0 - (1.0 - dot_product) * (1.0 - dot_product);
        let render_distance = dot_product_smooth * (AREA_SIZE * render_size) as f32;
        (render_distance * LOOK_DOWN_RENDER_MULTIPLIER).max(AREA_SIZE as f32)
    }

    fn is_voxel_visible(
        internal_location: &InternalLocation,
        look: Vec3,
        camera_position: Vec3,
    ) -> bool {
        let location: Location = (*internal_location).into();
        let location_vec = vec3(location.x as f32, location.y as f32, location.z as f32);
        // norm(location - camera)
        let direction_to_location = (location_vec - camera_position).normalize_or_zero();
        let dot_product = direction_to_location.dot(look);

        dot_product > VOXEL_RENDER_THRESHOLD
            || !((location_vec.x - camera_position.x).abs() > VOXEL_PROXIMITY_THRESHOLD
                || (location_vec.y - camera_position.y).abs() > VOXEL_PROXIMITY_THRESHOLD
                || (location_vec.z - camera_position.z).abs() > VOXEL_PROXIMITY_THRESHOLD)
    }

    /// returns an iterator of the voxel meshes to be rendered in an optimised order
    fn optimise_render_order<'a>(
        mesh_infos: &'a [(&'a InternalLocation, &'a MeshInfo)],
    ) -> impl Iterator<Item = (&'a InternalLocation, &'a MeshInfo)> {
        let mut groups: Vec<Vec<(&'a InternalLocation, &'a MeshInfo)>> =
            vec![vec![]; MAX_VOXEL_VARIANTS];
        for pair in mesh_infos {
            let index = pair.1.1.index();
            groups[index].push(*pair);
        }

        groups.into_iter().flatten()
    }

    /// Returns the number of rendered areas and faces
    pub fn render_voxels(&self, camera: &Camera3D, render_size: u32) -> (usize, usize) {
        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);
        self.shader.set_voxel_material(camera);
        let position: Vec3 = camera.position;
        let look = (camera.target - position).normalize_or_zero();
        let render_distance = Self::calculate_render_distance(look, render_size);

        let visible_areas: Vec<_> = self
            .meshes
            .iter()
            .filter(|(area, _meshes)| Self::is_area_visible(**area, camera, look, render_distance))
            .collect();

        let visible_voxels: Vec<_> = visible_areas
            .par_iter()
            .flat_map(|(_, y)| *y)
            .filter(|(location, _mesh_with_face_count)| {
                Self::is_voxel_visible(location, look, position)
            })
            .collect();
        let optimised_voxel_meshes = Self::optimise_render_order(&visible_voxels);

        let mut faces_visible: usize = 0;
        for (_location, (face_count, _, mesh)) in optimised_voxel_meshes {
            debug_assert!(*face_count > 0, "Meshes map is storing empty voxels");
            faces_visible += *face_count as usize;
            draw_mesh(mesh);
        }

        (visible_areas.len(), faces_visible)
    }

    pub fn get_voxel_face_count(&self) -> usize {
        self.meshes
            .values()
            .flat_map(|areas| areas.values())
            .map(|voxel_meshes| voxel_meshes.0 as usize)
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

    pub fn get_texture_manager(&self) -> &TextureManager {
        self.mesh_generator.get_texture_manager()
    }
}
