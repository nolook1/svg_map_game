use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use super::draw_state::DrawingPoints;
use crate::svg_creation::math_utils::smooth_lines;

/// Draws lines for the current in-progress user drawing.
pub fn draw_lines(
    mut commands: Commands,
    mut drawing_points: ResMut<DrawingPoints>,
) {
    let window_size = 3;
    let smoothed_points = smooth_lines(&drawing_points.points, window_size);

    for entity in drawing_points.line_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for points in smoothed_points.windows(2) {
        let line = shapes::Line(points[0], points[1]);
        let entity = commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&line),
                ..default()
            },
            Stroke::new(Srgba::rgb(0.0, 0.0, 0.0), 0.5),
        )).id();
        drawing_points.line_entities.push(entity);
    }
}
