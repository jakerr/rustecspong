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

pub struct ShimmerSystem;

impl System for ShimmerSystem {
    type Components = Components;
    type Services = Services;
}

pub struct DrawSystem {
    gl: Option<RefCell<Gl>>,
}

impl EntityProcess for ShimmerSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        for e in entities {
            let color = &mut data.colors[e];
            let ref mut rng = rand::thread_rng();
            color.0[0] = rng.gen_range(0.5, 1.0);
            color.0[1] = rng.gen_range(0.5, 1.0);
            color.0[2] = rng.gen_range(0.5, 1.0);
        }
    }
}

impl System for DrawSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for DrawSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use graphics::*;
        use ShapeVarient::*;
        let pad = WINDOW_PADDING;
        if let Some(ref gl_cell) = self.gl {
            let mut gl = gl_cell.borrow_mut();
            let event = data.services.event.borrow();
            if let Some(render) = event.render_args() {
                let view_width = render.width as f64 - 2.0 * pad;
                let view_height = render.height as f64 - 2.0 * pad;
                gl.draw([pad as i32, pad as i32, view_width as i32, view_height as i32], |c, gl| { // viewport
                    graphics::clear([0.2, 0.2, 0.2, 1.0], gl);
                    graphics::rectangle([0.0, 1.0, 0.0, 1.0],
                                        [0.0, 0.0, view_width, view_height],
                                        c.transform,
                                        gl);
                    graphics::rectangle([0.0, 0.0, 0.0, 1.0],
                                        [1.0, 1.0, view_width - 2.0,  view_height - 2.0],
                                        c.transform,
                                        gl);
                    for e in entities {
                        let position = &data.positions[e];
                        let shape = &data.shapes[e];
                        let graphics::Color(color) = data.colors[e];
                        match shape.varient {
                            Circle(rad) => {
                                let circle = Ellipse::new(color);
                                circle.draw(
                                    graphics::ellipse::centered([
                                        position.x,
                                        position.y,
                                        rad, rad
                                    ]),
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            }
                            Square(w, h) => {
                                let square = Rectangle::new(color);
                                square.draw(
                                    graphics::rectangle::centered([
                                        position.x,
                                        position.y,
                                        w, h
                                    ]),
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            },
                            Point => {
                                let pixel = Rectangle::new(color);
                                pixel.draw(
                                    [
                                        position.x,
                                        position.y,
                                        0.5, 0.5
                                    ],
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            }
                        }
                    }
                });
            } // if let Some(render)
        } //gl cell
    }
}

pub struct CollisionSystem;

impl System for CollisionSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use ShapeVarient::*;

        let event = data.services.event.clone();
        let event =  event.borrow();
        if let Some(update) = event.update_args() {
            let evec: Vec<EntityData<Components>> = entities.collect();
            for (i, e1) in evec.iter().enumerate() {
                let shape1 = data.shapes[*e1].clone();

                let targets = evec.iter().skip(i+1);
                for e2 in targets {
                    let p1 = data.positions[*e1].clone();
                    let p2 = data.positions[*e2].clone();
                    let dist2 = {
                        let d2 = (p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y);
                        d2
                    };
                    let shape2 = data.shapes[*e2].clone();
                    match (e1, e2, p1, p2, &shape1.varient, &shape2.varient) {
                        (_, _, _, _, &Circle(r1), &Circle(r2)) => {
                            if (r1 + r2)*(r1 + r2) > dist2 {
                            }
                        }
                        (s, c, square_center, circle_center, &Square(w,h), &Circle(r)) |
                        (c, s, circle_center, square_center, &Circle(r), &Square(w, h)) => {
                            let mut px = circle_center.clone();
                            let mut vx = 1.0;
                            let mut vy = 1.0;
                            if circle_center.x < square_center.x - w { px.x = square_center.x - w; vx = -1.0 }
                            if circle_center.x > square_center.x + w { px.x = square_center.x + w; vx = -1.0 }
                            if circle_center.y < square_center.y - h { px.y = square_center.y - h; vy = -1.0 }
                            if circle_center.y > square_center.y + h { px.y = square_center.y + h; vy = -1.0 }

                            let dx = (px.x - circle_center.x);
                            let dy = (px.y - circle_center.y);
                            let dist2 = dx * dx + dy * dy;

                            if r*r > dist2 {
                                println!("Circle / Square collided!!");
                                *(&mut(data.positions[*c].x)) = px.x + r + 10.0;
                                *(&mut(data.velocities[*c].x)) *= vx;
                                *(&mut(data.velocities[*c].y)) *= vy;
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}

pub struct MoveSystem;

impl System for MoveSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for MoveSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
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
            let event = data.services.event.clone();
            let event =  event.borrow();
                if let Some(update) = event.update_args() {
                    let dt = update.dt;
                    let view_width = WINDOW_W as f64 - 2.0 * WINDOW_PADDING;
                    let view_height = WINDOW_H as f64 - 2.0 * WINDOW_PADDING;

                    let (px, py) = {
                        let position = &mut(data.positions[e]);
                        position.x += vx * dt;
                        position.y += vy * dt;
                        (position.x, position.y)
                    };

                    let (w, h) = match shape.varient {
                        Circle(r) => (r, r),
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
                        } else if px - w < 0.0 {
                          {
                              let position = &mut(data.positions[e]);
                              position.x = w;
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
                        } else if py - h < 0.0 {
                          {
                              let position = &mut(data.positions[e]);
                              position.y = h;
                          }
                          let velocity  = &mut(data.velocities[e]);
                          velocity.y *= velocity_mult;
                        }
                      },
                      Remove => {
                        if px - w > view_width
                        || px + w < 0.0 {
                            println!("Should remove, went off horizontal edge");
                        }
                        if py - h > view_height
                        || py + h < 0.0 {
                            println!("Should remove, went off vertical edge");
                        }
                      }
                }
            }
        }
    }
}

pub struct ControlSystem;

impl System for ControlSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for ControlSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use input::Button::Keyboard;
        const PADDLE_V: f64 = 800.0;
        for e in entities {
            let (vx, vy) = {
                let v = &data.velocities[e];
                (v.x, v.y)
            };
            let event = data.services.event.clone();
            let event =  event.borrow();
            let (up, down) = {
                let controller  = &(data.player_controllers[e]);
                (Keyboard(controller.up), Keyboard(controller.down))
            };
            let velocity = &mut(data.velocities[e]);
            event.press(|button| {
                if button == up {
                    velocity.y = -PADDLE_V;
                } else if button == down {
                    velocity.y = PADDLE_V;
                }
            });
            event.release(|button| {
                if button == up
                || button == down {
                    velocity.y = 0.0;
                }
            });
        }
    }
}

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
        #[hot] colors: graphics::Color,
        #[hot] shimmers: Shimmer,
        #[hot] player_controllers: PlayerController,
        #[hot] clamps: WindowClamp
    }
}

