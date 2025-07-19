use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

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
        user_settings::UserSettings,
        voxel::{MAX_VOXEL_VARIANTS, Voxel},
        world::World,
    },
    service::{camera_controller::CameraController, world_time::WorldTime},
    utils::StackVec,
};

use super::{
    mesh_generator::{FaceDirection, MeshGenerator},
    texture_manager::TextureManager,
    voxel_shader::VoxelShader,
};

const AREA_RENDER_THRESHOLD: f32 = 0.35;
const LOOK_DOWN_RENDER_MULTIPLIER: f32 = 0.5;
const VOXEL_RENDER_THRESHOLD: f32 = 0.71;
const VOXEL_PROXIMITY_THRESHOLD: f32 = 5.5;
const AREAS_TO_LOAD_PER_FRAME: usize = 2;

/// stores the face count, voxel type and mesh data
type MeshInfo = (u8, Voxel, Mesh);
type Meshes = HashMap<AreaLocation, RenderArea>;

struct RenderArea {
    mesh_map: HashMap<InternalLocation, MeshInfo>,
    lights: HashSet<InternalLocation>,
}
impl RenderArea {
    pub fn new_empty() -> Self {
        Self {
            mesh_map: HashMap::new(),
            lights: HashSet::new(),
        }
    }

    pub fn insert(&mut self, location: InternalLocation, mesh_info: MeshInfo) {
        if mesh_info.1 == Voxel::Lamp {
            self.lights.insert(location);
        }
        self.mesh_map.insert(location, mesh_info);
    }

    pub fn remove(&mut self, location: &InternalLocation) {
        self.lights.remove(location);
        self.mesh_map.remove(location);
    }
}

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
    render_set: HashSet<AreaLocation>,
}
impl Renderer {
    pub fn new(texture_manager: Rc<TextureManager>) -> Self {
        Self {
            meshes: Meshes::new(),
            mesh_generator: MeshGenerator::new(texture_manager),
            shader: VoxelShader::new(),
            render_set: HashSet::new(),
        }
    }

    pub fn unload_area(&mut self, area_location: AreaLocation) {
        let remove_result = self.meshes.remove(&area_location);
        if remove_result.is_none() {
            debug!("Area {:?} is already unloaded", area_location);
        }
        self.render_set.remove(&area_location);
    }

