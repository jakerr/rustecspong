#![allow(unused_variables, unused_imports)]

#[macro_use]
extern crate ecs;
use ecs::*;
use ecs::system::{EntityProcess, EntitySystem};

extern crate shader_version;
extern crate input;
extern crate event;
extern crate graphics;
extern crate sdl2_window;
extern crate window;
extern crate opengl_graphics;
extern crate quack;

use window::WindowSettings;

use std::collections::HashMap;
use std::default::Default;
use std::rand;
use std::rand::Rng;
use event::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs};
use quack::Set;
use std::cell::RefCell;

use opengl_graphics::{
    Gl,
};
use sdl2_window::Sdl2Window;

static WINDOW_W: f64 = 800.0;
static WINDOW_H: f64 = 600.0;
static WINDOW_PADDING: i32 = 20;

pub struct ShimmerSystem;

impl System for ShimmerSystem {
    type Components = Components;
}

pub struct DrawSystem {
    gl: Option<RefCell<Gl>>,
    args: Option<RefCell<RenderArgs>>
}

impl EntityProcess for ShimmerSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components>) {
        for e in entities {
            let color = &mut data.colors[e];
            let ref mut rng = rand::thread_rng();
            color.r = rng.gen_range(0.5, 1.0);
            color.g = rng.gen_range(0.5, 1.0);
            color.b = rng.gen_range(0.5, 1.0);
        }
    }
}

impl System for DrawSystem {
    type Components = Components;
    fn is_active(&self) -> bool { false }
}

impl EntityProcess for DrawSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components>) {
        use graphics::*;
        use ShapeVarient::*;
        for e in entities {
            let position = &data.positions[e];
            let shape = &data.shapes[e];
            let color = &data.colors[e];
            let pad = WINDOW_PADDING;
            if let Some(ref gl_cell) = self.gl {
            if let Some(ref arg_cell) = self.args {
                let mut gl = gl_cell.borrow_mut();
                let mut args = arg_cell.borrow();
                gl.draw([pad, pad, args.width as i32 - 2 * pad, args.height as i32 - 2 * pad], |c, gl| { // viewport
                    graphics::clear([0.2, 0.2, 0.2, 1.0], gl);
                    let r = Rectangle::new([0.0, 0.0, 0.0, 1.0]);
                    r.draw([0.0, 0.0, WINDOW_W, WINDOW_H], &c, gl);
                    match shape.varient {
                        Circle(rad) => {
                            Ellipse::new([color.r, color.g, color.b, 1.0])
                                .draw([
                                      position.x,
                                      position.y,
                                      rad, rad
                                ], &c, gl);
                        },
                        Point | _ => {
                            Rectangle::new([color.r, color.g, color.b, 1.0])
                                .draw([
                                      position.x,
                                      position.y,
                                      1.0f64, 1.0f64
                                ], &c, gl);
                        },
                    };
                });
            } //gl cell
            } // arg cell
        }
    }
}

