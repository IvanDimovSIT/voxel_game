use std::collections::HashMap;

use macroquad::{math::Vec3, rand::rand};

use crate::{
    graphics::mesh_manager::MeshManager,
    model::voxel::Voxel,
    service::creatures::{
        bunny_creature::BunnyCreature,
        butterfly_creature::ButterflyCreature,
        creature::Creature,
        creature_manager::{CreatureDTO, CreatureId},
        penguin_creature::PenguinCreature,
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
        CreatureId::Penguin => Box::new(PenguinCreature::new(position, mesh_manager)),
    }
}

/// returns None if the dto is invalid
pub fn create_creature_from_dto(
    dto: CreatureDTO,
    mesh_manager: &MeshManager,
) -> Option<Box<dyn Creature>> {
    match dto.id {
        CreatureId::Bunny => BunnyCreature::from_dto(dto, mesh_manager),
        CreatureId::Butterfly => ButterflyCreature::from_dto(dto, mesh_manager),
        CreatureId::Penguin => PenguinCreature::from_dto(dto, mesh_manager),
    }
}

thread_local! {
    /// registers all allowed spawn voxels for each creature
    static ALLOWED_SPAWN_VOXEL_MAP: HashMap<Voxel, Vec<CreatureId>> = {
        let mut map = HashMap::new();
        add_allowed_voxels(&mut map, CreatureId::Bunny, BunnyCreature::get_allowed_spawn_voxels());
        add_allowed_voxels(&mut map, CreatureId::Butterfly, ButterflyCreature::get_allowed_spawn_voxels());
        add_allowed_voxels(&mut map, CreatureId::Penguin, PenguinCreature::get_allowed_spawn_voxels());

        map
    };
}

fn add_allowed_voxels(map: &mut HashMap<Voxel, Vec<CreatureId>>, id: CreatureId, voxels: &[Voxel]) {
    for voxel in voxels {
        if let Some(vec) = map.get_mut(voxel) {
            vec.push(id);
        } else {
            map.insert(*voxel, vec![id]);
        }
    }
}

/// returns a random Creature id that's allowed to spawn on th voxel
pub fn random_creature_id_for_voxel(voxel: Voxel) -> Option<CreatureId> {
    ALLOWED_SPAWN_VOXEL_MAP.with(|map| {
        let allowed_ids = map
            .get(&voxel)
            .map(|vec| vec.as_slice())
            .unwrap_or([].as_slice());

        if allowed_ids.is_empty() {
            None
        } else {
            Some(allowed_ids[rand() as usize % allowed_ids.len()])
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const ALL_CREATURE_IDS: [CreatureId; 3] = [
        CreatureId::Bunny,
        CreatureId::Butterfly,
        CreatureId::Penguin,
    ];

    #[test]
    fn test_allowed_spawn_voxel_map() {
        ALLOWED_SPAWN_VOXEL_MAP.with(|map| {
            let mut all_creature_ids = HashSet::new();
            for id in ALL_CREATURE_IDS {
                all_creature_ids.insert(id);
            }

            assert!(!map.is_empty());
            for (voxel, ids) in map.iter() {
                assert_ne!(*voxel, Voxel::None);
                assert!(!ids.is_empty());

                let mut seen_ids = HashSet::new();
                for id in ids {
                    assert!(
                        !seen_ids.contains(id),
                        "'ALLOWED_SPAWN_VOXEL_MAP' contains duplicate ids for a voxel"
                    );
                    seen_ids.insert(*id);
                    all_creature_ids.remove(id);
                    assert!(
                        ALL_CREATURE_IDS.contains(id),
                        "'ALL_CREATURE_IDS' constant is outdated"
                    )
                }
            }

            assert!(
                all_creature_ids.is_empty(),
                "not all creatures are spawnable"
            );
        });
    }
}
