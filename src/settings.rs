use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

use crate::spatial_grid::SnapState;
use crate::draw_state::DrawingInfo;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_settings_config)
            .add_systems(Update, (
                camera_movement_system,
                camera_zoom_system,
                snap_toggle_system,
                drawing_toggle_system,
            ));
    }
}

#[derive(Resource, Default)]
pub struct SettingsConfig {
    pub translation_speed: f32,
    pub zoom_speed: f32,
}

fn setup_settings_config(mut commands: Commands) {
    commands.insert_resource(SettingsConfig {
        translation_speed: 250.0,
        zoom_speed:0.1,
    });
}

fn camera_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Camera, &mut Transform)>,
    time: Res<Time>,
    drawing_config: Res<SettingsConfig>,
) {
    for (_camera, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let speed = drawing_config.translation_speed;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        transform.translation += time.delta_secs() * speed * direction;
    }
}

pub fn snap_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut snap_state: ResMut<SnapState>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        snap_state.is_enabled = !snap_state.is_enabled;
        println!(
            "Snapping is now {}",
            if snap_state.is_enabled { "ENABLED" } else { "DISABLED" }
        );
    }
}

pub fn drawing_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut drawing_info: ResMut<DrawingInfo>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        drawing_info.drawing_enabled = !drawing_info.drawing_enabled;
        println!(
            "Drawing is now {}",
            if drawing_info.drawing_enabled { "ENABLED" } else { "DISABLED" }
        );
    }
}

fn camera_zoom_system(
    mut query: Query<(&Camera, &mut Transform)>,
    mut scroll_evr: EventReader<MouseWheel>,
    drawing_config: Res<SettingsConfig>,
) {
    let zoom_speed = drawing_config.zoom_speed;
    let mut zoom_delta: f32 = 0.0;

    for ev in scroll_evr.read() {
        zoom_delta += ev.y;
    }

    if zoom_delta.abs() > 0.0 {
        for (_camera, mut transform) in query.iter_mut() {
            let current_scale = transform.scale.x;
            let new_scale = (current_scale * (1.0 - zoom_delta * zoom_speed)).clamp(0.2, 5.0);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

/// Converts mouse cursor position to world coordinates.
pub fn world_mouse_pos(
    window: &mut Window,
    cameras: &Query<(&Camera, &Transform)>,
) -> Vec2 {
    let (_camera, camera_transform) = cameras.single();
    let pos = window.cursor_position().unwrap_or_default();
    let size = Vec2::new(window.width() as f32, window.height() as f32);
    let adjusted_point = Vec2::new(pos.x, size.y - pos.y) - size / 2.0;
    let world_pos = camera_transform.compute_matrix() * adjusted_point.extend(0.0).extend(1.0);
    Vec2::new(world_pos.x, world_pos.y)
}
