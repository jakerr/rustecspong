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
use event::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};
use quack::Set;
use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;
use input::keyboard;

use opengl_graphics::{
    Gl,
};
use sdl2_window::Sdl2Window;

const WINDOW_W: f64 = 800.0;
const WINDOW_H: f64 = 600.0;
const WINDOW_PADDING: f64 = 20.0;
const VIEW_W: f64 = (WINDOW_W - 2.0 * WINDOW_PADDING);
const VIEW_H: f64 = (WINDOW_H - 2.0 * WINDOW_PADDING);

pub struct Meta {
    render: Cell<RenderArgs>,
    update: Cell<UpdateArgs>
}

pub struct ShimmerSystem;

impl System for ShimmerSystem {
    type Components = Components;
}

pub struct DrawSystem {
    gl: Option<RefCell<Gl>>,
    meta: Option<Rc<Meta>>
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
        let pad = WINDOW_PADDING;
        if let Some(ref gl_cell) = self.gl {
        if let Some(ref meta) = self.meta {
            let mut gl = gl_cell.borrow_mut();
            let render = meta.render.get();
            let view_width = render.width as f64 - 2.0 * pad;
            let view_height = render.height as f64 - 2.0 * pad;
            gl.draw([pad as i32, pad as i32, view_width as i32, view_height as i32], |c, gl| { // viewport
                graphics::clear([0.2, 0.2, 0.2, 1.0], gl);
                let r = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
                r.draw([0.0, 0.0, view_width, view_height], &c, gl);
                let r = Rectangle::new([0.0, 0.0, 0.0, 1.0]);
                r.draw([1.0, 1.0, view_width - 2.0,  view_height - 2.0], &c, gl);
                for e in entities {
                    let position = &data.positions[e];
                    let shape = &data.shapes[e];
                    let color = &data.colors[e];
                    match shape.varient {
                        Circle(rad) => {
                            Ellipse::new([color.r, color.g, color.b, 1.0])
                                .draw([
                                      position.x,
                                      position.y,
                                      2.0*rad, 2.0*rad
                                ], &c, gl);
                        },
                        Square(w, h) => {
                            Rectangle::new([color.r, color.g, color.b, 1.0])
                                .draw([
                                      position.x,
                                      position.y,
                                      w, h
                                ], &c, gl);
                        },
                        Point => {
                            Rectangle::new([color.r, color.g, color.b, 1.0])
                                .draw([
                                      position.x,
                                      position.y,
                                      1.0f64, 1.0f64
                                ], &c, gl);
                        }
                    }
                }
            });
        } //gl cell
        } // meta
    }
}

pub struct MoveSystem {
    meta: Option<Rc<Meta>>
}

impl System for MoveSystem {
    type Components = Components;
}

impl EntityProcess for MoveSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components>) {
        use graphics::*;
        use ShapeVarient::*;
        use ClampVarient::*;
        for e in entities {
            let (vx, vy) = {
                let v = &data.velocities[e];
                (v.x, v.y)
            };
            let shape = data.shapes[e].clone();
            let clamp = data.clamps[e].clone();
            if let Some(ref meta) = self.meta {
                let dt = meta.update.get().dt;
                let render = meta.render.get();
                let view_width = render.width as f64 - 2.0 * WINDOW_PADDING;
                let view_height = render.height as f64 - 2.0 * WINDOW_PADDING;

                let (px, py) = {
                    let position = &mut(data.positions[e]);
                    position.x += vx * dt;
                    position.y += vy * dt;
                    (position.x, position.y)
                };

                let (w, h) = match shape.varient {
                    Circle(r) => (r*2.0, r*2.0),
                    Square(w,h) => (w, h),
                    Point => (0.0, 0.0)
                };

                let velocity_mult = match clamp.varient {
                  Bounce => -1.0,
                  Stop => 0.0,
                  _ => 1.0
                };

                match clamp.varient {
                  Bounce | Stop => {
                    if px + w > view_width {
                      {
                          let position = &mut(data.positions[e]);
                          position.x = view_width - w;
                      }
                      let velocity  = &mut(data.velocities[e]);
                      velocity.x *= velocity_mult;
                    } else if px < 0.0 {
                      {
                          let position = &mut(data.positions[e]);
                          position.x = 0.0;
                      }
                      let velocity  = &mut(data.velocities[e]);
                      velocity.x *= velocity_mult;
                    }
                    if py + h > view_height {
                      {
                          let position = &mut(data.positions[e]);
                          position.y = view_height - h;
                      }
                      let velocity  = &mut(data.velocities[e]);
                      velocity.y *= velocity_mult;
                    } else if py < 0.0 {
                      {
                          let position = &mut(data.positions[e]);
                          position.y = 0.0;
                      }
                      let velocity  = &mut(data.velocities[e]);
                      velocity.y *= velocity_mult;
                    }
                  },
                  Remove => {
                    if px > view_width
                    || px + w < 0.0 {
                        println!("Should remove, went off horizontal edge");
                    }
                    if py > view_height
                    || py + h < 0.0 {
                        println!("Should remove, went off vertical edge");
                    }
                  }
                }
            }
        }
    }
}

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
    up: keyboard::Key,
    down: keyboard::Key
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
        #[hot] clamps: WindowClamp
    }
}

