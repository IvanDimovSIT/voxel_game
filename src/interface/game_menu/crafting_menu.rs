use std::{cell::RefCell, collections::HashMap, fmt::Write, rc::Rc};

use macroquad::{
    camera::set_default_camera,
    color::WHITE,
    input::{MouseButton, is_mouse_button_released, mouse_position},
    math::vec2,
    miniquad::window::screen_size,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        game_menu::game_menu::MenuSelection,
        style::{BACKGROUND_COLOR, BUTTON_HOVER_COLOR, SECONDARY_TEXT_COLOR, TEXT_COLOR},
        util::{
            darken_background, draw_centered_multiline_text, draw_item_name, draw_rect_with_shadow,
            get_text_width, is_point_in_rect,
        },
    },
    model::{
        inventory::{Inventory, Item},
        user_settings::UserSettings,
        voxel::Voxel,
    },
    service::{
        crafting::{CraftingRecipe, find_craftable},
        input::{ScrollDirection, get_scroll_direction},
        sound_manager::{SoundId, SoundManager},
    },
    utils::use_str_buffer,
};

const BUFFER_ERROR: &str = "error writing to buffer in crafting menu";

const CARDS_PER_PAGE: usize = 3;

const CARD_WIDTH: f32 = 0.6;
const CARD_HEIGHT: f32 = 0.2;
const CARD_OFFSET_PX: f32 = 4.0;
const CARD_Y_COEF: f32 = 0.2;

const ROW_PADDING: f32 = 4.0;

const INPUT_ITEM_START_OFFSET_X: f32 = 4.0;
const INPUT_ITEM_START_OFFSET_Y: f32 = 3.0;

const MENU_WIDTH_OF_CARD_WIDTH: f32 = 1.1;
const MENU_HEIGHT_OF_CARD_HEIGHT: f32 = 1.1;

const TOP_MENU_Y_MARGIN_PX: f32 = 6.0;

const MULTILINE_Y_OFFSET_OF_FONT: f32 = 1.5;
const MULTILINE_FONT_SIZE_OF_MENU_SIZE: f32 = 0.1;
const NO_CRAFTABLE_MULTILINE_TEXT: [&str; 2] = ["No items", "can be crafted"];

const PAGES_COUNTER_OFFSET_X: f32 = 3.0;
const PAGES_COUNTER_FONT_SIZE: f32 = 0.07;
const PAGES_COUNTER_OFFSET_Y: f32 = 8.0;

#[derive(Debug, Clone)]
pub struct CraftingMenuContext {
    available_recipes: Vec<CraftingRecipe>,
    all_items: HashMap<Voxel, u32>,
    current_page: usize,
}
impl CraftingMenuContext {
    pub fn new(inventory: &Inventory) -> Rc<RefCell<Self>> {
        let all_items = inventory.create_all_items_map();
        Rc::new(RefCell::new(Self {
            available_recipes: find_craftable(&all_items),
            current_page: 0,
            all_items,
        }))
    }

