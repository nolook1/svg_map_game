use bevy::prelude::*;
use bevy_svg::prelude::*;
use std::path::PathBuf;
use std::io::Write;

use crate::{
    draw_state::{DrawingPoints, DrawingInfo},
    math_utils::bounding_box,
};

pub fn draw_svg(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    filename: PathBuf,
    svg_pos: Vec2,
) -> Entity {
    let filename_str = filename.strip_prefix("assets/").unwrap().to_owned();
    let svg = asset_server.load(filename_str);
    let transform = Transform::from_translation(Vec3::new(svg_pos.x, svg_pos.y, 0.0));
    commands.spawn((Svg2d(svg), transform, Origin::Center)).id()
}

pub fn save_svg(
    drawing_points: &mut ResMut<DrawingPoints>,
    drawing_info: &ResMut<DrawingInfo>,
) -> (PathBuf, f32, f32, f32, f32) {
    let points = &drawing_points.points;
    let (min_x, min_y, max_x, max_y) = if let Some((min, max)) = bounding_box(points) {
        (min.x, min.y, max.x, max.y)
    } else {
        println!("Bounding Box Empty!");
        return (
            PathBuf::new(),
            0.0, 0.0, 0.0, 0.0
        );
    };

    let filename = format!("assets/svgs/drawing{}.svg", drawing_info.counter);
    let path = std::path::Path::new(&filename);

    if !path.exists() {
        std::fs::create_dir_all("assets/svgs").unwrap();
    }

    let mut file = std::fs::File::create(&path).unwrap();
    let _ = writeln!(
        file,
        "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}'>",
        max_x - min_x,
        max_y - min_y
    );
    let _ = write!(file, "<path d='");

    let mut move_next = true;
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            move_next = true;
            continue;
        }
        let x = point.x - min_x;
        let y = max_y - point.y;
        if move_next {
            let _ = write!(file, "M {} {} ", x, y);
            move_next = false;
        } else {
            let _ = write!(file, "L {} {} ", x, y);
        }
    }
    let _ = write!(file, "' fill='none' stroke='black' stroke-width=\"0.5\"/>");
    let _ = write!(file, "</svg>");

    (path.to_path_buf(), min_x, min_y, max_x, max_y)
}
