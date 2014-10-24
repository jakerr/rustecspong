#![feature(phase)]
#![feature(if_let)]

#[phase(plugin)]
extern crate rustecs_macros;

extern crate rustecs;
extern crate serialize;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use std::rand;
use std::rand::Rng;
use std::num::abs;

use opengl_graphics::{
    Gl,
    Texture,
};
use sdl2_game_window::WindowSDL2;
use piston::{
    EventIterator,
    EventSettings,
    WindowSettings,
};
use piston::graphics::{
    AddBorder,
    AddRectangle,
    AddEllipse,
    AddColor,
    Context,
    Draw,
};
use piston::event::{
    UpdateEvent,
    Event,
    Render,
    Update,
    Input,
};
use piston::input::{
    Focus,
};

use rustecs::{
    Entities,
    Components,
};

fn drawSystem(event: &Event,
              gl: &mut Gl,
              positions: &mut Components<Position>,
              shapes: &mut Components<Shape>,
              colors: &mut Components<Color>) {
    if let &Render(args) = event {
        let w = args.width as f64;
        let h = args.height as f64;
        gl.viewport(0, 0, w as i32, h as i32);
        let c = Context::abs(w, h);
        // Clear background.
        c.rgb(0.0, 0.0, 0.0).draw(gl);

        for (eid, pos) in positions.iter_mut() {
          let mut shape = Point;
          let mut border = None;
          if shapes.contains_key(eid) {
            shape = shapes.get_mut(eid).shape;
            border = shapes.get_mut(eid).border;
          }
          let (r, g, b) = if colors.contains_key(eid) {
              (colors.get_mut(eid).r, colors.get_mut(eid).g, colors.get_mut(eid).b)
          } else {
              (1.0, 1.0, 1.0)
          };
          let mut drawing = c.rgb(r, g, b);
          match (shape, border) {
              (Point, None)    => drawing.rect(1.0, 1.0, w, h).draw(gl),
              (Point, Some(b)) => drawing.rect(1.0, 1.0, w, h).border_radius(b).draw(gl),

              (Circle(rad), None)    => drawing.ellipse(pos.x, pos.y, rad, rad).draw(gl),
              (Circle(rad), Some(b)) => drawing.ellipse(pos.x, pos.y, rad, rad).border_radius(b).draw(gl),

              (Square(w,h), None)    => drawing.rect(pos.x, pos.y, w, h).draw(gl),
              (Square(w,h), Some(b)) => drawing.rect(pos.x, pos.y, w, h).border_radius(b).draw(gl),
          };
        }
    }
}

fn shimmerSystem(event: &Event,
                 colors: &mut Components<Color>) {
    for (_, color) in colors.iter_mut() {
        let ref mut rng = rand::task_rng();
        color.r = rng.gen_range(0.5, 1.0);
        color.g = rng.gen_range(0.5, 1.0);
        color.b = rng.gen_range(0.5, 1.0);
    }
}

fn moveSystem(event: &Event,
              positions: &mut Components<Position>,
              velocities: &mut Components<Velocity>) {
    if let &Update(args) = event {
        let dt = args.dt;
        for (eid, position) in positions.iter_mut() {
            if !velocities.contains_key(eid) {
                continue;
            }

            // If we have both a position and a velocity, integrate.
            let velocity = velocities.get_mut(eid);
            position.x += velocity.x * dt;
            position.y += velocity.y * dt;
            if position.x > 800.0 || position.x < 0.0 {
                velocity.x *= -1.0;
            }
            if position.y > 600.0 || position.y < 0.0 {
                velocity.y *= -1.0;
            }
        }
    }
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Position {
    x: f64,
    y: f64
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub enum ShapeVarient {
    Point,
    Circle(f64), // radius
    Square(f64, f64), // width, height
}

impl Default for ShapeVarient {
    fn default() -> ShapeVarient { Point }
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Shape {
    shape: ShapeVarient,
    border: Option<f64>
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Velocity {
    x: f64,
    y: f64
}

world! {
    World,
    components Position, Shape, Velocity, Color;
}

fn main() {
    let opengl = piston::shader_version::opengl::OpenGL_3_2;
    let mut window = WindowSDL2::new(
        opengl,
        WindowSettings {
            title: "Shooter".to_string(),
            size: [800, 600],
            fullscreen: false,
            exit_on_esc: true,
            samples: 4,
        }
    );

    let mut gl = Gl::new(opengl);

    let mut world = World::new();

    let num_things: i32 = 300;

    let ref mut rng = rand::task_rng();
    for _ in range(0, num_things) {
        let r = rng.gen_range(10.0, 40.0);
        let shape = if rng.gen() {
            Square(r, r)
        } else {
            Circle(r)
        };
        let border = Some(rng.gen_range(1.0, 3.0));

        let x = (800.0 - r) / 2.0;
        let y = (600.0 - r) / 2.0;

        let e = Entity::new()
            .with_position(
                Position{
                    x: x,
                    y: y
                })
            .with_velocity(
                Velocity {
                    x: rng.gen_range(-80.0, 80.0),
                    y: rng.gen_range(-80.0, 80.0)
                })
            .with_shape(
                Shape {
                    shape: shape,
                    border: border
                })
            .with_color(
                Color{
                    r: 1.0,
                    g: 0.5,
                    b: 0.2
                });
        world.add(e);
    }
    let event_settings = EventSettings {
        updates_per_second: 120,
        max_frames_per_second: 60,
    };

    for e in EventIterator::new(&mut window, &event_settings) {
        moveSystem(&e, &mut world.positions, &mut world.velocities);
        shimmerSystem(&e, &mut world.colors);
        drawSystem(&e, &mut gl, &mut world.positions, &mut world.shapes, &mut world.colors);
    }
}
