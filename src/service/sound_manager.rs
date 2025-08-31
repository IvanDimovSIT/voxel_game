use std::collections::HashMap;

use macroquad::{
    audio::{Sound, load_sound, play_sound_once},
    prelude::{error, info},
};

use crate::{model::user_settings::UserSettings, service::physics::player_physics::CollisionType};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SoundId {
    Fall,
    Destroy,
    Place,
    Click,
    Bounce,
}

const BASE_SOUNDS_PATH: &str = "assets/sounds/";
const SOUNDS: [(SoundId, &str); 5] = [
    (SoundId::Fall, "fall.wav"),
    (SoundId::Destroy, "destroy.wav"),
    (SoundId::Place, "place.wav"),
    (SoundId::Click, "click.wav"),
    (SoundId::Bounce, "bounce.wav"),
];

pub struct SoundManager {
    sounds: HashMap<SoundId, Sound>,
}
impl SoundManager {
    /// loads sounds from files
    pub async fn new() -> Self {
        let mut sounds = HashMap::new();
        for (id, path) in SOUNDS {
            let full_path = format!("{BASE_SOUNDS_PATH}{path}");
            let sound = load_sound(&full_path)
                .await
                .unwrap_or_else(|_| panic!("Failed to load '{path}'"));
            info!("Loaded sound with id {:?} from '{}'", id, path);
            sounds.insert(id, sound);
        }

        Self { sounds }
    }

    pub fn play_sound(&self, sound_id: SoundId, user_settings: &UserSettings) {
        if !user_settings.has_sound {
            return;
        }

        if let Some(sound) = self.sounds.get(&sound_id) {
            play_sound_once(sound);
        } else {
            error!("Failed to find sound for {:?}", sound_id)
        }
    }

    pub fn play_sound_for_collision(
        &self,
        collision_type: CollisionType,
        user_settings: &UserSettings,
    ) {
        match collision_type {
            CollisionType::Bounce => self.play_sound(SoundId::Bounce, user_settings),
            CollisionType::Strong => self.play_sound(SoundId::Fall, user_settings),
            _ => {}
        }
    }
}
