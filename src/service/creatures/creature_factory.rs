use macroquad::math::Vec3;

use crate::{
    graphics::mesh_manager::MeshManager,
    service::creatures::{
        bunny_creature::BunnyCreature,
        butterfly_creature::ButterflyCreature,
        creature_manager::{Creature, CreatureDTO, CreatureId},
    },
};

pub fn create_creature(
    id: CreatureId,
    position: Vec3,
    mesh_manager: &MeshManager,
) -> Box<dyn Creature> {
    match id {
        CreatureId::Bunny => Box::new(BunnyCreature::new(position, mesh_manager)),
        CreatureId::Butterfly => Box::new(ButterflyCreature::new(position, mesh_manager)),
    }
}

/// returns None if the dto is invalid
pub fn create_creature_from_dto(
    dto: CreatureDTO,
    mesh_manager: &MeshManager,
) -> Option<Box<dyn Creature>> {
    match dto.id {
        CreatureId::Bunny => BunnyCreature::from_dto(dto, mesh_manager),
        CreatureId::Butterfly => BunnyCreature::from_dto(dto, mesh_manager),
    }
}
