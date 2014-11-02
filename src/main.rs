#![feature(phase)]
#![feature(if_let)]

#[phase(plugin)]
extern crate rustecs_macros;

extern crate rustecs;
extern crate serialize;
extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;

extern crate nphysics;
extern crate "nalgebra" as na;
extern crate ncollide;

use na::{Vec2, Translation};
use ncollide::geom::{Ball, Plane, Cuboid};
use nphysics::world::World;
use nphysics::object::{RigidBody, RigidBodyHandle};

use std::collections::HashMap;
use std::default::Default;
use std::rand;
use std::rand::Rng;
use std::num::abs;

use opengl_graphics::{
    Gl,
};
use sdl2_window::Sdl2Window;
use piston::{
    EventIterator,
    EventSettings,
    WindowSettings,
};
use graphics::{
    AddBorder,
    AddRectangle,
    AddEllipse,
    AddColor,
    Context,
    Draw,
};
use graphics::vecmath;
use graphics::can::{
    CanTransform,
    CanViewTransform
};
use piston::event::{
    PressEvent,
    ReleaseEvent,
    Event,
    Render,
    Update,
};
use piston::input::{
    keyboard,
    Keyboard,
};
use rustecs::{
    Components,
    EntityContainer,
};

const WINDOW_W: f64 = 800.0;
const WINDOW_H: f64 = 600.0;
const WINDOW_PADDING: i32 = 20;

static PIXELS_PER_METER: i32 = 150;

