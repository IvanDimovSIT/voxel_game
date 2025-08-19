use std::fmt::Write;

use macroquad::{
    color::WHITE,
    input::{MouseButton, is_mouse_button_released, mouse_position},
    math::vec2,
    miniquad::window::screen_size,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        game_menu::game_menu::MenuSelection,
        style::{BACKGROUND_COLOR, SHADOW_COLOR, TEXT_COLOR},
        util::{darken_background, draw_item_name, draw_rect_with_shadow},
    },
    model::{
        inventory::{Inventory, Item, MAX_ITEMS_PER_SLOT},
        player_info::PlayerInfo,
        voxel::Voxel,
    },
    service::input::exit_focus,
    utils::use_str_buffer,
};

const VOXEL_SIZE: f32 = 0.08;
const INNER_VOXELS_MULTIPLIER: f32 = 0.8;
const BORDER_VOXELS_MULTIPLIER: f32 = (1.0 - INNER_VOXELS_MULTIPLIER) / 2.0;
const VOXELS_IN_ROW: usize = Inventory::SELECTED_SIZE;
const VOXELS_IN_COLUMN: usize = Inventory::INVENTORY_SIZE / VOXELS_IN_ROW;
const SELECTED_VOXELS_OFFSET: f32 = 0.6;
const BASE_COUNT_FONT_SIZE: f32 = 0.5;

enum ItemSource {
    Inventory,
    Selection,
}
struct HoveredItem {
    index: usize,
    source: ItemSource,
}
impl HoveredItem {
    fn inventory(index: usize) -> Self {
        Self {
            index,
            source: ItemSource::Inventory,
        }
    }

    fn selection(index: usize) -> Self {
        Self {
            index,
            source: ItemSource::Selection,
        }
    }

    fn set(&self, inventory: &mut Inventory, item: Option<Item>) {
        match self.source {
            ItemSource::Inventory => inventory.items[self.index] = item,
            ItemSource::Selection => inventory.selected[self.index] = item,
        }
    }

    fn get(&self, inventory: &Inventory) -> Option<Item> {
        match self.source {
            ItemSource::Inventory => inventory.items[self.index],
            ItemSource::Selection => inventory.selected[self.index],
        }
    }
}

/// returns the new menu state and voxel selection
pub fn draw_voxel_selection_menu(
    texture_manager: &TextureManager,
    player_info: &mut PlayerInfo,
    mut selected: Option<Item>,
) -> (Option<Item>, MenuSelection) {
    debug_assert!(selected.is_none() || selected.unwrap().voxel != Voxel::None);
    let (width, height) = screen_size();
    darken_background(width, height);

    let voxel_size = VOXEL_SIZE * width.min(height);
    let menu_width = VOXELS_IN_ROW as f32 * voxel_size;
    let menu_height =
        VOXELS_IN_COLUMN as f32 * voxel_size + SELECTED_VOXELS_OFFSET * voxel_size + voxel_size;
    let menu_x = (width - menu_width) * 0.5;
    let menu_y = (height - menu_height) * 0.5;
    draw_rect_with_shadow(menu_x, menu_y, menu_width, menu_height, BACKGROUND_COLOR);

    draw_inventory_voxels(texture_manager, player_info, voxel_size, menu_x, menu_y);
    draw_selected_voxels(texture_manager, player_info, voxel_size, menu_x, menu_y);
    draw_hovered_item_name(player_info, voxel_size, menu_x, menu_y);

    if let Some(selected_item) = selected {
        draw_held_item(texture_manager, voxel_size, selected_item);
    }

    if is_mouse_button_released(MouseButton::Left) {
        if let Some(some_item) = selected {
            selected = set_voxel_in_selection(menu_x, menu_y, voxel_size, player_info, some_item);
        } else {
            selected = get_item_from_menu(menu_x, menu_y, voxel_size, player_info);
        }
    }

    if exit_focus() {
        (selected, MenuSelection::BackToGame)
    } else {
        (selected, MenuSelection::None)
    }
}

fn draw_hovered_item_name(player_info: &PlayerInfo, voxel_size: f32, menu_x: f32, menu_y: f32) {
    if let Some(hovered) = get_hovered_item(menu_x, menu_y, voxel_size, player_info) {
        let (voxel_name, count) = if let Some(item) = hovered.get(&player_info.inventory) {
            (item.voxel.display_name(), item.count)
        } else {
            return;
        };

        let (x, y) = mouse_position();
        let font_size = voxel_size * 0.5;
        draw_item_name(x, y, voxel_name, count, font_size);
    }
}

/// returns the currently hovered over item in the slot and sets it to empty
fn get_item_from_menu(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &mut PlayerInfo,
) -> Option<Item> {
    get_hovered_item(menu_x, menu_y, voxel_size, player_info).and_then(|hovered| {
        let item = hovered.get(&player_info.inventory);
        hovered.set(&mut player_info.inventory, None);
        item
    })
}

/// returns the currently hovered over item in the slot
fn get_hovered_item(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &PlayerInfo,
) -> Option<HoveredItem> {
    let (mouse_x, mouse_y) = mouse_position();
    let x = ((mouse_x - menu_x) / voxel_size).floor() as i32;
    if x < 0 || x >= VOXELS_IN_ROW as i32 {
        return None;
    }

    let inventory_y = ((mouse_y - menu_y) / voxel_size).floor() as i32;
    if inventory_y >= 0 && inventory_y < VOXELS_IN_COLUMN as i32 {
        let index = x as usize + inventory_y as usize * VOXELS_IN_ROW;
        if (0..player_info.inventory.items.len()).contains(&index) {
            return Some(HoveredItem::inventory(index));
        } else {
            return None;
        }
    }

    let selection_y = mouse_y
        - (menu_y + SELECTED_VOXELS_OFFSET * voxel_size + VOXELS_IN_COLUMN as f32 * voxel_size);
    if selection_y >= 0.0 && selection_y < voxel_size {
        let index = x as usize;
        return Some(HoveredItem::selection(index));
    }

    None
}

