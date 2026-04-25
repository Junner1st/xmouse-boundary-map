use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerState {
    pub position: Point,
    pub buttons_down: bool,
}

impl PointerState {
    pub fn with_position(&self, position: Point) -> Self {
        Self {
            position,
            buttons_down: self.buttons_down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawMotion {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub mm_width: Option<i32>,
    pub mm_height: Option<i32>,
}

impl Monitor {
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub fn right(&self) -> i32 {
        self.x + self.width - 1
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height - 1
    }

    pub fn clamp_y(&self, y: i32) -> i32 {
        y.clamp(self.y, self.bottom())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MappingMode {
    RelativeResolution,
    PhysicalSize,
    CustomScale { y_scale: f64 },
}

impl Default for MappingMode {
    fn default() -> Self {
        Self::RelativeResolution
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMapping {
    pub from: String,
    pub to: String,
    pub side: Side,
    pub mode: MappingMode,
}
