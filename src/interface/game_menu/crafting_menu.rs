use std::{cell::RefCell, fmt::Write, rc::Rc};

use macroquad::{
    camera::set_default_camera,
    color::WHITE,
    input::{MouseButton, is_mouse_button_released, mouse_position},
    math::vec2,
    miniquad::window::screen_size,
    text::Font,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    graphics::screen_effects::darken_background,
    interface::{
        game_menu::game_menu_context::MenuSelection,
        style::{
            BACKGROUND_COLOR, BUTTON_HOVER_COLOR, SECONDARY_TEXT_COLOR, SELECTED_COLOR, TEXT_COLOR,
        },
        util::{
            draw_centered_multiline_text, draw_game_text, draw_item_name, draw_rect_with_shadow,
            get_text_width, is_point_in_rect,
        },
    },
    model::{
        inventory::{AvailableItems, Inventory, Item},
        user_settings::UserSettings,
    },
    service::{
        asset_manager::AssetManager,
        crafting::{CraftingRecipe, craft_recipe, find_craftable},
        input::{ScrollDirection, get_scroll_direction},
        sound_manager::SoundId,
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
const CARD_ITEM_START_OFFSET_Y: f32 = 3.0;
const CARD_FONT_SIZE_COEF: f32 = 0.9;
const OUTPUT_ITEM_OFFSET_X_COEF: f32 = 1.5;
const MAX_CARD_WIDTH_RELATIVE_TO_HEIGHT: f32 = 4.0;

const MENU_WIDTH_OF_CARD_WIDTH: f32 = 1.1;
const MENU_HEIGHT_OF_CARD_HEIGHT: f32 = 1.1;

const TOP_MENU_Y_MARGIN_PX: f32 = 6.0;

const MULTILINE_Y_OFFSET_OF_FONT: f32 = 1.5;
const MULTILINE_FONT_SIZE_OF_MENU_SIZE: f32 = 0.1;
const NO_CRAFTABLE_MULTILINE_TEXT: [&str; 2] = ["No items", "can be crafted"];

const PAGES_COUNTER_OFFSET_X: f32 = 3.0;
const PAGES_COUNTER_FONT_SIZE: f32 = 0.07;
const PAGES_COUNTER_OFFSET_Y: f32 = 8.0;

const BULK_CRAFT_COUNT: u32 = 5;

pub type CraftingMenuHandle = Rc<RefCell<CraftingMenuContext>>;

#[derive(Debug, Clone)]
pub struct CraftingMenuContext {
    available_recipes: Vec<(CraftingRecipe, u32)>,
    all_items: AvailableItems,
    current_page: usize,
}
impl CraftingMenuContext {
    pub fn new(inventory: &Inventory) -> CraftingMenuHandle {
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
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> MenuSelection {
        set_default_camera();
        let (width, height) = screen_size();
        darken_background(width, height);

        let (menu_width, menu_height) = Self::calculate_menu_size(width, height);
        let menu_x = (width - menu_width) / 2.0;
        let menu_y = height * CARD_Y_COEF - TOP_MENU_Y_MARGIN_PX;
        draw_rect_with_shadow(menu_x, menu_y, menu_width, menu_height, BACKGROUND_COLOR);
        self.handle_pages(menu_x, menu_y, menu_height, &asset_manager.font);

        let to_craft = self.handle_crafting_cards(
            asset_manager,
            user_settings,
            width,
            menu_width,
            menu_height,
            menu_y,
        );

        if let Some((recipe, count)) = to_craft {
            self.craft_item(recipe, inventory, count);
        }

        MenuSelection::None
    }

    fn craft_item(&mut self, recipe: CraftingRecipe, inventory: &mut Inventory, count: u32) {
        debug_assert!(count <= BULK_CRAFT_COUNT);
        craft_recipe(&recipe, inventory, count as u8);

        self.all_items = inventory.create_all_items_map();
        self.available_recipes = find_craftable(&self.all_items);
        self.current_page = self.current_page.min(self.calculate_max_page());
    }

    /// draws the crafting cards and returns the number of crafted items
    fn handle_crafting_cards(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        width: f32,
        menu_width: f32,
        menu_height: f32,
        menu_y: f32,
    ) -> Option<(CraftingRecipe, u32)> {
        let crafted: Vec<_> = self
            .available_recipes
            .iter()
            .skip(self.current_page * CARDS_PER_PAGE)
            .take(CARDS_PER_PAGE)
            .enumerate()
            .map(|(index, (recipe, count))| {
                (
                    *recipe,
                    self.draw_crafting_card(asset_manager, user_settings, recipe, *count, index),
                )
            })
            .filter(|(_, count)| *count > 0)
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
                &asset_manager.font,
            );
        }

        crafted.first().cloned()
    }

    fn calculate_max_page(&self) -> usize {
        let partially_full = if self.available_recipes.len() % CARDS_PER_PAGE >= 1 {
            1
        } else {
            0
        };
        ((self.available_recipes.len() / CARDS_PER_PAGE) + partially_full).max(1) - 1
    }

    /// returns the number of times to craft
    fn draw_crafting_card(
        &self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        crafting_recipe: &CraftingRecipe,
        max_craftable_count: u32,
        index: usize,
    ) -> u32 {
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();

        let (card_width, card_height) = Self::calculate_card_size(width, height);
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
        let item_y_start = card_y + CARD_ITEM_START_OFFSET_Y;
        let item_size = card_height / 4.0;

        for (index, input) in crafting_recipe.get_inputs().enumerate() {
            let item_y = item_y_start + index as f32 * (item_size + ROW_PADDING);
            let available = self.all_items.get(input.voxel);
            Self::draw_item_input(
                item_x_start,
                item_y,
                item_size,
                input,
                available,
                asset_manager.texture_manager.get_icon(input.voxel),
                &asset_manager.font,
            );
        }

        let already_have = self.all_items.get(crafting_recipe.output.voxel);
        Self::draw_output_item(
            card_x + card_width - item_size * OUTPUT_ITEM_OFFSET_X_COEF,
            item_y_start,
            item_size,
            &crafting_recipe.output,
            max_craftable_count,
            already_have,
            asset_manager,
        );

        Self::handle_card_click(
            is_hovered,
            max_craftable_count,
            asset_manager,
            user_settings,
        )
    }

    /// returns the number of times to craft
    fn handle_card_click(
        is_hovered: bool,
        max_craftable_count: u32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> u32 {
        if !is_hovered {
            return 0;
        }

        if is_mouse_button_released(MouseButton::Left) {
            asset_manager
                .sound_manager
                .play_sound(SoundId::Click, user_settings);
            1
        } else if is_mouse_button_released(MouseButton::Right) {
            asset_manager
                .sound_manager
                .play_sound(SoundId::Click, user_settings);
            BULK_CRAFT_COUNT.min(max_craftable_count)
        } else {
            0
        }
    }

    /// draws the output item for a recipe
    fn draw_output_item(
        x: f32,
        y: f32,
        size: f32,
        item: &Item,
        count: u32,
        already_have: u32,
        asset_manager: &AssetManager,
    ) {
        let texture = asset_manager.texture_manager.get_icon(item.voxel);
        const TEXT_X_OFFSET: f32 = 5.0;
        let (mouse_x, mouse_y) = mouse_position();
        let font_size = size * CARD_FONT_SIZE_COEF;
        let buffer = format!("Makes {}X", item.count);
        let text_width = get_text_width(&buffer, font_size, &asset_manager.font);
        let text_x = x - text_width - size - TEXT_X_OFFSET;

        draw_game_text(
            &buffer,
            text_x,
            y + font_size,
            font_size,
            SELECTED_COLOR,
            &asset_manager.font,
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

        let second_line_y = y + size;
        use_str_buffer(|buffer| {
            let can_craft_count = count as u8 * item.count;
            write!(buffer, "Can craft {can_craft_count}").expect(BUFFER_ERROR);

            let second_line_text_y = second_line_y + font_size;
            draw_game_text(
                buffer,
                text_x,
                second_line_text_y,
                font_size,
                SECONDARY_TEXT_COLOR,
                &asset_manager.font,
            );
        });

        let third_line_y = second_line_y + size;
        use_str_buffer(|buffer| {
            write!(buffer, "(have {already_have})").expect(BUFFER_ERROR);
            let third_line_text_y = third_line_y + font_size;
            draw_game_text(
                buffer,
                text_x,
                third_line_text_y,
                font_size,
                SECONDARY_TEXT_COLOR,
                &asset_manager.font,
            );
        });

        if is_point_in_rect(texture_x, y, size, size, mouse_x, mouse_y) {
            draw_item_name(
                mouse_x,
                mouse_y,
                item.voxel.display_name(),
                item.count,
                font_size,
                &asset_manager.font,
            );
        }
    }

    /// draws the input items for a recipe
    fn draw_item_input(
        x: f32,
        y: f32,
        size: f32,
        item: &Item,
        available: u32,
        texture: Texture2D,
        font: &Font,
    ) {
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

        let font_size = size * CARD_FONT_SIZE_COEF;
        let text_start_x = x + size + TEXT_OFFSET_X;
        let text_start_y = y + font_size;
        use_str_buffer(|buffer| {
            write!(buffer, "X{}", item.count).expect(BUFFER_ERROR);
            let count_offset = get_text_width(buffer, font_size, font);
            draw_game_text(
                buffer,
                text_start_x,
                text_start_y,
                font_size,
                SELECTED_COLOR,
                font,
            );

            buffer.clear();
            write!(buffer, " (have {})", available).expect(BUFFER_ERROR);
            draw_game_text(
                buffer,
                text_start_x + count_offset,
                text_start_y,
                font_size,
                SECONDARY_TEXT_COLOR,
                font,
            );
        });

        if is_point_in_rect(x, y, size, size, mouse_x, mouse_y) {
            draw_item_name(
                mouse_x,
                mouse_y,
                item.voxel.display_name(),
                item.count,
                font_size,
                font,
            );
        }
    }

    /// draws the page counter and manages page scrolling
    fn handle_pages(&mut self, menu_x: f32, menu_y: f32, menu_height: f32, font: &Font) {
        let max_page = self.calculate_max_page();
        self.draw_pages_counter(menu_x, menu_y, menu_height, max_page, font);

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

    fn draw_pages_counter(
        &self,
        menu_x: f32,
        menu_y: f32,
        menu_height: f32,
        max_page: usize,
        font: &Font,
    ) {
        let x = menu_x + PAGES_COUNTER_OFFSET_X;
        let font_size = menu_height * PAGES_COUNTER_FONT_SIZE;
        let y = menu_y + menu_height - PAGES_COUNTER_OFFSET_Y;

        use_str_buffer(|buffer| {
            write!(
                buffer,
                "Page {}/{} (mouse scroll)",
                self.current_page + 1,
                max_page + 1
            )
            .expect(BUFFER_ERROR);
            draw_game_text(buffer, x, y, font_size, TEXT_COLOR, font);
        });
    }

    fn calculate_card_size(width: f32, height: f32) -> (f32, f32) {
        let card_height = height * CARD_HEIGHT;
        let card_width = (width * CARD_WIDTH).min(MAX_CARD_WIDTH_RELATIVE_TO_HEIGHT * card_height);

        (card_width, card_height)
    }

    fn calculate_menu_size(width: f32, height: f32) -> (f32, f32) {
        let (card_width, card_height) = Self::calculate_card_size(width, height);
        let menu_width = card_width * MENU_WIDTH_OF_CARD_WIDTH;
        let menu_height =
            CARDS_PER_PAGE as f32 * (card_height * MENU_HEIGHT_OF_CARD_HEIGHT + CARD_OFFSET_PX);

        (menu_width, menu_height)
    }
}