fn draw_system(event: &Event,
              gl: &mut Gl,
              positions: &mut Components<Position>,
              shapes: &mut Components<Shape>,
              colors: &mut Components<Color>) {
    if let &Render(args) = event {
        let w = args.width as f64;
        let h = args.height as f64;

        let pad = WINDOW_PADDING;
        gl.viewport(pad, pad, w as i32 - 2 * pad, h as i32 - 2 * pad);

        let c = Context::abs(w, h);
        // Clear background.
        c.rgb(0.2, 0.2, 0.2).draw(gl);
        c.rgb(0.7, 0.7, 0.7).rect(0.0, 0.0, WINDOW_W, WINDOW_H).border_radius(1.0).transform(vecmath::scale(0.5, 0.5)).draw(gl);

        let c = Context::abs(w, h);
        for (eid, pos) in positions.iter_mut() {
          let mut shape = Point;
          let mut border = None;
          if shapes.contains_key(eid) {
            shape = shapes.get_mut(eid).varient;
            border = shapes.get_mut(eid).border;
          }
          let (r, g, b) = if colors.contains_key(eid) {
              (colors.get_mut(eid).r, colors.get_mut(eid).g, colors.get_mut(eid).b)
          } else {
              (1.0, 1.0, 1.0)
          };
          let drawing = c.rgb(r, g, b).view_transform(vecmath::rotate_radians(1.047));
          match (shape, border) {
              (Point, None)    => drawing.rect(1.0, 1.0, w, h).draw(gl),
              (Point, Some(b)) => drawing.rect(1.0, 1.0, w, h).border_radius(b).draw(gl),

              (Circle(rad), None)    => drawing.circle(pos.x, pos.y, rad).draw(gl),
              (Circle(rad), Some(b)) => drawing.circle(pos.x, pos.y, rad).border_radius(b).draw(gl),

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

fn phys_system(event: &Event,
               phys: &mut PhysicalWorld,
               positions: &mut Components<Position>) {
    if let &Update(args) = event {
        let dt = args.dt;
        phys.world.step(dt as f32);
        for eid in phys.bodies.keys() {
          if !positions.contains_key(eid) {
              continue;
          }
          let rb = phys.bodies.find(eid).unwrap().deref().borrow();
          let transform = rb.transform_ref();
          let phys_pos = na::translation(transform);

          let position = positions.get_mut(eid);
          position.x = PIXELS_PER_METER as f64 * phys_pos.x as f64;
          position.y = PIXELS_PER_METER as f64 * phys_pos.y as f64;
        }
    }
}

fn move_system(event: &Event,
               to_delete: &mut Vec<u32>,
              positions: &mut Components<Position>,
              shapes: &mut Components<Shape>,
              clamps: &mut Components<WindowClamp>,
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

            if !clamps.contains_key(eid) {
              continue;
            }
            let (w, h) = match shapes.find(eid) {
                Some(&s) =>
                  match s.varient {
                    Circle(r) => (r, r),
                    Square(w,h) => (w, h),
                    Point => (0.0, 0.0)
                  },
                None => (0.0, 0.0)
            };

            let clamp = clamps.get_mut(eid);
            let velocity_mult = match clamp.varient {
              Bounce => -1.0,
              Stop => 0.0,
              _ => 1.0
            };
            match clamp.varient {
              Bounce | Stop => {
                if position.x + w > WINDOW_W {
                  position.x = WINDOW_W - w;
                  velocity.x *= velocity_mult;
                } else if position.x < 0.0 {
                  position.x = 0.0;
                  velocity.x *= velocity_mult;
                }
                if position.y + h > WINDOW_H {
                  position.y = WINDOW_H - h;
                  velocity.y *= velocity_mult;
                } else if position.y < 0.0 {
                  position.y = 0.0;
                  velocity.y *= velocity_mult;
                }
              },
              Remove => {
                if position.x > WINDOW_W
                || position.x + w < 0.0 {
                  to_delete.push(*eid);
                }
                if position.y > WINDOW_H
                || position.y + h < 0.0 {
                  to_delete.push(*eid);
                }
              }
            }
        }
    }
}

fn control_system(event: &Event,
              controllers: &mut Components<PlayerController>,
              velocities: &mut Components<Velocity>) {

    const PADDLE_V: f64 = 800.0;

    for (eid, controller) in controllers.iter_mut() {
        event.press(|button| {
            if button == Keyboard(controller.up) {
                velocities.get_mut(eid).y = -PADDLE_V;
            } else if button == Keyboard(controller.down) {
                velocities.get_mut(eid).y = PADDLE_V;
            }
        });
        event.release(|button| {
            if button == Keyboard(controller.up)
            || button == Keyboard(controller.down) {
                velocities.get_mut(eid).y = 0.0;
            }
        });
    }
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub enum ClampVarient {
    Bounce,
    Stop,
    Remove // Acts when item leaves window.
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct WindowClamp {
    varient: ClampVarient
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct PlayerController {
    up: keyboard::Key,
    down: keyboard::Key
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
    varient: ShapeVarient,
    border: Option<f64>
}

#[deriving(Clone, Decodable, Encodable, PartialEq, Show)]
pub struct Velocity {
    x: f64,
    y: f64
}

world! {
    components Position, Shape, Velocity, Color, Shimmer, PlayerController,
        WindowClamp;
}

type PhysicalBodies = HashMap<u32, RigidBodyHandle>;

struct PhysicalWorld {
  bodies: PhysicalBodies,
  world: World
}

impl PhysicalWorld {
  fn new() -> PhysicalWorld {
    let mut p = PhysicalWorld {
      bodies: HashMap::new(),
      world: World::new(),
    };
    p.world.set_gravity(Vec2::new(0.0f32, 9.81));
    p
  }
}

fn make_walls(phys: &mut PhysicalWorld) {
    let half_w = WINDOW_W as f32 / PIXELS_PER_METER as f32 / 2.0;
    let half_h = WINDOW_H as f32 / PIXELS_PER_METER as f32 / 2.0;

    let p = 1.0f32  / PIXELS_PER_METER as f32;

    // Top
    let mut rb = RigidBody::new_static(Cuboid::new(Vec2::new(half_w, p)), 0.3, 0.6);
    rb.append_translation(&Vec2::new(half_w, 0.0));
    phys.world.add_body(rb);

    // Bottom
    let mut rb = RigidBody::new_static(Cuboid::new(Vec2::new(half_w, p)), 0.3, 0.6);
    rb.append_translation(&Vec2::new(half_w, half_h * 2.0));
    phys.world.add_body(rb);

    // Left
    let mut rb = RigidBody::new_static(Cuboid::new(Vec2::new(p, half_h)), 0.3, 0.6);
    rb.append_translation(&Vec2::new(0.0, half_h));
    phys.world.add_body(rb);

    // Right
    let mut rb = RigidBody::new_static(Cuboid::new(Vec2::new(p, half_h)), 0.3, 0.6);
    rb.append_translation(&Vec2::new(half_w * 2.0, half_h));
    phys.world.add_body(rb);
}

fn make_ball(ents: &mut Entities, phys: &mut PhysicalWorld, xoff: f64) {
    const BALL_R: f64 = 20.0;
    let shape = Circle(BALL_R);
    let x = WINDOW_W / 2.0 - BALL_R + xoff;
    let y = WINDOW_H / 2.0 - BALL_R;

    let e = Entity::new()
        .with_shimmer(Shimmer)
        .with_position(
            Position{
                x: x,
                y: y
            })
        .with_shape(
            Shape {
                varient: shape,
                border: None
            })
        .with_color(
            Color{
                r: 1.0,
                g: 0.5,
                b: 0.2
            })
        .with_window_clamp(
            WindowClamp {
               varient: Bounce
            });
    let eid = ents.add(e);


    let br = (BALL_R / PIXELS_PER_METER as f64) as f32;
    let mut rb = RigidBody::new_dynamic(Ball::new(br), 1.0f32, 0.3, 0.6);
    rb.append_translation(&Vec2::new((x / PIXELS_PER_METER as f64) as f32 + br, (y / PIXELS_PER_METER as f64) as f32 + br));

    let handle = phys.world.add_body(rb);
    phys.bodies.insert(eid, handle);
    

}

fn make_circle(ents: &mut Entities) {
    const BALL_R: f64 = WINDOW_H / 2.0;
    let shape = Circle(BALL_R);
    let x = WINDOW_W / 2.0 - BALL_R;
    let y = WINDOW_H / 2.0 - BALL_R;

    let e = Entity::new()
        .with_position(
            Position{
                x: x,
                y: y
            })
        .with_shape(
            Shape {
                varient: shape,
                border: Some(3.0)
            })
        .with_color(
            Color{
                r: 1.0,
                g: 0.5,
                b: 0.2
            });
    ents.add(e);
}

fn make_player(ents: &mut Entities, p1: bool) -> u32 {
    const FROM_WALL: f64 = 20.0;
    const PADDLE_W: f64 = 20.0;
    const PADDLE_H: f64 = 150.0;
    let x = if p1 {
        FROM_WALL
    } else {
        WINDOW_W - FROM_WALL - PADDLE_W
    };
    let y = (WINDOW_H - PADDLE_H) / 2.0;
    let e = Entity::new()
        .with_player_controller(
            PlayerController {
                up: if p1 { keyboard::W } else { keyboard::I },
                down: if p1 { keyboard::S } else { keyboard::K }
            })
        .with_velocity(Velocity { x: 0.0, y: 0.0 } )
        .with_position(
            Position{
                x: x,
                y: y
            })
        .with_shape(
            Shape {
                varient: Square(PADDLE_W, PADDLE_H),
                border: None
            })
        .with_color(
            Color{
                r: 0.5,
                g: 0.2,
                b: 0.8
            })
        .with_window_clamp(
            WindowClamp {
               varient: Stop
            });
    ents.add(e)
}

fn main() {
    let opengl = piston::shader_version::opengl::OpenGL_3_2;
    let mut window = Sdl2Window::new(
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
    let mut phys = PhysicalWorld::new();
    let mut ents = Entities::new();

    make_player(&mut ents, true);
    make_player(&mut ents, false);
    make_circle(&mut ents);
    make_ball(&mut ents, &mut phys, 0.0);
    make_ball(&mut ents, &mut phys, 80.0);
    make_ball(&mut ents, &mut phys, 160.0);
    make_ball(&mut ents, &mut phys, 240.0);
    make_ball(&mut ents, &mut phys, 0.0);
    make_ball(&mut ents, &mut phys, 85.0);
    make_ball(&mut ents, &mut phys, 165.0);
    make_ball(&mut ents, &mut phys, 245.0);

    make_walls(&mut phys);

    let event_settings = EventSettings {
        updates_per_second: 120,
        max_frames_per_second: 60,
    };

    let mut to_delete: Vec<u32> = vec!();
    for e in EventIterator::new(&mut window, &event_settings) {
        control_system(&e, &mut ents.player_controllers, &mut ents.velocities);
        move_system(&e,
                    &mut to_delete,
                    &mut ents.positions,
                    &mut ents.shapes,
                    &mut ents.window_clamps,
                    &mut ents.velocities);
        phys_system(&e, &mut phys, &mut ents.positions);
        shimmer_system(&mut ents.shimmers, &mut ents.colors);
        draw_system(&e, &mut gl, &mut ents.positions, &mut ents.shapes, &mut ents.colors);
        for v in to_delete.iter() {
          ents.remove(*v);
        }
        to_delete.clear();
    }
}
