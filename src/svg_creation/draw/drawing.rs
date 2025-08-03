use bevy::prelude::*;
use crate::draw_input::*;
use crate::draw_logic::*;
use crate::draw_state::*;
use crate::draw_undo_delete::*;
use crate::draw_visual::*;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PendingSegments::default())
            .insert_resource(DrawingInfo::default())
            .insert_resource(DrawingPoints::default())
            .insert_resource(FollowConfig { speed: 2.0 })
            .insert_resource(DrawingTimer::default())
            .add_systems(Update, (
                drawing_control_system,
                undo_last_drawing,
                drawing_confirmation_system,
                drawing_cooldown_system,
            ))
            .add_systems(FixedUpdate, (
                drawing,
                draw_lines,
            ));
    }
}
