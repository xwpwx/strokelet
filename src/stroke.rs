use crate::Direction;

pub const SAMPLE_COUNT: usize = 32;
pub const MATCH_DISTANCE: f64 = 0.20;
pub const MATCH_MARGIN: f64 = 0.07;
pub const MAX_RULES: usize = 16;
pub const ABSOLUTE_MAX_RULES: usize = 64;

pub fn straight_points(direction: Direction) -> Vec<(f64, f64)> {
    match direction {
        Direction::Up => vec![(0.0, 0.0), (0.0, -80.0)],
        Direction::Down => vec![(0.0, 0.0), (0.0, 80.0)],
        Direction::Left => vec![(0.0, 0.0), (-80.0, 0.0)],
        Direction::Right => vec![(0.0, 0.0), (80.0, 0.0)],
    }
}

/// 重采样、平移到重心、按较长边缩放。不旋转。
pub fn normalize(points: &[(f64, f64)]) -> Option<Vec<(f64, f64)>> {
    if points.len() < 2
        || points
            .iter()
            .any(|point| !point.0.is_finite() || !point.1.is_finite())
    {
        return None;
    }
    let sampled = resample(points, SAMPLE_COUNT)?;
    let count = sampled.len() as f64;
    let origin_x = sampled.iter().map(|point| point.0).sum::<f64>() / count;
    let origin_y = sampled.iter().map(|point| point.1).sum::<f64>() / count;
    let shifted: Vec<(f64, f64)> = sampled
        .into_iter()
        .map(|(x, y)| (x - origin_x, y - origin_y))
        .collect();
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for (x, y) in &shifted {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }
    let span = (max_x - min_x).max(max_y - min_y);
    if span <= f64::EPSILON {
        return None;
    }
    Some(
        shifted
            .into_iter()
            .map(|(x, y)| (x / span, y / span))
            .collect(),
    )
}

pub fn mean_distance(left: &[(f64, f64)], right: &[(f64, f64)]) -> Option<f64> {
    if left.len() != right.len() || left.is_empty() {
        return None;
    }
    let total: f64 = left
        .iter()
        .zip(right)
        .map(|(a, b)| (a.0 - b.0).hypot(a.1 - b.1))
        .sum();
    Some(total / left.len() as f64)
}

pub fn best_match(templates: &[Vec<(f64, f64)>], points: &[(f64, f64)]) -> Option<usize> {
    let candidate = normalize(points)?;
    let mut scores = Vec::new();
    for (index, template) in templates.iter().enumerate() {
        let Some(prepared) = normalize(template) else {
            continue;
        };
        let Some(distance) = mean_distance(&candidate, &prepared) else {
            continue;
        };
        scores.push((index, distance));
    }
    scores.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let (index, best) = scores.first().copied()?;
    if best > MATCH_DISTANCE {
        return None;
    }
    if let Some((_, second)) = scores.get(1)
        && second - best < MATCH_MARGIN
    {
        return None;
    }
    Some(index)
}

pub fn conflicts(existing: &[(f64, f64)], candidate: &[(f64, f64)]) -> bool {
    let Some(left) = normalize(existing) else {
        return false;
    };
    let Some(right) = normalize(candidate) else {
        return false;
    };
    mean_distance(&left, &right).is_some_and(|distance| distance <= MATCH_DISTANCE)
}

fn resample(points: &[(f64, f64)], count: usize) -> Option<Vec<(f64, f64)>> {
    if points.len() < 2 || count < 2 {
        return None;
    }
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    for pair in points.windows(2) {
        let step = (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1);
        let last = *lengths.last()?;
        lengths.push(last + step);
    }
    let total = *lengths.last()?;
    if total <= f64::EPSILON {
        return None;
    }
    let mut sampled = Vec::with_capacity(count);
    for index in 0..count {
        let target = total * index as f64 / (count - 1) as f64;
        let mut segment = lengths.partition_point(|length| *length < target);
        segment = segment.saturating_sub(1);
        if segment >= points.len() - 1 {
            segment = points.len() - 2;
        }
        let span = (lengths[segment + 1] - lengths[segment]).max(f64::EPSILON);
        let ratio = ((target - lengths[segment]) / span).clamp(0.0, 1.0);
        let x = points[segment].0 + (points[segment + 1].0 - points[segment].0) * ratio;
        let y = points[segment].1 + (points[segment + 1].1 - points[segment].1) * ratio;
        sampled.push((x, y));
    }
    Some(sampled)
}
