use std::collections::HashMap;

use macroquad::{
    audio::{Sound, load_sound, play_sound_once},
    prelude::{error, info},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SoundId {
    Fall,
    Destroy,
    Place,
}

const SOUNDS: [(SoundId, &str); 3] = [
    (SoundId::Fall, "resources/sounds/fall.wav"),
    (SoundId::Destroy, "resources/sounds/destroy.wav"),
    (SoundId::Place, "resources/sounds/place.wav"),
];

pub struct SoundManager {
    sounds: HashMap<SoundId, Sound>,
}
impl SoundManager {
    /// loads sounds from files
    pub async fn new() -> Self {
        let mut sounds = HashMap::new();
        for (id, path) in SOUNDS {
            let sound = load_sound(path)
                .await
                .expect(&format!("Failed to load '{path}'"));
            info!("Loaded sound with id {:?} from '{}'", id, path);
            sounds.insert(id, sound);
        }

        Self { sounds }
    }

    pub fn play_sound(&self, sound_id: SoundId) {
        if let Some(sound) = self.sounds.get(&sound_id) {
            play_sound_once(sound);
        } else {
            error!("Failed to find sound for {:?}", sound_id)
        }
    }
}
