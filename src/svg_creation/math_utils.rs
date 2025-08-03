use bevy::prelude::*;

pub fn closest_point_on_segment(p: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let ab = b - a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq == 0.0 { return a; }
    let t = (p - a).dot(ab) / ab_len_sq;
    if t < 0.0 { a }
    else if t > 1.0 { b }
    else { a + ab * t }
}

pub fn lines_intersect(p1: Vec2, p2: Vec2, q1: Vec2, q2: Vec2) -> bool {
    let det = (p2.x - p1.x) * (q2.y - q1.y) - (p2.y - p1.y) * (q2.x - q1.x);
    if det.abs() < f32::EPSILON { return false; }
    let u = ((q1.x - p1.x) * (q2.y - q1.y) - (q1.y - p1.y) * (q2.x - q1.x)) / det;
    let v = ((q1.x - p1.x) * (p2.y - p1.y) - (q1.y - p1.y) * (p2.x - p1.x)) / det;
    u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0
}

pub fn segment_intersection_point(p1: Vec2, p2: Vec2, q1: Vec2, q2: Vec2) -> Option<Vec2> {
    let s1 = p2 - p1;
    let s2 = q2 - q1;
    let denom = -s2.x * s1.y + s1.x * s2.y;
    if denom.abs() < f32::EPSILON {
        return None;
    }
    let s = (-s1.y * (p1.x - q1.x) + s1.x * (p1.y - q1.y)) / denom;
    let t = ( s2.x * (p1.y - q1.y) - s2.y * (p1.x - q1.x)) / denom;
    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        Some(Vec2::new(p1.x + (t * s1.x), p1.y + (t * s1.y)))
    } else {
        None
    }
}

pub fn paths_intersect(path1: &[Vec2], path2: &[Vec2]) -> bool {
    for seg1 in path1.windows(2) {
        for seg2 in path2.windows(2) {
            if lines_intersect(seg1[0], seg1[1], seg2[0], seg2[1]) {
                return true;
            }
        }
    }
    false
}

pub fn bounding_box(points: &[Vec2]) -> Option<(Vec2, Vec2)> {
    if points.is_empty() { return None; }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for p in points.iter().filter(|p| p.is_finite()) {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    Some((Vec2::new(min_x, min_y), Vec2::new(max_x, max_y)))
}

pub fn smooth_lines(path: &[Vec2], window: usize) -> Vec<Vec2> {
    if path.len() < 2 || window < 2 { return path.to_vec(); }
    let mut smoothed = Vec::with_capacity(path.len());
    for i in 0..path.len() {
        let start = i.saturating_sub(window / 2);
        let end = (i + window / 2 + 1).min(path.len());
        let slice = &path[start..end];
        let avg = slice.iter().copied().fold(Vec2::ZERO, |a, b| a + b) / (slice.len() as f32);
        smoothed.push(avg);
    }
    smoothed
}

pub fn ramer_douglas_peucker(path: &[Vec2], epsilon: f32) -> Vec<Vec2> {
    if path.len() < 2 {
        return path.to_vec();
    }

    let (start, end) = (path[0], path[path.len() - 1]);
    let mut max_dist = 0.0;
    let mut max_index = 0;

    for i in 1..path.len() - 1 {
        let point = path[i];
        let dist = perpendicular_distance(point, start, end);
        if dist > max_dist {
            max_dist = dist;
            max_index = i;
        }
    }

    if max_dist > epsilon {
        let mut left = ramer_douglas_peucker(&path[..=max_index], epsilon);
        let right = ramer_douglas_peucker(&path[max_index..], epsilon);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![start, end]
    }
}

fn perpendicular_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let ab = line_end - line_start;
    if ab.length_squared() == 0.0 {
        (point - line_start).length()
    } else {
        let t = ((point - line_start).dot(ab)) / ab.length_squared();
        let proj = line_start + ab * t;
        (point - proj).length()
    }
}
