use std::collections::HashMap;

use macroquad::{
    audio::{Sound, load_sound, play_sound_once},
    prelude::{error, info},
};

use crate::model::user_settings::UserSettings;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SoundId {
    Fall,
    Destroy,
    Place,
    Click,
}

const SOUNDS: [(SoundId, &str); 4] = [
    (SoundId::Fall, "resources/sounds/fall.wav"),
    (SoundId::Destroy, "resources/sounds/destroy.wav"),
    (SoundId::Place, "resources/sounds/place.wav"),
    (SoundId::Click, "resources/sounds/click.wav"),
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
}
