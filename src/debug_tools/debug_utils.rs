use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::svg_creation::spatial_grid::SpatialGrid;

#[derive(Resource, Default)]
pub struct GridDebugOverlayState {
    pub visible: bool,
}

#[derive(Component)]
pub struct GridDebugDraw;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GridDebugOverlayState::default())
            .add_systems(Update, (draw_spatial_grid_overlay, grid_debug_toggle_system));
    }
}

pub fn grid_debug_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GridDebugOverlayState>,
    mut commands: Commands,
    query: Query<Entity, With<GridDebugDraw>>,
) {
    if keys.just_pressed(KeyCode::F12) {
        state.visible = !state.visible;
        for e in query.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

pub fn draw_spatial_grid_overlay(
    mut commands: Commands,
    state: Res<GridDebugOverlayState>,
    spatial_grid: Res<SpatialGrid>,
    query: Query<Entity, With<GridDebugDraw>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    if !state.visible {
        return;
    }

    let (_, cam_transform) = camera_query.single();
    let cam_pos = cam_transform.translation().truncate();
    let view_width = 1600.0; 
    let view_height = 900.0;
    let view_min = cam_pos - Vec2::new(view_width, view_height) / 2.0;
    let view_max = cam_pos + Vec2::new(view_width, view_height) / 2.0;

    let cell_color = Srgba::new(0.0, 0.7, 0.2, 0.18);
    let cell_border = Srgba::rgb(0.0, 0.7, 0.2);
    let mut cell_path_builder = PathBuilder::new();

    let cell_size = spatial_grid.cell_size;
    let min = spatial_grid.bounds.min;
    let cols = spatial_grid.cols;
    let rows = spatial_grid.rows;

    for col in 0..cols {
        let x = min.x + col as f32 * cell_size;
        for row in 0..rows {
            let y = min.y + row as f32 * cell_size;
            let rect_min = Vec2::new(x, y);
            let rect_max = rect_min + Vec2::new(cell_size, cell_size);

            if rect_max.x < view_min.x || rect_min.x > view_max.x ||
               rect_max.y < view_min.y || rect_min.y > view_max.y {
                continue;
            }

            cell_path_builder.move_to(rect_min);
            cell_path_builder.line_to(Vec2::new(rect_max.x, rect_min.y));
            cell_path_builder.line_to(rect_max);
            cell_path_builder.line_to(Vec2::new(rect_min.x, rect_max.y));
            cell_path_builder.close();
        }
    }

    let cell_path = cell_path_builder.build();
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&cell_path),
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        },
        Fill::color(cell_color),
        Stroke::new(cell_border, 1.0),
        GridDebugDraw,
    ));

    let seg_color = Srgba::rgb(0.8, 0.0, 0.8); // Purple
    let mut seg_path_builder = PathBuilder::new();

    for seg in &spatial_grid.segments {
        if seg.start == seg.end { continue; }
        if (seg.start.x < view_min.x && seg.end.x < view_min.x) ||
           (seg.start.x > view_max.x && seg.end.x > view_max.x) ||
           (seg.start.y < view_min.y && seg.end.y < view_min.y) ||
           (seg.start.y > view_max.y && seg.end.y > view_max.y) {
            continue;
        }
        seg_path_builder.move_to(seg.start);
        seg_path_builder.line_to(seg.end);
    }

    let seg_path = seg_path_builder.build();
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&seg_path),
            transform: Transform::from_xyz(0.0, 0.0, 12.0),
            ..default()
        },
        Stroke::new(seg_color, 1.7),
        GridDebugDraw,
    ));
}
