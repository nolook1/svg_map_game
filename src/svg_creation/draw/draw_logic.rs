use bevy::prelude::*;
use crate::settings::world_mouse_pos;
use crate::svg_creation::{
    spatial_grid::{SpatialGrid, SnapState, PathSegment},
    merge_svg::*,
    math_utils::{segment_intersection_point},
};
use super::draw_state::*;

const MIN_MOUSE_VELOCITY: f32 = 1.0; // px/sec
const SNAP_RADIUS: f32 = 15.0;
const BLOCK_RADIUS: f32 = 7.0;

pub fn drawing(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: Query<&mut Window>,
    mut drawing_info: ResMut<DrawingInfo>,
    mut drawing_points: ResMut<DrawingPoints>,
    mut svg_library: ResMut<SvgLibrary>,
    mut spatial_grid: ResMut<SpatialGrid>,
    mut pending_segments: ResMut<PendingSegments>,
    mut snap_state: ResMut<SnapState>,
    cameras: Query<(&Camera, &Transform)>,
    config: Res<FollowConfig>,
    mut drawing_timer: ResMut<DrawingTimer>,
    time: Res<Time>,
) {
    let mut window = windows.single_mut();

    if !(drawing_info.is_drawing && !drawing_info.is_paused) {
        return;
    }

    // Block drawing if confirmation prompt is pending
    if drawing_info.confirm_pending {
        if !drawing_info.confirm_prompt_printed {
            println!("Finish drawing at border? (Y/N)");
            drawing_info.confirm_prompt_printed = true;
        }
        return;
    }

    if !drawing_timer.timer.tick(time.delta()).just_finished() {
        return;
    }

    let mouse_pos = world_mouse_pos(&mut window, &cameras);
    let now = time.elapsed_secs();

    let mut should_add_point = false;
    match drawing_info.last_mouse_pos {
        Some(last_mouse_pos) => {
            let last_time = drawing_info.last_mouse_time.unwrap_or(now);
            let dt = (now - last_time).max(0.0001);
            let velocity = (mouse_pos - last_mouse_pos).length() / dt;
            if velocity > MIN_MOUSE_VELOCITY {
                should_add_point = true;
            }
        }
        None => {
            should_add_point = true;
        }
    }

    if !should_add_point {
        return;
    }

    // Border block logic
    if let Some((seg, nearest_point)) = spatial_grid.query_nearest_segment(mouse_pos, SNAP_RADIUS) {
        let near_start = nearest_point.distance(seg.start) < SNAP_RADIUS;
        let near_end = nearest_point.distance(seg.end) < SNAP_RADIUS;
        let near_segment = nearest_point.distance(mouse_pos) < BLOCK_RADIUS;
        if near_segment && !(near_start || near_end) {
            return;
        }
    }

    let new_pos = compute_new_pos(&drawing_info, mouse_pos, config.speed);

    if let Some(last) = drawing_info.last_pos {
        for seg in &spatial_grid.segments {
            if let Some(hit) = segment_intersection_point(last, new_pos, seg.start, seg.end) {
                let start_dist = hit.distance(seg.start);
                let end_dist = hit.distance(seg.end);
                let snapped_point = if start_dist < end_dist { seg.start } else { seg.end };

                if !drawing_info.confirm_pending {
                    drawing_info.confirm_pending = true;
                    drawing_info.confirm_point = Some(snapped_point);
                    drawing_info.confirm_seg_id = Some(seg.id);
                    drawing_info.confirm_prompt_printed = false;
                }
                return;
            }
        }
    }

    update_drawing_points(
        &mut drawing_info,
        &mut drawing_points,
        &mut pending_segments,
        new_pos,
        mouse_pos,
        now,
    );

    let snapped_id = drawing_info.snapped_seg_id;
    let initial_id = snap_state.initial_seg_id;
    if drawing_points.points.len() < 2 {
        return;
    }
    if let (Some(current_id), Some(start_id)) = (snapped_id, initial_id) {
        let blocked = snap_state.blocked_range;
        let outside_block =
            current_id < start_id.saturating_sub(blocked) || current_id > start_id + blocked;

        if outside_block {
            finalize_svg_drawing(
                commands,
                asset_server,
                &mut drawing_info,
                &mut drawing_points,
                &mut svg_library,
                &mut spatial_grid,
                &mut snap_state,
            );
        }
    }
}

