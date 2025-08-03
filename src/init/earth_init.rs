use bevy::prelude::*;
use std::path::PathBuf;
use regex::Regex;

use crate::{
    spatial_grid::{PathSegment, SpatialGrid},
    svg_utils::draw_svg
};

pub struct WorldInitPlugin;

impl Plugin for WorldInitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, world_setup);
    }
}

fn parse_viewbox(svg: &str) -> Option<(f32, f32, f32, f32)> {
    // Basic regex for: viewBox="minx miny width height"
    let re = Regex::new(r#"viewBox\s*=\s*"([\d\.\-]+) ([\d\.\-]+) ([\d\.\-]+) ([\d\.\-]+)""#).ok()?;
    let caps = re.captures(svg)?;
    Some((
        caps[1].parse().ok()?,
        caps[2].parse().ok()?,
        caps[3].parse().ok()?,
        caps[4].parse().ok()?,
    ))
}

pub fn world_setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut spatial_grid: ResMut<SpatialGrid>,
) {
    let filename = PathBuf::from("assets/earth/earthBorder.svg");
    let svg_position = Vec2::new(0.0, 0.0);
    let _entity = draw_svg(commands, asset_server, filename.clone(), svg_position);

    if let Ok(svg_data) = std::fs::read_to_string(&filename) {
        let viewbox = parse_viewbox(&svg_data).unwrap_or((0.0, 0.0, 5000.0, 3000.0));
        let (_min_x, _min_y, vb_width, vb_height) = viewbox;

        let mut segments = Vec::new();

        // Regex for all d="..." or d='...'
        let re_single = Regex::new(r"d\s*=\s*'([^']*)'").unwrap();
        let re_double = Regex::new(r#"d\s*=\s*"([^"]*)""#).unwrap();

        for cap in re_single.captures_iter(&svg_data) {
            let d_part = &cap[1];
            parse_path_segments(d_part, vb_width, vb_height, &mut segments);
        }
        for cap in re_double.captures_iter(&svg_data) {
            let d_part = &cap[1];
            parse_path_segments(d_part, vb_width, vb_height, &mut segments);
        }

        for i in 0..10 {
            if let Some(seg) = segments.get(i) {
                println!("Segment {}: start {:?}, end {:?}", i, seg.start, seg.end);
            }
        }
        spatial_grid.segments.extend(segments);
        spatial_grid.rebuild_grid();

        println!(
            "Grid: min {:?} max {:?} cols {} rows {} segments {}",
            spatial_grid.bounds.min,
            spatial_grid.bounds.max,
            spatial_grid.cols,
            spatial_grid.rows,
            spatial_grid.segments.len()
        );
        println!("Loaded original.svg with {} segments", spatial_grid.segments.len());
    } else {
        println!("Failed to load original.svg for grid integration.");
    }
}

fn parse_path_segments(
    d_part: &str,
    vb_width: f32,
    vb_height: f32,
    segments: &mut Vec<PathSegment>,
) {
    let coords: Vec<&str> = d_part
        .split(|c| c == 'M' || c == 'L')
        .filter(|s| !s.trim().is_empty())
        .collect();

    let mut id = segments.len();
    let mut last_point = None;

    for part in coords {
        let nums: Vec<f32> = part
            .split_whitespace()
            .filter_map(|n| n.parse::<f32>().ok())
            .collect();
        if nums.len() == 2 {
            // Center and flip coordinates
            let current = Vec2::new(
                nums[0] - vb_width / 2.0,
                vb_height / 2.0 - nums[1]
            );
            if let Some(prev) = last_point {
                segments.push(PathSegment {
                    start: prev,
                    end: current,
                    id,
                });
                id += 1;
            }
            last_point = Some(current);
        }
    }
}
