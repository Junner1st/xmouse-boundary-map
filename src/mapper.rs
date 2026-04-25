use anyhow::{Context, Result};
use tracing::warn;

use crate::geometry::{EdgeMapping, MappingMode, Monitor, Point, PointerState, Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapOutcome {
    Warp(Point),
    Noop,
}

#[derive(Debug)]
pub struct BoundaryMapper {
    monitors: Vec<Monitor>,
    edges: Vec<EdgeMapping>,
}

impl BoundaryMapper {
    pub fn new(monitors: Vec<Monitor>, configured_edges: Vec<EdgeMapping>) -> Self {
        let edges = if configured_edges.is_empty() {
            auto_edges(&monitors)
        } else {
            valid_configured_edges(&monitors, configured_edges)
        };

        Self { monitors, edges }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn edges(&self) -> &[EdgeMapping] {
        &self.edges
    }

    pub fn map_crossing(&self, previous: &PointerState, current: &PointerState) -> MapOutcome {
        let Some(from) = self.monitor_at(previous.position) else {
            return MapOutcome::Noop;
        };

        let edge = match self.monitor_at(current.position) {
            Some(to) if from.name == to.name => return MapOutcome::Noop,
            Some(to) => self.edges.iter().find(|edge| {
                edge.from == from.name && edge.to == to.name && edge.side.matches(from, to)
            }),
            None => crossed_empty_space_edge(&self.edges, from, current.position),
        };

        let Some(edge) = edge else {
            return MapOutcome::Noop;
        };
        let Some(to) = self.monitors.iter().find(|monitor| monitor.name == edge.to) else {
            return MapOutcome::Noop;
        };

        match target_point(edge, from, to, previous.position) {
            Ok(point) => MapOutcome::Warp(point),
            Err(error) => {
                warn!(?error, "failed to map monitor crossing");
                MapOutcome::Noop
            }
        }
    }

    fn monitor_at(&self, point: Point) -> Option<&Monitor> {
        self.monitors.iter().find(|monitor| monitor.contains(point))
    }
}

fn target_point(edge: &EdgeMapping, from: &Monitor, to: &Monitor, source: Point) -> Result<Point> {
    let mapped_y = match edge.mode {
        MappingMode::RelativeResolution => map_relative_resolution(from, to, source.y),
        MappingMode::PhysicalSize => map_physical_size(from, to, source.y)?,
        MappingMode::CustomScale { y_scale } => map_custom_scale(from, to, source.y, y_scale),
    };

    let x = match edge.side {
        Side::Right => to.x,
        Side::Left => to.right(),
    };

    Ok(Point {
        x,
        y: to.clamp_y(mapped_y.round() as i32),
    })
}

fn map_relative_resolution(from: &Monitor, to: &Monitor, y: i32) -> f64 {
    let from_relative = (y - from.y) as f64 / from.height as f64;
    to.y as f64 + from_relative * to.height as f64
}

fn map_physical_size(from: &Monitor, to: &Monitor, y: i32) -> Result<f64> {
    let from_mm = from
        .mm_height
        .filter(|height| *height > 0)
        .with_context(|| format!("{} has no physical height", from.name))?;
    let to_mm = to
        .mm_height
        .filter(|height| *height > 0)
        .with_context(|| format!("{} has no physical height", to.name))?;

    let source_mm = (y - from.y) as f64 / from.height as f64 * from_mm as f64;
    Ok(to.y as f64 + source_mm / to_mm as f64 * to.height as f64)
}

fn map_custom_scale(from: &Monitor, to: &Monitor, y: i32, y_scale: f64) -> f64 {
    to.y as f64 + (y - from.y) as f64 * y_scale
}

fn auto_edges(monitors: &[Monitor]) -> Vec<EdgeMapping> {
    let mut edges = Vec::new();

    for from in monitors {
        for to in monitors {
            if from.name == to.name {
                continue;
            }

            if from.x + from.width == to.x && vertical_overlap(from, to) {
                edges.push(EdgeMapping {
                    from: from.name.clone(),
                    to: to.name.clone(),
                    side: Side::Right,
                    mode: MappingMode::RelativeResolution,
                });
            } else if to.x + to.width == from.x && vertical_overlap(from, to) {
                edges.push(EdgeMapping {
                    from: from.name.clone(),
                    to: to.name.clone(),
                    side: Side::Left,
                    mode: MappingMode::RelativeResolution,
                });
            }
        }
    }

    edges
}

fn valid_configured_edges(monitors: &[Monitor], edges: Vec<EdgeMapping>) -> Vec<EdgeMapping> {
    edges
        .into_iter()
        .filter(|edge| {
            let from = monitors.iter().find(|monitor| monitor.name == edge.from);
            let to = monitors.iter().find(|monitor| monitor.name == edge.to);

            match (from, to) {
                (Some(from), Some(to)) if edge.side.matches(from, to) => true,
                (Some(_), Some(_)) => {
                    warn!(
                        "ignoring configured edge with invalid side: {:?} {} -> {}",
                        edge.side, edge.from, edge.to
                    );
                    false
                }
                _ => {
                    warn!(
                        "ignoring configured edge with unknown monitor: {:?} {} -> {}",
                        edge.side, edge.from, edge.to
                    );
                    false
                }
            }
        })
        .collect()
}

fn vertical_overlap(a: &Monitor, b: &Monitor) -> bool {
    a.y <= b.bottom() && b.y <= a.bottom()
}

fn crossed_empty_space_edge<'a>(
    edges: &'a [EdgeMapping],
    from: &Monitor,
    current: Point,
) -> Option<&'a EdgeMapping> {
    let side = if current.x >= from.x + from.width {
        Side::Right
    } else if current.x < from.x {
        Side::Left
    } else {
        return None;
    };

    edges.iter().find(|edge| edge.from == from.name && edge.side == side)
}