//fn draw_system(event: &Event,
//              gl: &mut Gl,
//              positions: &mut Components<Position>,
//              shapes: &mut Components<Shape>,
//              colors: &mut Components<Color>) {
//    use graphics::*;
//    if let Some(args) = event.update_args() {
//        let w = args.width as f64;
//        let h = args.height as f64;
//
//        let pad = WINDOW_PADDING;
//        gl.viewport(pad, pad, w as i32 - 2 * pad, h as i32 - 2 * pad);
//
//        gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
//            graphics::clear([0.2, 0.2, 0.0, 1.0], gl);
//            let r = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
//            r.draw([0.0, 0.0, WINDOW_W, WINDOW_H], &c, gl);
//        });
//
////        c.rgb(0.7, 0.7, 0.7).rect(0.0, 0.0, WINDOW_W, WINDOW_H).border_radius(1.0).draw(gl);
////
////        for (eid, pos) in positions.iter_mut() {
////          let mut shape = Point;
////          let mut border = None;
////          if shapes.contains_key(eid) {
////            shape = shapes.get_mut(eid).varient;
////            border = shapes.get_mut(eid).border;
////          }
////          let (r, g, b) = if colors.contains_key(eid) {
////              (colors.get_mut(eid).r, colors.get_mut(eid).g, colors.get_mut(eid).b)
////          } else {
////              (1.0, 1.0, 1.0)
////          };
////          let drawing = c.rgb(r, g, b);
////          match (shape, border) {
////              (Point, None)    => drawing.rect(1.0, 1.0, w, h).draw(gl),
////              (Point, Some(b)) => drawing.rect(1.0, 1.0, w, h).border_radius(b).draw(gl),
////
////              (Circle(rad), None)    => drawing.ellipse(pos.x, pos.y, rad, rad).draw(gl),
////              (Circle(rad), Some(b)) => drawing.ellipse(pos.x, pos.y, rad, rad).border_radius(b).draw(gl),
////
////              (Square(w,h), None)    => drawing.rect(pos.x, pos.y, w, h).draw(gl),
////              (Square(w,h), Some(b)) => drawing.rect(pos.x, pos.y, w, h).border_radius(b).draw(gl),
////          };
////        }
//    }
//}
//
//fn move_system(event: &Event,
//               to_delete: &mut Vec<u32>,
//              positions: &mut Components<Position>,
//              shapes: &mut Components<Shape>,
//              clamps: &mut Components<WindowClamp>,
//              velocities: &mut Components<Velocity>) {
//    if let Some(args) = event.update_args {
//        let dt = args.dt;
////
////        for (eid, position) in positions.iter_mut() {
////            if !velocities.contains_key(eid) {
////                continue;
////            }
////
////            // If we have both a position and a velocity, integrate.
////            let velocity = velocities.get_mut(eid);
////            position.x += velocity.x * dt;
////            position.y += velocity.y * dt;
////
////            if !clamps.contains_key(eid) {
////              continue;
////            }
////            let (w, h) = match shapes.find(eid) {
////                Some(&s) =>
////                  match s.varient {
////                    Circle(r) => (r, r),
////                    Square(w,h) => (w, h),
////                    Point => (0.0, 0.0)
////                  },
////                None => (0.0, 0.0)
////            };
////
////            let clamp = clamps.get_mut(eid);
////            let velocity_mult = match clamp.varient {
////              Bounce => -1.0,
////              Stop => 0.0,
////              _ => 1.0
////            };
////            match clamp.varient {
////              Bounce | Stop => {
////                if position.x + w > WINDOW_W {
////                  position.x = WINDOW_W - w;
////                  velocity.x *= velocity_mult;
////                } else if position.x < 0.0 {
////                  position.x = 0.0;
////                  velocity.x *= velocity_mult;
////                }
////                if position.y + h > WINDOW_H {
////                  position.y = WINDOW_H - h;
////                  velocity.y *= velocity_mult;
////                } else if position.y < 0.0 {
////                  position.y = 0.0;
////                  velocity.y *= velocity_mult;
////                }
////              },
////              Remove => {
////                if position.x > WINDOW_W
////                || position.x + w < 0.0 {
////                  to_delete.push(*eid);
////                }
////                if position.y > WINDOW_H
////                || position.y + h < 0.0 {
////                  to_delete.push(*eid);
////                }
////              }
////            }
////        }
//    }
//}
//
//fn control_system(event: &Event,
//              controllers: &mut Components<PlayerController>,
//              velocities: &mut Components<Velocity>) {
//
//    const PADDLE_V: f64 = 800.0;
//
////    for (eid, controller) in controllers.iter_mut() {
////        event.press(|button| {
////            if button == Keyboard(controller.up) {
////                velocities.get_mut(eid).y = -PADDLE_V;
////            } else if button == Keyboard(controller.down) {
////                velocities.get_mut(eid).y = PADDLE_V;
////            }
////        });
////        event.release(|button| {
////            if button == Keyboard(controller.up)
////            || button == Keyboard(controller.down) {
////                velocities.get_mut(eid).y = 0.0;
////            }
////        });
////    }
//}

#[derive(Clone, PartialEq, Debug)]
pub enum ClampVarient {
    Bounce,
    Stop,
    Remove // Acts when item leaves window.
}

#[derive(Clone, PartialEq, Debug)]
pub struct WindowClamp {
    varient: ClampVarient
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlayerController {
    up: input::Button,
    down: input::Button
}

#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    x: f64,
    y: f64
}

#[derive(Clone, PartialEq, Debug)]
pub struct Shimmer;

#[derive(Clone, PartialEq, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShapeVarient {
    Point,
    Circle(f64), // radius
    Square(f64, f64), // width, height
}

