//! Static navigation and swept collision share inflated obstacle geometry.
use bevy::prelude::*;
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub half_size: Vec2,
}

#[derive(Resource, Clone, Debug)]
pub struct NavigationMap {
    half_size: f32,
    cell_size: f32,
    obstacles: Vec<Obstacle>,
    temporary: std::collections::BTreeMap<Entity, (Vec2, f32, f32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathError {
    InvalidPosition,
    Unreachable,
}

impl NavigationMap {
    pub fn new(half_size: f32, cell_size: f32, obstacles: Vec<Obstacle>) -> Self {
        assert!(half_size.is_finite() && half_size > 0.0);
        assert!(cell_size.is_finite() && cell_size > 0.0 && cell_size <= half_size);
        assert!(obstacles.iter().all(|o| o.center.is_finite()
            && o.half_size.is_finite()
            && o.half_size.cmpgt(Vec2::ZERO).all()));
        Self {
            half_size,
            cell_size,
            obstacles,
            temporary: default(),
        }
    }
    pub(super) fn add_ring(&mut self, id: Entity, center: Vec2, radius: f32, thickness: f32) {
        self.temporary.insert(id, (center, radius, thickness));
    }
    pub(super) fn remove_ring(&mut self, id: Entity) {
        self.temporary.remove(&id);
    }
    pub(super) fn clear_temporary(&mut self) {
        self.temporary.clear();
    }
    pub fn half_size(&self) -> f32 {
        self.half_size
    }
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }
    pub(super) fn projectile_sweep(&self, start: Vec2, end: Vec2, radius: f32) -> Vec2 {
        self.sweep_policy(start, end, radius, false)
    }
    pub(super) fn projectile_line_clear(&self, start: Vec2, end: Vec2, radius: f32) -> bool {
        self.line_clear_policy(start, end, radius, false)
    }
    fn walkable_static(&self, point: Vec2, radius: f32) -> bool {
        point.is_finite()
            && point
                .abs()
                .cmplt(Vec2::splat(self.half_size - radius))
                .all()
            && self.obstacles.iter().all(|o| {
                (point - o.center)
                    .abs()
                    .cmpgt(o.half_size + Vec2::splat(radius))
                    .any()
            })
    }
    pub fn walkable(&self, point: Vec2, radius: f32) -> bool {
        self.walkable_static(point, radius)
            && self.temporary.values().all(|(center, ring, thickness)| {
                (point.distance(*center) - ring).abs() > thickness + radius
            })
    }
    /// Resolve a birth point without repairing invalid static configuration.
    /// Temporary terrain may project it only within the blocking ring's local
    /// neighborhood, through clear static geometry. This never moves an actor.
    pub fn resolve_placement(&self, preferred: Vec2, radius: f32) -> Option<Vec2> {
        if !radius.is_finite() || radius <= 0.0 || !self.walkable_static(preferred, radius) {
            return None;
        }
        if self.walkable(preferred, radius) {
            return Some(preferred);
        }
        let limit = self
            .temporary
            .values()
            .filter(|(center, ring, thickness)| {
                (preferred.distance(*center) - ring).abs() <= thickness + radius
            })
            .map(|(_, ring, thickness)| ring + thickness + radius)
            .fold(0.0_f32, f32::max)
            .min(self.half_size());
        (1..=(limit / 0.05).ceil() as usize).find_map(|step| {
            let distance = (step as f32 * 0.05).min(limit);
            (0..64).find_map(|spoke| {
                let point = preferred
                    + Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 64.0) * distance;
                (self.walkable(point, radius)
                    && self.line_clear_policy(preferred, point, radius, false))
                .then_some(point)
            })
        })
    }
    /// Conservative capsule sweep. Dashes use this same collision policy.
    pub fn sweep(&self, start: Vec2, end: Vec2, radius: f32) -> Vec2 {
        self.sweep_policy(start, end, radius, true)
    }
    fn sweep_policy(&self, start: Vec2, end: Vec2, radius: f32, temporary: bool) -> Vec2 {
        if !(if temporary {
            self.walkable(start, radius)
        } else {
            self.walkable_static(start, radius)
        }) || !end.is_finite()
        {
            return start;
        }
        let delta = end - start;
        if delta.length_squared() < 1e-10 {
            return start;
        }
        let bound = self.half_size - radius - 0.001;
        let mut fraction = 1.0_f32;
        for axis in 0..2 {
            if delta[axis] > 0.0 {
                fraction = fraction.min((bound - start[axis]) / delta[axis]);
            } else if delta[axis] < 0.0 {
                fraction = fraction.min((-bound - start[axis]) / delta[axis]);
            }
        }
        for obstacle in &self.obstacles {
            let half = obstacle.half_size + Vec2::splat(radius);
            if let Some(hit) =
                segment_box(start, end, obstacle.center - half, obstacle.center + half)
            {
                fraction = fraction.min((hit - 0.001 / delta.length()).max(0.0));
            }
        }
        // An annulus can be thinner than any fixed sampling step, including a
        // short grazing intersection. Solve its first expanded boundary in f64.
        if temporary {
            for (center, ring, thickness) in self.temporary.values() {
                if let Some(hit) = segment_ring(start, end, *center, *ring, *thickness + radius) {
                    fraction = fraction.min((hit - 0.001 / delta.length()).max(0.0));
                }
            }
        }
        start + delta * fraction.clamp(0.0, 1.0)
    }
    pub fn line_clear(&self, start: Vec2, end: Vec2, radius: f32) -> bool {
        self.line_clear_policy(start, end, radius, true)
    }
    fn line_clear_policy(&self, start: Vec2, end: Vec2, radius: f32, temporary: bool) -> bool {
        (if temporary {
            self.walkable(end, radius)
        } else {
            self.walkable_static(end, radius)
        }) && self
            .sweep_policy(start, end, radius, temporary)
            .distance_squared(end)
            < 1e-6
    }
    /// Smoothed route excluding the start. Blocked clicks project at most 3 cells.
    pub fn path(&self, start: Vec2, requested: Vec2, radius: f32) -> Result<Vec<Vec2>, PathError> {
        self.path_policy(start, requested, radius, true)
    }
    /// Configuration reachability uses the current static map; temporary walls
    /// may interrupt a route during play without making the mode invalid.
    pub fn path_static(
        &self,
        start: Vec2,
        requested: Vec2,
        radius: f32,
    ) -> Result<Vec<Vec2>, PathError> {
        self.path_policy(start, requested, radius, false)
    }
    fn path_policy(
        &self,
        start: Vec2,
        requested: Vec2,
        radius: f32,
        temporary: bool,
    ) -> Result<Vec<Vec2>, PathError> {
        let walkable = |point| {
            if temporary {
                self.walkable(point, radius)
            } else {
                self.walkable_static(point, radius)
            }
        };
        let line_clear = |start, end| self.line_clear_policy(start, end, radius, temporary);
        if !start.is_finite()
            || !requested.is_finite()
            || !walkable(start)
            || requested.abs().cmpgt(Vec2::splat(self.half_size)).any()
        {
            return Err(PathError::InvalidPosition);
        }
        if line_clear(start, requested) {
            return Ok(vec![requested]);
        }
        let width = ((self.half_size * 2.0) / self.cell_size).ceil() as usize;
        let center = |index: usize| {
            Vec2::new((index % width) as f32 + 0.5, (index / width) as f32 + 0.5) * self.cell_size
                - Vec2::splat(self.half_size)
        };
        let nearest = |point: Vec2, accessible_from: Option<Vec2>| {
            (0..width * width)
                .filter(|&i| {
                    walkable(center(i)) && accessible_from.is_none_or(|p| line_clear(p, center(i)))
                })
                .min_by(|&a, &b| {
                    center(a)
                        .distance_squared(point)
                        .total_cmp(&center(b).distance_squared(point))
                })
        };
        let source = nearest(start, Some(start)).ok_or(PathError::Unreachable)?;
        let mut goal = nearest(requested, None).ok_or(PathError::Unreachable)?;
        let mut destination = if walkable(requested) && line_clear(center(goal), requested) {
            requested
        } else {
            center(goal)
        };
        if destination.distance(requested) > self.cell_size * 3.0 {
            return Err(PathError::Unreachable);
        }
        let heuristic = |i: usize| {
            let dx = (i % width).abs_diff(goal % width) as u32;
            let dy = (i / width).abs_diff(goal / width) as u32;
            10 * dx.max(dy) + 4 * dx.min(dy)
        };
        let mut open = BinaryHeap::new();
        let mut costs = vec![u32::MAX; width * width];
        let mut previous = vec![usize::MAX; width * width];
        costs[source] = 0;
        open.push(Reverse((heuristic(source), 0_u32, source)));
        while let Some(Reverse((_, cost, current))) = open.pop() {
            if cost != costs[current] {
                continue;
            }
            if current == goal {
                break;
            }
            let x = (current % width) as isize;
            let y = (current / width) as isize;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= width as isize || ny >= width as isize {
                        continue;
                    }
                    let next = ny as usize * width + nx as usize;
                    if !line_clear(center(current), center(next)) {
                        continue;
                    }
                    let next_cost = cost + if dx != 0 && dy != 0 { 14 } else { 10 };
                    if next_cost < costs[next] {
                        costs[next] = next_cost;
                        previous[next] = current;
                        open.push(Reverse((next_cost + heuristic(next), next_cost, next)));
                    }
                }
            }
        }
        if costs[goal] == u32::MAX {
            if walkable(requested) {
                return Err(PathError::Unreachable);
            }
            // The nearest geometric projection may lie across a sealed wall.
            // An exhausted search has visited the entire reachable component;
            // choose its nearest nearby projection without a second search.
            goal = (0..costs.len())
                .filter(|&i| {
                    costs[i] != u32::MAX && center(i).distance(requested) <= self.cell_size * 3.0
                })
                .min_by(|&a, &b| {
                    center(a)
                        .distance_squared(requested)
                        .total_cmp(&center(b).distance_squared(requested))
                        .then_with(|| costs[a].cmp(&costs[b]))
                })
                .ok_or(PathError::Unreachable)?;
            destination = center(goal);
        }
        let mut route = vec![destination];
        let mut current = goal;
        while current != source {
            route.push(center(current));
            current = previous[current];
        }
        route.push(center(source));
        route.reverse();
        let mut smoothed = Vec::new();
        let mut anchor = start;
        let mut index = 0;
        while index < route.len() {
            let mut next = index;
            while next + 1 < route.len() && line_clear(anchor, route[next + 1]) {
                next += 1;
            }
            smoothed.push(route[next]);
            anchor = route[next];
            index = next + 1;
        }
        Ok(smoothed)
    }
}

fn segment_ring(start: Vec2, end: Vec2, center: Vec2, ring: f32, padding: f32) -> Option<f32> {
    let offset = start.as_dvec2() - center.as_dvec2();
    let direction = end.as_dvec2() - start.as_dvec2();
    let a = direction.length_squared();
    if a == 0.0 {
        return None;
    }
    // The caller has already established that the start is walkable, so it
    // lies either in the inner opening or beyond the outer expanded circle.
    let inside = offset.length_squared() < f64::from(ring).powi(2);
    let boundary = if inside {
        (f64::from(ring) - f64::from(padding)).max(0.0)
    } else {
        f64::from(ring) + f64::from(padding)
    };
    let b = offset.dot(direction);
    let c = offset.length_squared() - boundary * boundary;
    let discriminant = b * b - a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b + if inside { 1.0 } else { -1.0 } * discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&t).then_some(t as f32)
}

fn segment_box(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> Option<f32> {
    let delta = end - start;
    let (mut enter, mut exit) = (0.0_f32, 1.0_f32);
    for axis in 0..2 {
        if delta[axis].abs() < 1e-8 {
            if start[axis] < min[axis] || start[axis] > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - start[axis]) / delta[axis];
            let b = (max[axis] - start[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    Some(enter)
}
