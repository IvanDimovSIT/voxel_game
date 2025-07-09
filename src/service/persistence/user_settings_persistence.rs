use rayon::spawn;

use crate::{
    model::user_settings::UserSettings,
    service::persistence::generic_persistence::{read_binary_object, write_binary_object},
};

const USER_SETTINGS_FILEPATH: &str = "settings.dat";

pub fn read_or_initialise_user_settings() -> UserSettings {
    read_binary_object(USER_SETTINGS_FILEPATH).unwrap_or_default()
}

pub fn write_user_settings_blocking(user_settings: &UserSettings) {
    let _result = write_binary_object(USER_SETTINGS_FILEPATH, &user_settings);
}

/// non blocking write
pub fn write_user_settings(user_settings: &UserSettings) {
    let user_settings_copy = user_settings.clone();
    spawn(move || {
        let _result = write_binary_object(USER_SETTINGS_FILEPATH, &user_settings_copy);
    });
}
