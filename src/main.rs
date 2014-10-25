#![feature(phase)]
#![feature(if_let)]

#[phase(plugin)]
extern crate rustecs_macros;

extern crate rustecs;
extern crate serialize;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;
use std::default::Default;
use std::rand;
use std::rand::Rng;
use std::num::abs;

use opengl_graphics::{
    Gl,
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
    PressEvent,
    ReleaseEvent,
    Event,
    Render,
    Update,
    Input,
};
use piston::input::{
    keyboard,
    Keyboard,
};
use rustecs::{
    Entities,
    Components,
};

static WINDOW_W: f64 = 800.0;
static WINDOW_H: f64 = 600.0;

fn draw_system(event: &Event,
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
          let drawing = c.rgb(r, g, b);
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

fn shimmer_system(shimmers: &mut Components<Shimmer>,
                 colors: &mut Components<Color>) {
    for (eid, _) in shimmers.iter_mut() {
        if !colors.contains_key(eid) {
            continue;
        }
        let color = colors.get_mut(eid);
        let ref mut rng = rand::task_rng();
        color.r = rng.gen_range(0.5, 1.0);
        color.g = rng.gen_range(0.5, 1.0);
        color.b = rng.gen_range(0.5, 1.0);
    }
}

fn move_system(event: &Event,
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
            if position.x > WINDOW_W || position.x < 0.0 {
                velocity.x *= -1.0;
            }
            if position.y > WINDOW_H || position.y < 0.0 {
                velocity.y *= -1.0;
            }
        }
    }
}

fn control_system(event: &Event,
              player_eids: &[u32, ..2],
              velocities: &mut Components<Velocity>) {

    const PADDLE_V: f64 = 800.0;
    let p1 = &player_eids[0];
    let p2 = &player_eids[1];
    event.press(|button| {
        if button == Keyboard(keyboard::W) {
            velocities.get_mut(p1).y = -PADDLE_V;
        } else if button == Keyboard(keyboard::S) {
            velocities.get_mut(p1).y = PADDLE_V;
        } else if button == Keyboard(keyboard::I) {
            velocities.get_mut(p2).y = -PADDLE_V;
        } else if button == Keyboard(keyboard::K) {
            velocities.get_mut(p2).y = PADDLE_V;
        }
    });
    event.release(|button| {
        if button == Keyboard(keyboard::W)
        || button == Keyboard(keyboard::S) {
            velocities.get_mut(p1).y = 0.0;
        } else if button == Keyboard(keyboard::I)
               || button == Keyboard(keyboard::K) {
            velocities.get_mut(p2).y = 0.0;
        }
    });
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Position {
    x: f64,
    y: f64
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Shimmer;

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
    components Position, Shape, Velocity, Color, Shimmer;
}

fn make_ball() -> Entity {
    const BALL_R: f64 = 20.0;
    let shape = Circle(BALL_R);
    let x = (WINDOW_W - BALL_R) / 2.0;
    let y = (WINDOW_H - BALL_R) / 2.0;

    fn random_vel() -> f64 {
        let ref mut rng = rand::task_rng();
        rng.gen_range(80.0, 100.0) * if rng.gen() { 1.0 } else { -1.0 }
    }

    Entity::new()
        .with_shimmer(Shimmer)
        .with_position(
            Position{
                x: x,
                y: y
            })
        .with_velocity(
            Velocity {
                x: random_vel(),
                y: random_vel()
            })
        .with_shape(
            Shape {
                shape: shape,
                border: None
            })
        .with_color(
            Color{
                r: 1.0,
                g: 0.5,
                b: 0.2
            })
}

fn make_player(p1: bool) -> Entity {
    const FROM_WALL: f64 = 20.0;
    const PADDLE_W: f64 = 20.0;
    const PADDLE_H: f64 = 150.0;
    let x = if p1 {
        FROM_WALL
    } else {
        WINDOW_W - FROM_WALL - PADDLE_W
    };
    let y = (WINDOW_H - PADDLE_H) / 2.0;
    Entity::new()
        .with_velocity(Velocity { x: 0.0, y: 0.0 } )
        .with_position(
            Position{
                x: x,
                y: y
            })
        .with_shape(
            Shape {
                shape: Square(PADDLE_W, PADDLE_H),
                border: None
            })
        .with_color(
            Color{
                r: 0.5,
                g: 0.2,
                b: 0.8
            })
}

fn main() {
    let opengl = piston::shader_version::opengl::OpenGL_3_2;
    let mut window = WindowSDL2::new(
        opengl,
        WindowSettings {
            title: "Pong".to_string(),
            size: [WINDOW_W as u32, WINDOW_H as u32],
            fullscreen: false,
            exit_on_esc: true,
            samples: 4,
        }
    );

    let mut gl = Gl::new(opengl);
    let mut world = World::new();

    let players = [
        world.add(make_player(true)),
        world.add(make_player(false))
    ];
    world.add(make_ball());

    let event_settings = EventSettings {
        updates_per_second: 120,
        max_frames_per_second: 60,
    };

    for e in EventIterator::new(&mut window, &event_settings) {
        control_system(&e, &players, &mut world.velocities);
        move_system(&e, &mut world.positions, &mut world.velocities);
        shimmer_system(&mut world.shimmers, &mut world.colors);
        draw_system(&e, &mut gl, &mut world.positions, &mut world.shapes, &mut world.colors);
    }
}
