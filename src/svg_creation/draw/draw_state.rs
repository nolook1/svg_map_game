use bevy::prelude::*;
use std::time::Duration;
use crate::svg_creation::spatial_grid::PathSegment;

/// Tracks in-progress points and entities for drawn lines.
#[derive(Resource, Default)]
pub struct DrawingPoints {
    pub points: Vec<Vec2>,
    pub line_entities: Vec<Entity>,
}

/// Pending segments not yet finalized into the spatial grid.
#[derive(Resource, Default)]
pub struct PendingSegments {
    pub segments: Vec<PathSegment>,
}

/// Drawing state and all necessary info.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawingFinalizeState {
    None,
    Cooldown,
}

impl Default for DrawingFinalizeState {
    fn default() -> Self {
        DrawingFinalizeState::None
    }
}

#[derive(Resource, Default)]
pub struct DrawingInfo {
    pub counter: usize,
    pub last_pos: Option<Vec2>,
    pub last_mouse_pos: Option<Vec2>,
    pub last_mouse_time: Option<f32>,
    pub snapped_seg_id: Option<usize>,
    pub is_drawing: bool,
    pub is_paused: bool,
    pub started_from_seg_id: Option<usize>,
    pub finalize_state: DrawingFinalizeState,
    pub cooldown_timer: Option<Timer>,
    pub drawing_enabled: bool,
    pub start_candidate: Option<Vec2>,
    pub confirm_pending: bool,
    pub confirm_point: Option<Vec2>,
    pub confirm_seg_id: Option<usize>,
    pub confirm_prompt_printed: bool,
}

#[derive(Resource, Default)]
pub struct FollowConfig {
    pub speed: f32,
}

#[derive(Resource)]
pub struct DrawingTimer {
    pub timer: Timer,
}

impl Default for DrawingTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(64), TimerMode::Repeating),
        }
    }
}
