use std::default::Default;
use input::keyboard;

#[derive(Clone, PartialEq, Debug)]
pub enum ClampVariant {
    Bounce,
    Stop,
    Remove // Acts when item leaves window.
}

#[derive(Clone, PartialEq, Debug)]
pub struct WindowClamp {
    pub variant: ClampVariant
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlayerController {
    pub up: keyboard::Key,
    pub down: keyboard::Key
}

#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64
}

#[derive(Clone, PartialEq, Debug)]
pub struct Shimmer;

#[derive(Clone, PartialEq, Debug)]
pub struct Fade(pub f32); // Speed of fade

#[derive(Clone, PartialEq, Debug)]
pub enum ShapeVariant {
    Point,
    Circle(f64), // radius
    Square(f64, f64), // width, height
    Line([f64; 4]), // x1, y1, x2, y2
}

impl Default for ShapeVariant {
    fn default() -> ShapeVariant { ShapeVariant::Point }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Shape {
    pub variant: ShapeVariant,
    pub border: Option<f64>
}

#[derive(Clone, PartialEq, Debug)]
pub struct Velocity {
    pub x: f64,
    pub y: f64
}

pub type Color = ::graphics::Color;

#[derive(Clone, PartialEq, Debug)]
pub struct HitCount {
    pub recent: bool,
    pub count: u32
}