pub fn drawing_confirmation_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut drawing_info: ResMut<DrawingInfo>,
    mut drawing_points: ResMut<DrawingPoints>,
    mut svg_library: ResMut<SvgLibrary>,
    mut spatial_grid: ResMut<SpatialGrid>,
    mut snap_state: ResMut<SnapState>,
    asset_server: Res<AssetServer>,
    commands: Commands,
) {
    if drawing_info.confirm_pending {
        if keys.just_pressed(KeyCode::KeyY) {
            if let Some(confirm_point) = drawing_info.confirm_point {
                if let Some(last) = drawing_points.points.last_mut() {
                    *last = confirm_point;
                }
            }
            finalize_svg_drawing(
                commands,
                asset_server,
                &mut drawing_info,
                &mut drawing_points,
                &mut svg_library,
                &mut spatial_grid,
                &mut snap_state,
            );
            drawing_info.confirm_pending = false;
            drawing_info.confirm_point = None;
            drawing_info.confirm_seg_id = None;
            drawing_info.confirm_prompt_printed = false;
        }
        if keys.just_pressed(KeyCode::KeyN) {
            drawing_info.confirm_pending = false;
            drawing_info.confirm_point = None;
            drawing_info.confirm_seg_id = None;
            drawing_info.confirm_prompt_printed = false;
        }
    }
}

pub fn drawing_cooldown_system(
    time: Res<Time>,
    mut drawing_info: ResMut<DrawingInfo>,
) {
    if drawing_info.finalize_state != DrawingFinalizeState::Cooldown {
        return;
    }
    if let Some(timer) = drawing_info.cooldown_timer.as_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            drawing_info.finalize_state = DrawingFinalizeState::None;
            drawing_info.cooldown_timer = None;
            println!("SVG end cooldown finished, you can now end the SVG.");
        }
    }
}

pub fn finalize_svg_drawing(
    commands: Commands,
    asset_server: Res<AssetServer>,
    drawing_info: &mut ResMut<DrawingInfo>,
    drawing_points: &mut ResMut<DrawingPoints>,
    svg_library: &mut ResMut<SvgLibrary>,
    spatial_grid: &mut ResMut<SpatialGrid>,
    snap_state: &mut ResMut<SnapState>,
) {
    let mut new_segments = Vec::new();

    if let Some(first) = drawing_points.points.first_mut() {
        if let Some((closest_pt, _seg_id)) = spatial_grid.query_nearest_point(
            *first,
            15.0,
            None,
            0,
        ) {
            if first.distance(closest_pt) < 15.0 {
                *first = closest_pt;
            }
        }
    }

    if let Some(confirm_point) = drawing_info.confirm_point {
        if let Some(last) = drawing_points.points.last_mut() {
            *last = confirm_point;
        }
    }

    for pair in drawing_points.points.windows(2) {
        if let [start, end] = pair {
            let id = spatial_grid.segments.len() + new_segments.len();
            new_segments.push(PathSegment {
                start: *start,
                end: *end,
                id,
            });
        }
    }

    check_and_merge_svg(
        commands,
        asset_server,
        drawing_info,
        drawing_points,
        svg_library,
    );

    spatial_grid.segments.extend(new_segments);
    spatial_grid.rebuild_grid();

    snap_state.is_blocking = false;
    snap_state.initial_seg_id = None;
    snap_state.initial_snap_pos = None;

    drawing_info.snapped_seg_id = None;
    drawing_points.points.clear();
    drawing_info.last_pos = None;
    drawing_info.last_mouse_pos = None;
    drawing_info.last_mouse_time = None;
    drawing_info.is_drawing = false;
    drawing_info.is_paused = false;
    drawing_info.confirm_pending = false;
    drawing_info.confirm_point = None;
    drawing_info.confirm_seg_id = None;
    drawing_info.confirm_prompt_printed = false;
    drawing_info.started_from_seg_id = None;
    drawing_info.start_candidate = None;
}

pub fn update_drawing_points(
    drawing_info: &mut ResMut<DrawingInfo>,
    drawing_points: &mut ResMut<DrawingPoints>,
    pending_segments: &mut ResMut<PendingSegments>,
    new_pos: Vec2,
    mouse_pos: Vec2,
    now: f32,
) {
    if let Some(last) = drawing_info.last_pos {
        let segment = PathSegment {
            start: last,
            end: new_pos,
            id: pending_segments.segments.len(),
        };
        pending_segments.segments.push(segment);
    }
    drawing_points.points.push(new_pos);
    drawing_info.last_pos = Some(new_pos);
    drawing_info.last_mouse_pos = Some(mouse_pos);
    drawing_info.last_mouse_time = Some(now);
}

pub fn compute_new_pos(
    drawing_info: &ResMut<DrawingInfo>,
    mouse_pos: Vec2,
    speed: f32,
) -> Vec2 {
    drawing_info.last_pos.map_or(mouse_pos, |last_pos| {
        let direction = mouse_pos - last_pos;
        if direction.length() > 0.0 {
            last_pos + direction.normalize() * speed
        } else {
            last_pos
        }
    })
}