impl Side {
    fn matches(self, from: &Monitor, to: &Monitor) -> bool {
        match self {
            Side::Right => from.x + from.width <= to.x,
            Side::Left => to.x + to.width <= from.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(name: &str, x: i32, y: i32, width: i32, height: i32) -> Monitor {
        Monitor {
            name: name.to_string(),
            x,
            y,
            width,
            height,
            mm_width: None,
            mm_height: None,
        }
    }

    #[test]
    fn maps_left_4k_to_right_1080p_by_relative_height() {
        let mapper = BoundaryMapper::new(
            vec![
                monitor("DP-1", 0, 0, 3840, 2160),
                monitor("HDMI-1", 3840, 0, 1920, 1080),
            ],
            Vec::new(),
        );

        let outcome = mapper.map_crossing(
            &PointerState {
                position: Point { x: 3839, y: 1620 },
                buttons_down: false,
            },
            &PointerState {
                position: Point { x: 3840, y: 1620 },
                buttons_down: false,
            },
        );

        assert_eq!(outcome, MapOutcome::Warp(Point { x: 3840, y: 810 }));
    }

    #[test]
    fn maps_right_1080p_to_left_4k_by_relative_height() {
        let mapper = BoundaryMapper::new(
            vec![
                monitor("DP-1", 0, 0, 3840, 2160),
                monitor("HDMI-1", 3840, 0, 1920, 1080),
            ],
            Vec::new(),
        );

        let outcome = mapper.map_crossing(
            &PointerState {
                position: Point { x: 3840, y: 810 },
                buttons_down: false,
            },
            &PointerState {
                position: Point { x: 3839, y: 810 },
                buttons_down: false,
            },
        );

        assert_eq!(outcome, MapOutcome::Warp(Point { x: 3839, y: 1620 }));
    }

    #[test]
    fn ignores_moves_that_stay_on_one_monitor() {
        let mapper = BoundaryMapper::new(
            vec![
                monitor("DP-1", 0, 0, 3840, 2160),
                monitor("HDMI-1", 3840, 0, 1920, 1080),
            ],
            Vec::new(),
        );

        let outcome = mapper.map_crossing(
            &PointerState {
                position: Point { x: 100, y: 100 },
                buttons_down: false,
            },
            &PointerState {
                position: Point { x: 110, y: 110 },
                buttons_down: false,
            },
        );

        assert_eq!(outcome, MapOutcome::Noop);
    }
}