/// puts the held voxel into the hovered slot and returns the replaced item
fn set_voxel_in_selection(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &mut PlayerInfo,
    mut selected_item: Item,
) -> Option<Item> {
    debug_assert_ne!(selected_item.voxel, Voxel::None);
    let hovered = get_hovered_item(menu_x, menu_y, voxel_size, player_info);
    match hovered {
        Some(some_hovered) => {
            let previous = some_hovered
                .get(&player_info.inventory)
                .and_then(|item_in_slot| {
                    if item_in_slot.voxel == selected_item.voxel {
                        let amount_to_transfer =
                            (MAX_ITEMS_PER_SLOT - selected_item.count).min(item_in_slot.count);
                        selected_item.count += amount_to_transfer;
                        if item_in_slot.count == amount_to_transfer {
                            None
                        } else {
                            Some(Item::new(
                                item_in_slot.voxel,
                                item_in_slot.count - amount_to_transfer,
                            ))
                        }
                    } else {
                        Some(item_in_slot)
                    }
                });

            some_hovered.set(&mut player_info.inventory, Some(selected_item));
            previous
        }
        None => {
            player_info.inventory.add_item(selected_item);
            None
        }
    }
}

fn draw_held_item(texture_manager: &TextureManager, voxel_size: f32, selected_item: Item) {
    debug_assert_ne!(selected_item.voxel, Voxel::None);
    let (mouse_x, mouse_y) = mouse_position();
    let texture = texture_manager.get_icon(selected_item.voxel);
    draw_item(&texture, selected_item.count, voxel_size, mouse_x, mouse_y);
}

fn draw_item(texture: &Texture2D, count: u8, voxel_size: f32, x: f32, y: f32) {
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                voxel_size * INNER_VOXELS_MULTIPLIER,
                voxel_size * INNER_VOXELS_MULTIPLIER,
            )),
            ..Default::default()
        },
    );
    let font_size = voxel_size * BASE_COUNT_FONT_SIZE;
    use_str_buffer(|buffer| {
        write!(buffer, "{count}").expect("error writing to text buffer");
        draw_text(buffer, x, y + font_size * 1.5, font_size, TEXT_COLOR);
    });
}

fn draw_selected_voxels(
    texture_manager: &TextureManager,
    player_info: &mut PlayerInfo,
    voxel_size: f32,
    menu_x: f32,
    menu_y: f32,
) {
    let y = menu_y + voxel_size * VOXELS_IN_COLUMN as f32 + SELECTED_VOXELS_OFFSET * voxel_size;
    let text_size = voxel_size * 0.6;
    draw_text("Selected:", menu_x, y, text_size, TEXT_COLOR);

    for x in 0..VOXELS_IN_ROW {
        let option_item = player_info.inventory.selected[x];
        if let Some(item) = option_item {
            let texture = texture_manager.get_icon(item.voxel);
            draw_item(
                &texture,
                item.count,
                voxel_size,
                x as f32 * voxel_size + menu_x + voxel_size * BORDER_VOXELS_MULTIPLIER,
                y + voxel_size * BORDER_VOXELS_MULTIPLIER,
            );
        } else {
            draw_rectangle(
                x as f32 * voxel_size + menu_x + voxel_size * BORDER_VOXELS_MULTIPLIER,
                y + voxel_size * BORDER_VOXELS_MULTIPLIER,
                voxel_size * INNER_VOXELS_MULTIPLIER,
                voxel_size * INNER_VOXELS_MULTIPLIER,
                SHADOW_COLOR,
            );
        }
    }
}

fn draw_inventory_voxels(
    texture_manager: &TextureManager,
    player_info: &mut PlayerInfo,
    voxel_size: f32,
    menu_x: f32,
    menu_y: f32,
) {
    for y in 0..VOXELS_IN_COLUMN {
        for x in 0..VOXELS_IN_ROW {
            let index = y * VOXELS_IN_ROW + x;
            let x_pos = menu_x + x as f32 * voxel_size + voxel_size * BORDER_VOXELS_MULTIPLIER;
            let y_pos = menu_y + y as f32 * voxel_size + voxel_size * BORDER_VOXELS_MULTIPLIER;
            if index >= player_info.inventory.items.len() {
                return;
            }

            if let Some(item) = player_info.inventory.items[index] {
                let texture = texture_manager.get_icon(item.voxel);
                draw_item(&texture, item.count, voxel_size, x_pos, y_pos);
                if Voxel::TRANSPARENT.contains(&item.voxel) {
                    draw_empty_slot(voxel_size, x_pos, y_pos);
                }
            } else {
                draw_empty_slot(voxel_size, x_pos, y_pos);
            }
        }
    }
}

fn draw_empty_slot(voxel_size: f32, x_pos: f32, y_pos: f32) {
    let empty_slot_size = voxel_size * INNER_VOXELS_MULTIPLIER;
    draw_rectangle(x_pos, y_pos, empty_slot_size, empty_slot_size, SHADOW_COLOR);
}
