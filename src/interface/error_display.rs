use macroquad::{
    input::is_key_down,
    math::vec2,
    text::draw_text,
    window::{clear_background, next_frame},
};

use crate::{
    interface::style::{BACKGROUND_COLOR, TEXT_COLOR},
    service::asset_manager::{AssetError, AssetLoadingErrors},
};

const LARGE_FONT: f32 = 28.0;
const SMALL_FONT: f32 = 20.0;
const X_OFFSET: f32 = 2.0;

pub struct ErrorDisplay {
    errors: Vec<String>,
}
impl ErrorDisplay {
    pub fn new(asset_errors: AssetLoadingErrors) -> Self {
        let errors = asset_errors
            .errors
            .into_iter()
            .map(Self::map_asset_error_to_text)
            .collect();

        Self { errors }
    }

    /// returns true if the game should exit
    pub async fn next_frame(&mut self) -> bool {
        clear_background(BACKGROUND_COLOR);

        draw_text(
            "ESC to exit",
            X_OFFSET,
            LARGE_FONT + 1.0,
            LARGE_FONT,
            TEXT_COLOR,
        );
        draw_text(
            "Error loading assets:",
            X_OFFSET,
            LARGE_FONT * 2.0,
            LARGE_FONT,
            TEXT_COLOR,
        );

        let start = vec2(X_OFFSET, LARGE_FONT * 3.0);

        for (i, error) in self.errors.iter().enumerate() {
            let x = start.x;
            let y = start.y + SMALL_FONT * i as f32;
            draw_text(error, x, y, SMALL_FONT, TEXT_COLOR);
        }

        let should_exit = is_key_down(macroquad::input::KeyCode::Escape);
        next_frame().await;

        should_exit
    }

    fn map_asset_error_to_text(asset_error: AssetError) -> String {
        match asset_error {
            AssetError::MissingImage { path } => format!("Missing image: '{path}'"),
            AssetError::MissingSound { path } => format!("Missing sound: '{path}'"),
            AssetError::MissingModel { path } => format!("Missing 3D model: '{path}'"),
            AssetError::ModelFileMustContainASingleModel { path } => {
                format!("Model file '{path}' contains more than one model or is empty")
            }
        }
    }
}
