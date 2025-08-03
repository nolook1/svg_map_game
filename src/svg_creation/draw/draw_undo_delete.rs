use bevy::prelude::*;
use super::draw_state::{DrawingPoints, PendingSegments};

const UNDO_COUNT: usize = 30;

pub fn undo_last_drawing(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut drawing_points: ResMut<DrawingPoints>,
    mut pending_segments: ResMut<PendingSegments>,
) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        for _ in 0..UNDO_COUNT {
            if !drawing_points.points.is_empty() {
                drawing_points.points.pop();
            }
            if !pending_segments.segments.is_empty() {
                pending_segments.segments.pop();
            }
        }
    }
}