impl Default for ShapeVarient {
    fn default() -> ShapeVarient { ShapeVarient::Point }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Shape {
    varient: ShapeVarient,
    border: Option<f64>
}

#[derive(Clone, PartialEq, Debug)]
pub struct Velocity {
    x: f64,
    y: f64
}

components! {
    Components {
        #[hot] positions: Position,
        #[hot] shapes: Shape,
        #[hot] velocities: Velocity,
        #[hot] colors: Color,
        #[hot] shimmers: Shimmer,
        #[hot] player_controllers: PlayerController,
        #[hot] window_clamps: WindowClamp
    }
}

systems! {
    Systems<Components> {
        shimmer: EntitySystem<ShimmerSystem> = EntitySystem::new(
            ShimmerSystem,
            aspect!(<Components> all: [colors])
        ),
        draw: EntitySystem<DrawSystem> = EntitySystem::new(
            DrawSystem{ gl: None, args: None },
            aspect!(<Components> all: [positions, shapes, colors])
        )
    }
}

fn make_ball(world: &mut World<Components, Systems>) {
    const BALL_R: f64 = 20.0;
    let x = (WINDOW_W - BALL_R) / 2.0;
    let y = (WINDOW_H - BALL_R) / 2.0;

    let entity = world.create_entity(Box::new(|entity: BuildData, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x,
                y: y
        });
        data.shimmers.add(&entity, Shimmer);
        data.shapes.add(&entity,
            Shape {
                varient: ShapeVarient::Circle(BALL_R),
                border: None
        });
        data.colors.add(&entity,
            Color{
                r: 1.0,
                g: 0.5,
                b: 0.2
        });
        data.window_clamps.add(&entity,
            WindowClamp {
               varient: ClampVarient::Bounce
        });
    }));
}

//fn make_player(ents: &mut Entities, p1: bool) -> u32 {
//    const FROM_WALL: f64 = 20.0;
//    const PADDLE_W: f64 = 20.0;
//    const PADDLE_H: f64 = 150.0;
//    let x = if p1 {
//        FROM_WALL
//    } else {
//        WINDOW_W - FROM_WALL - PADDLE_W
//    };
//    let y = (WINDOW_H - PADDLE_H) / 2.0;
//    let e = Entity::new()
//        .with_player_controller(
//            PlayerController {
//                up: if p1 { keyboard::W } else { keyboard::I },
//                down: if p1 { keyboard::S } else { keyboard::K }
//            })
//        .with_velocity(Velocity { x: 0.0, y: 0.0 } )
//        .with_position(
//            Position{
//                x: x,
//                y: y
//            })
//        .with_shape(
//            Shape {
//                varient: Square(PADDLE_W, PADDLE_H),
//                border: None
//            })
//        .with_color(
//            Color{
//                r: 0.5,
//                g: 0.2,
//                b: 0.8
//            })
//        .with_window_clamp(
//            WindowClamp {
//               varient: Stop
//            });
//    ents.add(e)
//}

fn main() {
    let opengl = shader_version::OpenGL::_3_2;
    let window = Sdl2Window::new(
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

    let mut world = World::<Components, Systems>::new();
    world.systems.draw.gl = Some(RefCell::new(gl));
    make_ball(&mut world);

    for e in event::events(window) {
        use event::{ ReleaseEvent, UpdateEvent, PressEvent, RenderEvent };

        if let Some(args) = e.update_args() {
            world.update();
        }
        if let Some(args) = e.render_args() {
            world.systems.draw.args = Some(RefCell::new(args));
            process!(world, draw);
            //(args.dt as f32);
//            use graphics::*;
//            gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
//                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
//                let r = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
//                let off = Color([0.0, 0.0, 0.0, 1.0]);
//                let on = Color([1.0, 1.0, 1.0, 1.0]);
//
//                let w = args.width as f64 / 64.0;
//                let h = args.height as f64 / 32.0;
//
//                for (y,row) in vm.screen_rows().enumerate() {
//                    for (x,byte) in row.iter().enumerate() {
//                        let x = x as f64 * w;
//                        let y = y as f64 * h;
//                        r.set(match *byte { 0 => off, _ => on })
//                        .draw([x, y, w, h], &c, gl);
//                    }
//                }
//            });
        }
    }
}
