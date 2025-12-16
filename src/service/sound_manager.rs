use std::collections::HashMap;

use macroquad::{
    audio::{PlaySoundParams, Sound, load_sound, play_sound, play_sound_once, stop_sound},
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
    Thunder,
    Music,
}

const BASE_SOUNDS_PATH: &str = "assets/sounds/";
const SOUNDS: [(SoundId, &str); 7] = [
    (SoundId::Fall, "fall.wav"),
    (SoundId::Destroy, "destroy.wav"),
    (SoundId::Place, "place.wav"),
    (SoundId::Click, "click.wav"),
    (SoundId::Bounce, "bounce.wav"),
    (SoundId::Thunder, "thunder.wav"),
    (SoundId::Music, "music.ogg"),
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

    /// starts or stops the in-game music based on user settings
    pub fn start_or_stop_music(&self, user_settings: &UserSettings) {
        if !user_settings.has_sound {
            self.stop_music();
            return;
        }

        if let Some(sound) = self.sounds.get(&SoundId::Music) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    ..Default::default()
                },
            );
        } else {
            error!("Failed to find sound for {:?}", SoundId::Music)
        }
    }

    pub fn stop_music(&self) {
        if let Some(sound) = self.sounds.get(&SoundId::Music) {
            stop_sound(sound);
        } else {
            error!("Failed to find sound for {:?}", SoundId::Music)
        }
    }

    pub fn play_sound_for_collision(
        &self,
        collision_type: CollisionType,
        user_settings: &UserSettings,
    ) {
        match collision_type {
            CollisionType::Bounce => self.play_sound(SoundId::Bounce, user_settings),
            CollisionType::Strong { voxel: _ } => self.play_sound(SoundId::Fall, user_settings),
            _ => {}
        }
    }
}