systems! {
    Systems<Components> {
        shimmer: EntitySystem<ShimmerSystem> = EntitySystem::new( ShimmerSystem,
            aspect!(<Components> all: [colors, shimmers])
        ),
        moves: EntitySystem<MoveSystem> = EntitySystem::new(
            MoveSystem{ meta: None },
            aspect!(<Components> all: [positions, shapes, velocities, clamps])
        ),
        draw: EntitySystem<DrawSystem> = EntitySystem::new(
            DrawSystem{ gl: None, meta: None },
            aspect!(<Components> all: [positions, shapes, colors])
        )
    }
}

fn make_ball(world: &mut World<Components, Systems>) {
    const BALL_R: f64 = 20.0;
    let x = (VIEW_W - BALL_R) / 2.0;
    let y = (VIEW_H - BALL_R) / 2.0;

    let entity = world.create_entity(|entity: BuildData, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x,
                y: y
        });
        data.velocities.add(&entity,
            Velocity{
                x: 300.0,
                y: 200.0,
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
        data.clamps.add(&entity,
            WindowClamp {
               varient: ClampVarient::Bounce
        });
    });
}

fn make_player(world: &mut World<Components, Systems>, p1: bool) {
    const FROM_WALL: f64 = 40.0;
    const PADDLE_W: f64 = 20.0;
    const PADDLE_H: f64 = 150.0;
    let x = if p1 {
        FROM_WALL
    } else {
        VIEW_W - FROM_WALL - PADDLE_W
    };
    let y = (VIEW_H - PADDLE_H) / 2.0;
    let entity = world.create_entity(|entity: BuildData, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x,
                y: y
        });
        data.shapes.add(&entity,
            Shape {
                varient: ShapeVarient::Square(PADDLE_W, PADDLE_H),
                border: None
        });
        data.colors.add(&entity,
            Color{
                r: 1.0,
                g: 0.5,
                b: 1.0
        });
        data.player_controllers.add(&entity,
            PlayerController {
                up: if p1 { keyboard::Key::W } else { keyboard::Key::I },
                down: if p1 { keyboard::Key::S } else { keyboard::Key::K },
        });
    });
}

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
    make_player(&mut world, true);
    make_player(&mut world, false);
    let mut meta = Rc::new(Meta {
        render: Cell::new(RenderArgs { ext_dt: 0.0, width: 0, height: 0 }),
        update: Cell::new(UpdateArgs { dt: 0.0 })
    });

    world.systems.moves.meta = Some(meta.clone());
    world.systems.draw.meta = Some(meta.clone());

    for e in event::events(window) {
        use event::{ ReleaseEvent, UpdateEvent, PressEvent, RenderEvent };

        if let Some(args) = e.update_args() {
            meta.update.set(args);
            world.update();
        }
        if let Some(args) = e.render_args() {
            meta.render.set(args);
            process!(world, draw);
        }
    }
}