systems! {
    Systems<Components, Services> {
        shimmer: EntitySystem<ShimmerSystem> = EntitySystem::new( ShimmerSystem,
            aspect!(<Components> all: [colors, shimmers])
        ),
        control: EntitySystem<ControlSystem> = EntitySystem::new(
            ControlSystem,
            aspect!(<Components> all: [player_controllers, velocities])
        ),
        collisions: EntitySystem<CollisionSystem> = EntitySystem::new(
            CollisionSystem,
            aspect!(<Components> all: [positions, shapes])
        ),
        moves: EntitySystem<MoveSystem> = EntitySystem::new(
            MoveSystem,
            aspect!(<Components> all: [positions, shapes, velocities, clamps])
        ),
        draw: EntitySystem<DrawSystem> = EntitySystem::new(
            DrawSystem{ gl: None },
            aspect!(<Components> all: [positions, shapes, colors])
        )
    }
}

services! {
    Services {
        event: RefCell<Event> =
            RefCell::new(Event::Update(UpdateArgs { dt: 3.14 }))
    }
}

fn make_ball(world: &mut World<Systems>) {
    const BALL_R: f64 = 10.0;
    let x = VIEW_W / 2.0;
    let y = VIEW_H / 2.0;

    let entity = world.create_entity(|entity: BuildData<Components>, data: &mut Components| {
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
        data.colors.add(&entity, graphics::Color([1.0, 0.5, 0.2, 1.0]));
        data.clamps.add(&entity,
            WindowClamp {
               varient: ClampVarient::Bounce
        });
    });
    let entity = world.create_entity(|entity: BuildData<Components>, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x + 20.0,
                y: y + 30.0
        });
        data.velocities.add(&entity,
            Velocity{
                x: 230.0,
                y: 100.0,
        });
        data.shimmers.add(&entity, Shimmer);
        data.shapes.add(&entity,
            Shape {
                varient: ShapeVarient::Circle(BALL_R),
                border: None
        });
        data.colors.add(&entity, graphics::Color([1.0, 0.5, 0.2, 1.0]));
        data.clamps.add(&entity,
            WindowClamp {
               varient: ClampVarient::Bounce
        });
    });
}

fn make_player(world: &mut World<Systems>, p1: bool) {
    const FROM_WALL: f64 = 20.0;
    const PADDLE_W: f64 = 10.0;
    const PADDLE_H: f64 = 75.0;
    let x = if p1 {
        FROM_WALL
    } else {
        VIEW_W - FROM_WALL
    };
    let y = VIEW_H / 2.0;
    let entity = world.create_entity(|entity: BuildData<Components>, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x,
                y: y
        });
        data.velocities.add(&entity,
            Velocity{
                x: 0.0,
                y: 0.0,
        });
        data.shapes.add(&entity,
            Shape {
                varient: ShapeVarient::Square(PADDLE_W, PADDLE_H),
                border: None
        });
        data.colors.add(&entity, graphics::Color([1.0, 0.5, 1.0, 1.0]));
        data.player_controllers.add(&entity,
            PlayerController {
                up: if p1 { keyboard::Key::W } else { keyboard::Key::I },
                down: if p1 { keyboard::Key::S } else { keyboard::Key::K },
        });
        data.clamps.add(&entity,
            WindowClamp {
               varient: ClampVarient::Stop
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

    let mut world = World::<Systems>::new();
    world.systems.draw.gl = Some(RefCell::new(gl));
    make_ball(&mut world);
    make_player(&mut world, true);
    make_player(&mut world, false);

    for e in event::events(window) {
        use event::{ ReleaseEvent, UpdateEvent, PressEvent, RenderEvent};
        *(world.data.services.event.borrow_mut()) = e;
        world.update();
    }
}
