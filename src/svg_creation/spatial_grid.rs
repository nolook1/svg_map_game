use bevy::prelude::*;

use crate::math_utils::closest_point_on_segment;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathSegment {
    pub start: Vec2,
    pub end: Vec2,
    pub id: usize,
}

#[derive(Resource)]
pub struct SpatialGrid {
    pub bounds: Rect,
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Vec<usize>>,
    pub segments: Vec<PathSegment>,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Resource)]
pub struct SnapState {
    pub initial_seg_id: Option<usize>,
    pub blocked_range: usize,
    pub initial_snap_pos: Option<Vec2>,
    pub is_blocking: bool,
    pub is_enabled: bool,
}

pub struct SpatialGridPlugin;

impl Plugin for SpatialGridPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SpatialGrid::new(50.0))
            .insert_resource(SnapState {
                initial_seg_id: None,
                blocked_range: 20,
                initial_snap_pos: None,
                is_blocking: false,
                is_enabled: false,
            });
    }
}

impl SpatialGrid {
    /// Creates a new, empty spatial grid.
    pub fn new(_cell_size: f32) -> Self {
        Self {
            bounds: Rect {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            },
            cell_size: 5.0,
            cols: 0,
            rows: 0,
            cells: Vec::new(),
            segments: Vec::new(),
        }
    }

    /// Set grid bounds explicitly (min, max world coordinates) and recalc grid.
    pub fn set_bounds(&mut self, min: Vec2, max: Vec2) {
        self.bounds.min = min;
        self.bounds.max = max;
        self.recalc_grid();
    }

    /// Set grid bounds automatically from SVG viewBox (min_x, min_y, width, height)
    pub fn set_bounds_from_viewbox(&mut self, viewbox: (f32, f32, f32, f32)) {
        let (_min_x, _min_y, width, height) = viewbox;
        self.bounds.min = Vec2::new(-width / 2.0, -height / 2.0);
        self.bounds.max = Vec2::new(width / 2.0, height / 2.0);
        self.recalc_grid();
    }

    /// Recalculate grid cells from bounds
    pub fn recalc_grid(&mut self) {
        let width = self.bounds.max.x - self.bounds.min.x;
        let height = self.bounds.max.y - self.bounds.min.y;
        self.cols = (width / self.cell_size).ceil() as usize;
        self.rows = (height / self.cell_size).ceil() as usize;
        self.cells = vec![Vec::new(); self.cols * self.rows];
    }

    fn cell_index(&self, col: usize, row: usize) -> usize {
        row * self.cols + col
    }

    fn point_to_cell(&self, point: Vec2) -> Option<(usize, usize)> {
        if point.x < self.bounds.min.x || point.x > self.bounds.max.x || point.y < self.bounds.min.y || point.y > self.bounds.max.y {
            return None;
        }
        let col = ((point.x - self.bounds.min.x) / self.cell_size).floor() as usize;
        let row = ((point.y - self.bounds.min.y) / self.cell_size).floor() as usize;
        if col >= self.cols || row >= self.rows {
            return None;
        }
        Some((col, row))
    }

    /// Removes a segment by its ID.
    pub fn remove_segment(&mut self, id: usize) {
        if id >= self.segments.len() {
            return;
        }
        let segment = self.segments[id];
        let min_x = segment.start.x.min(segment.end.x);
        let max_x = segment.start.x.max(segment.end.x);
        let min_y = segment.start.y.min(segment.end.y);
        let max_y = segment.start.y.max(segment.end.y);

        let start_cell = self.point_to_cell(Vec2::new(min_x, min_y));
        let end_cell = self.point_to_cell(Vec2::new(max_x, max_y));
        if start_cell.is_none() || end_cell.is_none() {
            return;
        }
        let (start_col, start_row) = start_cell.unwrap();
        let (end_col, end_row) = end_cell.unwrap();

        for col in start_col.min(end_col)..=start_col.max(end_col) {
            for row in start_row.min(end_row)..=start_row.max(end_row) {
                let idx = self.cell_index(col, row);
                self.cells[idx].retain(|&seg_id| seg_id != id);
            }
        }
        // Mark segment as empty to keep indexing consistent (optional).
        self.segments[id] = PathSegment {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            id,
        };
    }

