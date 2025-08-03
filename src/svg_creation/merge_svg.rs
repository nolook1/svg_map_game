use bevy::prelude::*;

use crate::{
    svg_utils::{draw_svg, save_svg},
    math_utils::{paths_intersect, smooth_lines, ramer_douglas_peucker},
    draw_state::{DrawingInfo, DrawingPoints}
};

#[derive(Component)]
pub struct SvgLine {
    pub path: Vec<Vec2>,
}

#[derive(Resource, Default)]
pub struct SvgLibrary {
    pub lines: Vec<(Entity, SvgLine)>,
}

pub struct MergeSvgPlugin;

impl Plugin for MergeSvgPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SvgLibrary::default());
    }
}

fn filter_close_points(points: &[Vec2], min_dist: f32) -> Vec<Vec2> {
    let mut out = Vec::new();
    for &pt in points {
        if out.last().map_or(true, |last| pt.distance(*last) > min_dist) {
            out.push(pt);
        }
    }
    out
}

pub fn check_and_merge_svg(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    drawing_info: &mut ResMut<DrawingInfo>,
    drawing_points: &mut ResMut<DrawingPoints>,
    svg_library: &mut ResMut<SvgLibrary>,
) -> Vec<Vec2> {
    let window_size = 3; // Try stronger smoothing, might look funny
    let epsilon = 0.2;
    let original_points = drawing_points.points.clone();
    let filtered_points = filter_close_points(&drawing_points.points, 1.0);
    let smoothed_path = smooth_lines(&filtered_points, window_size);
    let simplified = ramer_douglas_peucker(&smoothed_path, epsilon);

    let final_path = if !simplified.is_empty() { simplified } else { smoothed_path.clone() };

    if !original_points.is_empty() && final_path.len() >= 2 {
        let mut path = final_path.clone();
        path[0] = original_points[0];
        let last_idx = path.len() - 1;
        path[last_idx] = original_points[original_points.len() - 1];
        drawing_points.points = path.clone();
        let mut new_path = path.clone();
        new_path.push(Vec2::new(f32::NAN, f32::NAN));
        let mut entities_to_despawn = Vec::new();

        for (entity, line) in &svg_library.lines {
            if paths_intersect(&path, &line.path) {
                new_path.extend(line.path.clone());
                new_path.push(Vec2::new(f32::NAN, f32::NAN));
                entities_to_despawn.push(*entity);
                new_path.push(Vec2::new(f32::NAN, f32::NAN));
            }
        }

        for entity in entities_to_despawn {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }

        drawing_points.points = new_path.clone();
        let (filename, min_x, min_y, max_x, max_y) = save_svg(drawing_points, drawing_info);
        let center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        println!("Saving user SVG to: {:?}", filename);
        if let Ok(contents) = std::fs::read_to_string(&filename) {
            println!("SVG file contents:\n{}", contents.lines().take(5).collect::<Vec<_>>().join("\n"));
        } else {
            println!("SVG file not found: {:?}", filename);
        }
        println!("Spawning SVG at center: {:?}", center);
        let filename_str = filename.strip_prefix("assets/").unwrap().to_owned();
        println!("Loading asset with path: {:?}", filename_str);
        let new_entity = draw_svg(commands, asset_server, filename.clone(), center);

        svg_library.lines.push((
            new_entity,
            SvgLine {
                path: new_path.clone(),
            },
        ));

        let grid_path: Vec<Vec2> = new_path.iter().cloned().filter(|p| p.is_finite()).collect();

        drawing_points.points.clear();
        drawing_info.last_pos = None;
        drawing_info.counter += 1;

        return grid_path;
    } else {
        drawing_points.points.clear();
        drawing_info.last_pos = None;
        drawing_info.counter += 1;
        return Vec::new();
    }
}
