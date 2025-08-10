use macroquad::{
    color::WHITE,
    input::{MouseButton, is_mouse_button_pressed, is_mouse_button_released, mouse_position},
    math::vec2,
    miniquad::window::screen_size,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    graphics::{texture_manager::TextureManager, ui_display::VOXEL_SELECTION_SIZE},
    interface::{
        game_menu::game_menu::MenuSelection,
        style::{BACKGROUND_COLOR, SHADOW_COLOR, TEXT_COLOR},
        util::{darken_background, draw_rect_with_shadow, get_text_width},
    },
    model::{player_info::PlayerInfo, voxel::Voxel},
    service::input::exit_focus,
};

const VOXELS_IN_SELECTION_MENU: [Option<Voxel>; 14] = [
    Some(Voxel::Stone),
    Some(Voxel::Sand),
    Some(Voxel::Grass),
    Some(Voxel::Wood),
    Some(Voxel::Leaves),
    Some(Voxel::Brick),
    Some(Voxel::Dirt),
    Some(Voxel::Boards),
    Some(Voxel::Cobblestone),
    Some(Voxel::Clay),
    Some(Voxel::Lamp),
    Some(Voxel::Trampoline),
    Some(Voxel::Glass),
    Some(Voxel::Cactus),
];

const VOXEL_SIZE: f32 = 0.08;
const INNER_VOXELS_MULTIPLIER: f32 = 0.8;
const BORDER_VOXELS_MULTIPLIER: f32 = (1.0 - INNER_VOXELS_MULTIPLIER) / 2.0;
const VOXELS_IN_ROW: usize = VOXEL_SELECTION_SIZE;
const VOXELS_IN_COLUMN: usize = 5;
const SELECTED_VOXELS_OFFSET: f32 = 0.6;

fn get_voxel_at_index(index: usize) -> Option<Voxel> {
    if index < VOXELS_IN_SELECTION_MENU.len() {
        VOXELS_IN_SELECTION_MENU[index]
    } else {
        None
    }
}

/// returns the new menu state and voxel selection
pub fn draw_voxel_selection_menu(
    texture_manager: &TextureManager,
    player_info: &mut PlayerInfo,
    mut selected: Option<Voxel>,
) -> (Option<Voxel>, MenuSelection) {
    debug_assert_ne!(selected, Some(Voxel::None));
    let (width, height) = screen_size();
    darken_background(width, height);

    let voxel_size = VOXEL_SIZE * width.min(height);
    let menu_width = VOXELS_IN_ROW as f32 * voxel_size;
    let menu_height =
        VOXELS_IN_COLUMN as f32 * voxel_size + SELECTED_VOXELS_OFFSET * voxel_size + voxel_size;
    let menu_x = (width - menu_width) * 0.5;
    let menu_y = (height - menu_height) * 0.5;
    draw_rect_with_shadow(menu_x, menu_y, menu_width, menu_height, BACKGROUND_COLOR);

    draw_inventory_voxels(texture_manager, voxel_size, menu_x, menu_y);
    draw_selected_voxels(texture_manager, player_info, voxel_size, menu_x, menu_y);
    draw_hovered_voxel_name(player_info, voxel_size, menu_x, menu_y);

    if let Some(selected_voxel) = selected {
        draw_held_voxel(texture_manager, voxel_size, selected_voxel);
    }

    if is_mouse_button_pressed(MouseButton::Left)
        || (is_mouse_button_released(MouseButton::Left) && selected.is_some())
    {
        if let Some(some_voxel) = selected {
            selected = None;
            set_voxel_in_selection(menu_x, menu_y, voxel_size, player_info, some_voxel);
        } else {
            selected = get_voxel_from_menu(menu_x, menu_y, voxel_size, player_info);
        }
    }

    if exit_focus() {
        (selected, MenuSelection::BackToGame)
    } else {
        (selected, MenuSelection::None)
    }
}

fn draw_hovered_voxel_name(player_info: &PlayerInfo, voxel_size: f32, menu_x: f32, menu_y: f32) {
    const TEXT_BOX_X_OFFSET: f32 = 3.0;
    const TEXT_BOX_Y_OFFSET: f32 = -5.0;
    if let Some(hovered) = get_hovered_voxel(menu_x, menu_y, voxel_size, player_info) {
        let voxel_name = match hovered {
            HoveredVoxel::Inventory(voxel) => voxel,
            HoveredVoxel::Selection(index) => {
                if let Some(voxel) = player_info.voxel_selector.get_at(index) {
                    voxel
                } else {
                    return;
                }
            }
        }
        .display_name();

        let (x, y) = mouse_position();
        let font_size = voxel_size * 0.5;
        draw_rectangle(
            x,
            y - font_size,
            get_text_width(voxel_name, font_size) + TEXT_BOX_X_OFFSET,
            font_size,
            SHADOW_COLOR,
        );
        draw_text(
            voxel_name,
            x + TEXT_BOX_X_OFFSET,
            y + TEXT_BOX_Y_OFFSET,
            font_size,
            TEXT_COLOR,
        );
    }
}