    /// Rebuilds the spatial grid and reindexes all segments.
    pub fn rebuild_grid(&mut self) {
        if self.segments.is_empty() {
            println!("No segments to build grid from.");
            return;
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for seg in &self.segments {
            if seg.start == seg.end {
                continue;
            }
            min_x = min_x.min(seg.start.x.min(seg.end.x));
            min_y = min_y.min(seg.start.y.min(seg.end.y));
            max_x = max_x.max(seg.start.x.max(seg.end.x));
            max_y = max_y.max(seg.start.y.max(seg.end.y));
        }

        // Fix: ensure min <= max
        if min_x > max_x || min_y > max_y {
            println!("SpatialGrid: bounds calculation failed (min > max), using fallback bounds.");
            self.bounds.min = Vec2::ZERO;
            self.bounds.max = Vec2::ZERO;
        } else {
            self.bounds.min = Vec2::new(min_x, min_y);
            self.bounds.max = Vec2::new(max_x, max_y);
        }

        println!(
            "After rebuild: bounds min {:?} max {:?} cols {} rows {}",
            self.bounds.min, self.bounds.max, self.cols, self.rows
        );
        let degenerate = self.segments.iter().filter(|s| s.start == s.end).count();
        println!("Degenerate segments: {}", degenerate);

        self.recalc_grid();
        println!(
            "Grid recalculated: min {:?} max {:?} cols {} rows {}",
            self.bounds.min, self.bounds.max, self.cols, self.rows
        );

        self.cells.iter_mut().for_each(|cell| cell.clear());

        for (seg_id, seg) in self.segments.iter().enumerate() {
            if seg.start == seg.end {
                continue;
            }
            let min_x = seg.start.x.min(seg.end.x);
            let max_x = seg.start.x.max(seg.end.x);
            let min_y = seg.start.y.min(seg.end.y);
            let max_y = seg.start.y.max(seg.end.y);

            let start_cell = self.point_to_cell(Vec2::new(min_x, min_y));
            let end_cell = self.point_to_cell(Vec2::new(max_x, max_y));
            if start_cell.is_none() || end_cell.is_none() {
                continue;
            }
            let (start_col, start_row) = start_cell.unwrap();
            let (end_col, end_row) = end_cell.unwrap();

            for col in start_col.min(end_col)..=start_col.max(end_col) {
                for row in start_row.min(end_row)..=start_row.max(end_row) {
                    let idx = self.cell_index(col, row);
                    self.cells[idx].push(seg_id);
                }
            }
        }
    }

    /// Finds the nearest segment endpoint to `pos` within `snap_radius`.
    pub fn query_nearest_point(
        &self,
        pos: Vec2,
        snap_radius: f32,
        initial_seg_id: Option<usize>,
        blocked_range: usize,
    ) -> Option<(Vec2, usize)> {
        let cell_opt = self.point_to_cell(pos)?;
        let (col, row) = cell_opt;

        let mut closest_point = None;
        let mut closest_dist = snap_radius;
        let mut closest_id = None;

        for cx in col.saturating_sub(1)..=(col + 1).min(self.cols - 1) {
            for cy in row.saturating_sub(1)..=(row + 1).min(self.rows - 1) {
                let idx = self.cell_index(cx, cy);
                for &seg_id in &self.cells[idx] {
                    if let Some(initial_id) = initial_seg_id {
                        if seg_id >= initial_id.saturating_sub(blocked_range)
                            && seg_id <= initial_id + blocked_range
                        {
                            continue;
                        }
                    }
                    let segment = &self.segments[seg_id];
                    if segment.start == segment.end {
                        continue;
                    }
                    let pt = closest_point_on_segment(pos, segment.start, segment.end);
                    let dist = pt.distance(pos);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_point = Some(pt);
                        closest_id = Some(seg_id);
                    }
                }
            }
        }
        match (closest_point, closest_id) {
            (Some(pt), Some(id)) => Some((pt, id)),
            _ => None,
        }
    }
    pub fn query_nearest_segment(
        &self,
        pos: Vec2,
        snap_radius: f32,
    ) -> Option<(PathSegment, Vec2)> {
        // Similar to query_nearest_point, but return the whole segment and the closest point on it
        let cell_opt = self.point_to_cell(pos)?;
        let (col, row) = cell_opt;
        let mut closest_seg = None;
        let mut closest_point = None;
        let mut closest_dist = snap_radius;

        for cx in col.saturating_sub(1)..=(col + 1).min(self.cols - 1) {
            for cy in row.saturating_sub(1)..=(row + 1).min(self.rows - 1) {
                let idx = self.cell_index(cx, cy);
                for &seg_id in &self.cells[idx] {
                    let segment = &self.segments[seg_id];
                    if segment.start == segment.end { continue; }
                    let pt = closest_point_on_segment(pos, segment.start, segment.end);
                    let dist = pt.distance(pos);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_seg = Some(*segment);
                        closest_point = Some(pt);
                    }
                }
            }
        }
        match (closest_seg, closest_point) {
            (Some(seg), Some(pt)) => Some((seg, pt)),
            _ => None,
        }
    }
}
