use std::collections::HashSet;

use macroquad::{
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
};

use crate::{
    graphics::{
        mesh_manager::{MeshId, MeshManager},
        renderer::Renderer,
    },
    model::{area::AREA_HEIGHT, location::Location, voxel::Voxel, world::World},
    service::{
        asset_manager::AssetManager,
        physics::player_physics::{GRAVITY, MAX_FALL_SPEED},
    },
    utils::vector_to_location,
};

const MAX_ACTIVE_BOMBS: usize = 10;
const BOMB_DELAY_S: f32 = 3.0;
const SHORT_BOMB_DELAY_S: f32 = 0.5;
const INITIAL_BOMB_VELOCITY: f32 = -4.0;
const EXPLOSION_RADIUS: f32 = 4.0;
const EXPLOSION_RADIUS_SQ: f32 = EXPLOSION_RADIUS * EXPLOSION_RADIUS;
const EXPLOSION_DURATION_S: f32 = 0.8;

struct ActiveBomb {
    position: Vec3,
    velocity: f32,
    life_s: f32,
}

struct Explosion {
    life_s: f32,
    mesh: Mesh,
}
impl Explosion {
    fn new(position: Vec3, mesh_manager: &MeshManager) -> Self {
        Self {
            life_s: EXPLOSION_DURATION_S,
            mesh: mesh_manager.create_at(MeshId::Explosion, position),
        }
    }
}

pub struct BombSimulator {
    active_bombs: Vec<ActiveBomb>,
    explosions: Vec<Explosion>,
}
impl BombSimulator {
    pub fn new() -> Self {
        Self {
            active_bombs: Vec::with_capacity(MAX_ACTIVE_BOMBS),
            explosions: vec![],
        }
    }

    pub fn add_active_bomb(&mut self, location: Location) {
        if self.active_bombs.len() >= MAX_ACTIVE_BOMBS {
            return;
        }

        let bomb = ActiveBomb {
            position: location.into(),
            velocity: INITIAL_BOMB_VELOCITY,
            life_s: BOMB_DELAY_S,
        };
        self.active_bombs.push(bomb);
    }

    /// returns locations to be checked by other systems
    pub fn update(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        asset_manager: &AssetManager,
        delta: f32,
    ) -> HashSet<Location> {
        let mut explosion_at = vec![];
        for bomb in &mut self.active_bombs {
            Self::update_bomb(bomb, world, delta);
            if bomb.life_s <= 0.0 {
                explosion_at.push(bomb.position);
                //TODO: Play sound
            }
        }
        self.explosions.retain(|e| e.life_s > 0.0);
        self.active_bombs.retain(|b| b.life_s > 0.0);
        self.handle_explosions(explosion_at, world, renderer, asset_manager)
    }

    pub fn draw(&self, renderer: &Renderer) {
        for bomb in &self.active_bombs {
            //TODO: alternate between 2 models
            let mesh = renderer
                .get_mesh_generator()
                .generate_mesh_for_falling_voxel(Voxel::Bomb, bomb.position);

            draw_mesh(&mesh);
        }

        for explosion in &self.explosions {
            draw_mesh(&explosion.mesh);
        }
    }

    fn update_bomb(bomb: &mut ActiveBomb, world: &mut World, delta: f32) {
        bomb.life_s -= delta;
        bomb.velocity += (GRAVITY * delta).min(MAX_FALL_SPEED);
        bomb.position.z += bomb.velocity * delta;

        let location = vector_to_location(bomb.position + vec3(0.0, 0.0, Voxel::HALF_SIZE));
        let voxel = world.get(location);
        if !voxel.is_solid() {
            return;
        }

        bomb.velocity = 0.0;
        bomb.position.z = location.z as f32 - 1.0;
    }

    fn handle_explosions(
        &mut self,
        mut explosion_positions: Vec<Vec3>,
        world: &mut World,
        renderer: &mut Renderer,
        asset_manager: &AssetManager,
    ) -> HashSet<Location> {
        let mut locations_to_update = HashSet::new();
        while !explosion_positions.is_empty() {
            let updated_locations = self.explode_at(
                explosion_positions
                    .pop()
                    .expect("Missing explosion location"),
                world,
                asset_manager,
            );
            locations_to_update.extend(updated_locations);
        }

        for loc in &locations_to_update {
            renderer.update_location(world, *loc);
        }

        locations_to_update
    }

    /// returns removed locations
    fn explode_at(
        &mut self,
        position: Vec3,
        world: &mut World,
        asset_manager: &AssetManager,
    ) -> Vec<Location> {
        self.explosions
            .push(Explosion::new(position, &asset_manager.mesh_manager));
        let mut to_update = Vec::with_capacity(64);

        let cx = position.x.floor() as i32;
        let cy = position.y.floor() as i32;
        let cz = position.z.floor() as i32;

        let r = EXPLOSION_RADIUS.ceil() as i32;

        let z_min = (cz - r).max(0);
        let z_max = (cz + r).min(AREA_HEIGHT as i32 - 1);

        for x in (cx - r)..=(cx + r) {
            for y in (cy - r)..=(cy + r) {
                for z in z_min..=z_max {
                    let dx = x as f32 + Voxel::HALF_SIZE - position.x;
                    let dy = y as f32 + Voxel::HALF_SIZE - position.y;
                    let dz = z as f32 + Voxel::HALF_SIZE - position.z;

                    if dx * dx + dy * dy + dz * dz > EXPLOSION_RADIUS_SQ {
                        continue;
                    }

                    let loc = Location::new(x, y, z);
                    let voxel = world.get(loc);
                    if voxel == Voxel::Bomb {
                        self.add_active_bomb_from_explosion(loc);
                    }

                    if voxel.is_solid() {
                        world.set(loc, Voxel::None);
                        to_update.push(loc);
                    }
                }
            }
        }

        to_update
    }

    fn add_active_bomb_from_explosion(&mut self, location: Location) {
        if self.active_bombs.len() >= MAX_ACTIVE_BOMBS {
            return;
        }

        let bomb = ActiveBomb {
            position: location.into(),
            velocity: INITIAL_BOMB_VELOCITY,
            life_s: SHORT_BOMB_DELAY_S,
        };
        self.active_bombs.push(bomb);
    }
}