fn get_voxel_from_menu(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &mut PlayerInfo,
) -> Option<Voxel> {
    get_hovered_voxel(menu_x, menu_y, voxel_size, player_info).and_then(|hovered| match hovered {
        HoveredVoxel::Inventory(voxel) => Some(voxel),
        HoveredVoxel::Selection(index) => {
            let selected_voxel = player_info.voxel_selector.get_at(index);
            player_info.voxel_selector.set_at(index, None);
            selected_voxel
        }
    })
}

enum HoveredVoxel {
    Inventory(Voxel),
    Selection(usize),
}
fn get_hovered_voxel(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &PlayerInfo,
) -> Option<HoveredVoxel> {
    let (mouse_x, mouse_y) = mouse_position();
    let x = ((mouse_x - menu_x) / voxel_size).floor() as i32;
    if x < 0 || x >= VOXELS_IN_ROW as i32 {
        return None;
    }

    let inventory_y = ((mouse_y - menu_y) / voxel_size).floor() as i32;
    if inventory_y >= 0 && inventory_y < VOXELS_IN_COLUMN as i32 {
        let index = x as usize + inventory_y as usize * VOXELS_IN_ROW;
        if (0..VOXELS_IN_SELECTION_MENU.len()).contains(&index) {
            return VOXELS_IN_SELECTION_MENU[index].map(HoveredVoxel::Inventory);
        } else {
            return None;
        }
    }

    let selection_y = mouse_y
        - (menu_y + SELECTED_VOXELS_OFFSET * voxel_size + VOXELS_IN_COLUMN as f32 * voxel_size);
    if selection_y >= 0.0 && selection_y < voxel_size {
        let found_voxel = player_info.voxel_selector.get_at(x as usize);
        return found_voxel.map(|_| HoveredVoxel::Selection(x as usize));
    }

    None
}

fn set_voxel_in_selection(
    menu_x: f32,
    menu_y: f32,
    voxel_size: f32,
    player_info: &mut PlayerInfo,
    selected_voxel: Voxel,
) {
    debug_assert_ne!(selected_voxel, Voxel::None);
    let (mouse_x, mouse_y) = mouse_position();
    let x = ((mouse_x - menu_x) / voxel_size).floor() as i32;
    if x < 0 || x >= VOXELS_IN_ROW as i32 {
        return;
    }

    let y = mouse_y
        - (menu_y + SELECTED_VOXELS_OFFSET * voxel_size + VOXELS_IN_COLUMN as f32 * voxel_size);
    if y < 0.0 || y >= voxel_size {
        return;
    }
    player_info
        .voxel_selector
        .set_at(x as usize, Some(selected_voxel));
}

fn draw_held_voxel(texture_manager: &TextureManager, voxel_size: f32, selected_voxel: Voxel) {
    debug_assert_ne!(selected_voxel, Voxel::None);
    let (mouse_x, mouse_y) = mouse_position();
    let texture = texture_manager.get_icon(selected_voxel);
    draw_voxel_texture(&texture, voxel_size, mouse_x, mouse_y);
}

fn draw_voxel_texture(texture: &Texture2D, voxel_size: f32, x: f32, y: f32) {
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
        let option_voxel = player_info.voxel_selector.get_at(x);
        if let Some(voxel) = option_voxel {
            let texture = texture_manager.get_icon(voxel);
            draw_voxel_texture(
                &texture,
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
    voxel_size: f32,
    menu_x: f32,
    menu_y: f32,
) {
    for y in 0..VOXELS_IN_COLUMN {
        for x in 0..VOXELS_IN_ROW {
            let index = y * VOXELS_IN_ROW + x;
            let x_pos = menu_x + x as f32 * voxel_size + voxel_size * BORDER_VOXELS_MULTIPLIER;
            let y_pos = menu_y + y as f32 * voxel_size + voxel_size * BORDER_VOXELS_MULTIPLIER;

            if let Some(voxel) = get_voxel_at_index(index) {
                let texture = texture_manager.get_icon(voxel);
                draw_voxel_texture(&texture, voxel_size, x_pos, y_pos);
                if Voxel::TRANSPARENT.contains(&voxel) {
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