    pub fn draw_menu(
        &mut self,
        inventory: &mut Inventory,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> MenuSelection {
        set_default_camera();
        let (width, height) = screen_size();
        darken_background(width, height);

        let menu_width = width * CARD_WIDTH * MENU_WIDTH_OF_CARD_WIDTH;
        let menu_height = CARDS_PER_PAGE as f32
            * (CARD_OFFSET_PX + height * CARD_HEIGHT * MENU_HEIGHT_OF_CARD_HEIGHT);
        let menu_x = (width - menu_width) / 2.0;
        let menu_y = height * CARD_Y_COEF - TOP_MENU_Y_MARGIN_PX;
        draw_rect_with_shadow(menu_x, menu_y, menu_width, menu_height, BACKGROUND_COLOR);
        self.handle_pages(menu_x, menu_y, menu_height);

        let to_craft = self.handle_crafting_cards(
            texture_manager,
            sound_manager,
            user_settings,
            width,
            menu_width,
            menu_height,
            menu_y,
        );

        if let Some(recipe) = to_craft {
            self.craft_item(recipe, inventory);
        }

        MenuSelection::None
    }

    fn craft_item(&mut self, recipe: CraftingRecipe, inventory: &mut Inventory) {
        for input in recipe.get_inputs() {
            inventory.remove_item(*input);
        }
        inventory.add_item(recipe.output);
        self.all_items = inventory.create_all_items_map();
        self.available_recipes = find_craftable(&self.all_items);
        self.current_page = self.current_page.min(self.calculate_max_page());
    }

    /// draws the crafting cards and returns the map of crafted items
    fn handle_crafting_cards(
        &mut self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
        width: f32,
        menu_width: f32,
        menu_height: f32,
        menu_y: f32,
    ) -> Option<CraftingRecipe> {
        let crafted: Vec<_> = self
            .available_recipes
            .iter()
            .skip(self.current_page * CARDS_PER_PAGE)
            .take(CARDS_PER_PAGE)
            .enumerate()
            .map(|(index, recipe)| {
                (
                    *recipe,
                    self.draw_crafting_card(
                        texture_manager,
                        sound_manager,
                        user_settings,
                        recipe,
                        index,
                    ),
                )
            })
            .collect();

        if self.available_recipes.is_empty() {
            let font = menu_height.min(menu_width) * MULTILINE_FONT_SIZE_OF_MENU_SIZE;
            let text_y = menu_y + font * MULTILINE_Y_OFFSET_OF_FONT;
            draw_centered_multiline_text(
                &NO_CRAFTABLE_MULTILINE_TEXT,
                text_y,
                width,
                font,
                TEXT_COLOR,
            );
        }

        crafted
            .iter()
            .find(|(_, should_craft)| *should_craft)
            .map(|(recipe, _)| *recipe)
    }

    fn calculate_max_page(&self) -> usize {
        let partially_full = if self.available_recipes.len() % CARDS_PER_PAGE >= 1 {
            1
        } else {
            0
        };
        ((self.available_recipes.len() / CARDS_PER_PAGE) + partially_full).max(1) - 1
    }

    /// returns true if should craft
    fn draw_crafting_card(
        &self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
        crafting_recipe: &CraftingRecipe,
        index: usize,
    ) -> bool {
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();

        let card_width = width * CARD_WIDTH;
        let card_height = height * CARD_HEIGHT;
        let card_x = (width - card_width) / 2.0;
        let card_y = height * CARD_Y_COEF + (index as f32) * (CARD_OFFSET_PX + card_height);

        let is_hovered =
            is_point_in_rect(card_x, card_y, card_width, card_height, mouse_x, mouse_y);
        let background_color = if is_hovered {
            BUTTON_HOVER_COLOR
        } else {
            WHITE
        };

        draw_rect_with_shadow(card_x, card_y, card_width, card_height, background_color);
        let item_x_start = card_x + INPUT_ITEM_START_OFFSET_X;
        let item_y_start = card_y + INPUT_ITEM_START_OFFSET_Y;
        let input_item_size = card_height / 4.0;

        for (index, input) in crafting_recipe.get_inputs().enumerate() {
            let item_y = item_y_start + index as f32 * (input_item_size + ROW_PADDING);
            let available = self.all_items.get(&input.voxel).copied().unwrap_or(0);
            Self::draw_item_input(
                item_x_start,
                item_y,
                input_item_size,
                input,
                available,
                texture_manager.get_icon(input.voxel),
            );
        }

        const OUTPUT_ITEM_SIZE_RELATIVE_TO_INPUT: f32 = 1.4;
        const OUTPUT_ITEM_Y_OFFSET: f32 = 4.0;
        let output_item_size = input_item_size * OUTPUT_ITEM_SIZE_RELATIVE_TO_INPUT;
        let already_have = self
            .all_items
            .get(&crafting_recipe.output.voxel)
            .copied()
            .unwrap_or(0);
        Self::draw_output_item(
            card_x + card_width - output_item_size * 1.5,
            item_y_start + OUTPUT_ITEM_Y_OFFSET,
            output_item_size,
            &crafting_recipe.output,
            already_have,
            texture_manager.get_icon(crafting_recipe.output.voxel),
        );

        if is_hovered && is_mouse_button_released(MouseButton::Left) {
            sound_manager.play_sound(SoundId::Click, user_settings);
            true
        } else {
            false
        }
    }

    /// draws the output item for a recipe
    fn draw_output_item(
        x: f32,
        y: f32,
        size: f32,
        item: &Item,
        already_have: u32,
        texture: Texture2D,
    ) {
        const FONT_SIZE_COEF: f32 = 0.8;
        const TEXT_X_OFFSET: f32 = 5.0;
        let (mouse_x, mouse_y) = mouse_position();
        let font_size = size * FONT_SIZE_COEF;
        let buffer = format!("makes {}X", item.count);
        let text_width = get_text_width(&buffer, font_size);
        let text_x = x - text_width - size - TEXT_X_OFFSET;
        draw_text(
            &buffer,
            text_x,
            y + font_size,
            font_size,
            SECONDARY_TEXT_COLOR,
        );
        let texture_x = x - size;
        draw_texture_ex(
            &texture,
            texture_x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
        if is_point_in_rect(texture_x, y, size, size, mouse_x, mouse_y) {
            draw_item_name(
                mouse_x,
                mouse_y,
                item.voxel.display_name(),
                item.count,
                font_size,
            );
        }

        let next_line_y = y + size;
        use_str_buffer(|buffer| {
            write!(buffer, "(have {already_have})").expect(BUFFER_ERROR);

            let next_line_text_y = next_line_y + font_size;
            draw_text(
                buffer,
                text_x,
                next_line_text_y,
                font_size,
                SECONDARY_TEXT_COLOR,
            );
        });
    }

    /// draws the input items for a recipe
    fn draw_item_input(x: f32, y: f32, size: f32, item: &Item, available: u32, texture: Texture2D) {
        const INPUT_ITEM_FONT_SIZE: f32 = 0.9;
        const TEXT_OFFSET_X: f32 = 3.0;
        let (mouse_x, mouse_y) = mouse_position();
        draw_texture_ex(
            &texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
        let font_size = size * INPUT_ITEM_FONT_SIZE;
        let text_start_x = x + size + TEXT_OFFSET_X;
        let text_start_y = y + font_size;
        if is_point_in_rect(x, y, size, size, mouse_x, mouse_y) {
            draw_item_name(
                mouse_x,
                mouse_y,
                item.voxel.display_name(),
                item.count,
                font_size,
            );
        }
        use_str_buffer(|buffer| {
            write!(buffer, "X{} (have {})", item.count, available).expect(BUFFER_ERROR);
            draw_text(
                buffer,
                text_start_x,
                text_start_y,
                font_size,
                SECONDARY_TEXT_COLOR,
            );
        });
    }

    /// draws the page counter and manages page scrolling
    fn handle_pages(&mut self, menu_x: f32, menu_y: f32, menu_height: f32) {
        let max_page = self.calculate_max_page();
        self.draw_pages_counter(menu_x, menu_y, menu_height, max_page);

        match get_scroll_direction() {
            ScrollDirection::Down => {
                self.current_page = (self.current_page + 1).min(max_page);
            }
            ScrollDirection::Up => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                }
            }
            ScrollDirection::None => {}
        }
    }

    fn draw_pages_counter(&self, menu_x: f32, menu_y: f32, menu_height: f32, max_page: usize) {
        let x = menu_x + PAGES_COUNTER_OFFSET_X;
        let font_size = menu_height * PAGES_COUNTER_FONT_SIZE;
        let y = menu_y + menu_height - PAGES_COUNTER_OFFSET_Y;

        use_str_buffer(|buffer| {
            write!(buffer, "Page {}/{}", self.current_page + 1, max_page + 1).expect(BUFFER_ERROR);
            draw_text(buffer, x, y, font_size, TEXT_COLOR);
        });
    }
}