    fn add_area_to_load_queue(&mut self, area_location: AreaLocation) {
        if self.meshes.contains_key(&area_location) {
            return;
        }
        self.render_set.insert(area_location);
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
            self.meshes.insert(area_location, RenderArea::new_empty());
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

    pub fn update_location(&mut self, world: &mut World, location: impl Into<InternalLocation>) {
        let internal_location = location.into();
        if let Some(voxel) = world.get_without_loading(internal_location) {
            self.update_meshes_for_voxel(world, internal_location, voxel);
        }

        let mut neighbors = StackVec::<InternalLocation, 6>::new();
        neighbors.push(InternalLocation::new(
            internal_location.x + 1,
            internal_location.y,
            internal_location.z,
        ));
        neighbors.push(InternalLocation::new(
            internal_location.x - 1,
            internal_location.y,
            internal_location.z,
        ));
        neighbors.push(InternalLocation::new(
            internal_location.x,
            internal_location.y + 1,
            internal_location.z,
        ));
        neighbors.push(InternalLocation::new(
            internal_location.x,
            internal_location.y - 1,
            internal_location.z,
        ));
        if internal_location.z > 0 {
            neighbors.push(InternalLocation::new(
                internal_location.x,
                internal_location.y,
                internal_location.z - 1,
            ));
        }
        if internal_location.z < (AREA_HEIGHT - 1) {
            neighbors.push(InternalLocation::new(
                internal_location.x,
                internal_location.y,
                internal_location.z + 1,
            ));
        }

        for neighbor in neighbors {
            if let Some(neighbour_voxel) = world.get_without_loading(neighbor) {
                self.update_meshes_for_voxel(world, neighbor, neighbour_voxel);
            }
        }
    }

    /// loads the next areas in the load queue
    pub fn load_areas_in_queue(&mut self, world: &mut World) {
        let to_load: Vec<_> = self
            .render_set
            .iter()
            .copied()
            .take(AREAS_TO_LOAD_PER_FRAME)
            .collect();

        for area_location in to_load {
            self.render_set.remove(&area_location);
            self.load_full_area(world, area_location);
        }
    }

    /// generates all the meshes for the area
    fn load_full_area(&mut self, world: &mut World, area_location: AreaLocation) {
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
    pub fn render_voxels(
        &self,
        camera: &Camera3D,
        render_size: u32,
        world_time: &WorldTime,
        average_max_height: Option<f32>,
        user_settings: &UserSettings,
    ) -> (usize, usize) {
        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);
        let look = (camera.target - camera.position).normalize_or_zero();

        let visible_areas = self.prepare_visible_areas(camera, look, render_size);
        let lights = Self::prepare_lights(&visible_areas, user_settings);
        self.shader.set_voxel_material(
            camera,
            render_size,
            world_time,
            average_max_height,
            &lights,
        );

        let visible_voxels =
            Self::filter_visible_voxels(camera.position, look, &visible_areas, render_size);
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
            .flat_map(|areas| areas.mesh_map.values())
            .map(|voxel_meshes| voxel_meshes.0 as usize)
            .sum()
    }

    pub fn get_areas_waiting_to_be_rendered(&self) -> usize {
        self.render_set.len()
    }

    /// performs a blocking area loading operation
    pub fn load_all_blocking(&mut self, world: &mut World, areas: &[AreaLocation]) {
        for area_location in areas {
            self.load_full_area(world, *area_location);
        }
    }

    pub fn update_loaded_areas(&mut self, areas: &[AreaLocation]) {
        for area_location in areas {
            self.add_area_to_load_queue(*area_location);
        }

        let areas_to_unload: Vec<_> = self
            .meshes
            .keys()
            .filter(|loaded| !areas.contains(loaded))
            .copied()
            .collect();

        for area_location in areas_to_unload {
            self.render_set.remove(&area_location);
            self.unload_area(area_location);
        }
    }

    pub fn get_mesh_generator(&self) -> &MeshGenerator {
        &self.mesh_generator
    }

    pub fn get_texture_manager(&self) -> &TextureManager {
        self.mesh_generator.get_texture_manager()
    }

    pub fn get_texture_manager_copy(&self) -> Rc<TextureManager> {
        self.mesh_generator.get_texture_manager_copy()
    }

    fn prepare_lights(
        render_areas: &[(&AreaLocation, &RenderArea)],
        user_settings: &UserSettings,
    ) -> Vec<InternalLocation> {
        if !user_settings.dynamic_lighting {
            return vec![];
        }

        render_areas
            .iter()
            .flat_map(|(_, area)| area.lights.iter().copied())
            .collect()
    }

    /// prepares the areas that are visible to the camera
    fn prepare_visible_areas(
        &self,
        camera: &Camera3D,
        look: Vec3,
        render_size: u32,
    ) -> Vec<(&AreaLocation, &RenderArea)> {
        let area_render_distance = Self::calculate_area_render_distance(look, render_size);

        self.meshes
            .iter()
            .filter(|(area, _meshes)| {
                Self::is_area_visible(**area, camera, look, area_render_distance)
            })
            .collect()
    }

    /// filters the visible voxels based on the camera position and look direction
    fn filter_visible_voxels<'a>(
        camera_position: Vec3,
        look: Vec3,
        visible_areas: &'a Vec<(&'a AreaLocation, &'a RenderArea)>,
        render_size: u32,
    ) -> Vec<(&'a InternalLocation, &'a MeshInfo)> {
        let render_distance = (render_size * AREA_SIZE) as f32;

        visible_areas
            .par_iter()
            .flat_map(|(_, y)| &y.mesh_map)
            .filter(|(location, _mesh_with_face_count)| {
                Self::is_voxel_visible(location, look, camera_position, render_distance)
            })
            .collect()
    }

    /// calculates the area render distance based on the camera look direction
    fn calculate_area_render_distance(look: Vec3, render_size: u32) -> f32 {
        const DOWN: Vec3 = vec3(0.0, 0.0, 1.0);
        let dot_product = look.dot(DOWN).abs();
        // 1 - (1 - dot_product)^2
        let dot_product_smooth = 1.0 - (1.0 - dot_product) * (1.0 - dot_product);
        let render_distance = dot_product_smooth * (AREA_SIZE * render_size) as f32;

        (render_distance * LOOK_DOWN_RENDER_MULTIPLIER).max(AREA_SIZE as f32)
    }

    /// checks if the voxel is visible from the camera position
    fn is_voxel_visible(
        internal_location: &InternalLocation,
        look: Vec3,
        camera_position: Vec3,
        max_distance: f32,
    ) -> bool {
        let voxel_location: Vec3 = Location::from(*internal_location).into();
        let is_in_proximiity = (voxel_location.x - camera_position.x).abs()
            <= VOXEL_PROXIMITY_THRESHOLD
            && (voxel_location.y - camera_position.y).abs() <= VOXEL_PROXIMITY_THRESHOLD
            && (voxel_location.z - camera_position.z).abs() <= VOXEL_PROXIMITY_THRESHOLD;
        if is_in_proximiity {
            return true;
        }

        let direction_to_voxel = voxel_location - camera_position;
        let distance_to_voxel = direction_to_voxel.length();
        if distance_to_voxel > max_distance {
            return false;
        }

        let dot_product = direction_to_voxel.dot(look);

        // look . direction_to_voxel > VT * |direction_to_voxel|
        dot_product > VOXEL_RENDER_THRESHOLD * distance_to_voxel
    }
}
