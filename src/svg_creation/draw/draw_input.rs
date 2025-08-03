use bevy::prelude::*;
use super::draw_state::DrawingInfo;

pub fn drawing_control_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut drawing_info: ResMut<DrawingInfo>,
) {
    if !drawing_info.drawing_enabled {
        return;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if !drawing_info.is_drawing {
            drawing_info.is_drawing = true;
            drawing_info.is_paused = false;
            println!("Drawing started");
        } else {
            drawing_info.is_paused = !drawing_info.is_paused;
            println!("Drawing {}", if drawing_info.is_paused { "paused" } else { "resumed" });
        }
    }
}
