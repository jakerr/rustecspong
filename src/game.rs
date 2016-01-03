#![allow(unused_variables, unused_imports)]

#[macro_use]
extern crate ecs as ecsrs;

extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
//extern crate piston_window;
extern crate rand;
extern crate sdl2_window;
extern crate shader_version;
extern crate vecmath;

mod ecs;

use ecsrs::*;
//use piston::input::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};
//use piston::input::Button::Keyboard;
//use piston::input::keyboard;
use piston::input::*;
use piston::event_loop::*;
use rand::Rng;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use piston::window::{WindowSettings};
//use piston_window::{PistonWindow};

use opengl_graphics::{OpenGL, GlGraphics};
use sdl2_window::Sdl2Window as Window;
use self::ecs::components::*;
use self::ecs::scaffold::{Systems, Components};

const WINDOW_W: f64 = 800.0;
const WINDOW_H: f64 = 600.0;
const WINDOW_PADDING: f64 = 40.0;
const VIEW_W: f64 = (WINDOW_W - 2.0 * WINDOW_PADDING);
const VIEW_H: f64 = (WINDOW_H - 2.0 * WINDOW_PADDING);
const DISP_FUDGE: f64 = 5.0;

fn make_ball(world: &mut World<Systems>) {
    const BALL_R: f64 = 10.0;
    let ref mut rng = rand::thread_rng();
    let xoff = rng.gen_range(-100.0, 100.0);
    let yoff = rng.gen_range(-100.0, 100.0);

    let vx = rng.gen_range(400.0, 500.0);
    let vy = rng.gen_range(400.0, 500.0);

    let x = VIEW_W / 2.0 + xoff;
    let y = VIEW_H / 2.0 + yoff;

    let entity = world.create_entity(|entity: BuildData<Components>, data: &mut Components| {
        data.positions.add(&entity,
            Position{
                x: x,
                y: y
        });
        data.velocities.add(&entity,
            Velocity{
                x: vx,
                y: vy,
        });
        data.shimmers.add(&entity, Shimmer);
        data.shapes.add(&entity,
            Shape {
                variant: ShapeVariant::Circle(BALL_R),
                border: None
        });
        data.colors.add(&entity, [1.0, 0.5, 0.2, 1.0]);
        data.clamps.add(&entity,
            WindowClamp {
               variant: ClampVariant::Bounce
        });
    });
}

fn make_player(world: &mut World<Systems>, p1: bool) {
    const FROM_WALL: f64 = 20.0;
    const PADDLE_W: f64 = 10.0;
    const PADDLE_H: f64 = 60.0;
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
                variant: ShapeVariant::Square(PADDLE_W, PADDLE_H),
                border: None
        });
        data.colors.add(&entity, [0.3, 0.4, 1.0, 1.0]);
        data.player_controllers.add(&entity,
            PlayerController {
                up: if p1 { keyboard::Key::W } else { keyboard::Key::I },
                down: if p1 { keyboard::Key::S } else { keyboard::Key::K },
        });
        data.clamps.add(&entity,
            WindowClamp {
               variant: ClampVariant::Stop
        });
        data.hit_counts.add(&entity, HitCount { recent: false, count: 0 });
    });
}

//#[no_mangle]
fn main() {
    let opengl = OpenGL::V3_2;
    let window: Window = WindowSettings::new(
      "Pong".to_string(),
      piston::window::Size {
          width: WINDOW_W as u32,
          height: WINDOW_H as u32
      }
    ).fullscreen(true).exit_on_esc(true).samples(4).build().unwrap();
    let mut gl = GlGraphics::new(opengl);

    let mut world = World::<Systems>::new();
    world.systems.draw.gl = Some(RefCell::new(gl));
    make_ball(&mut world);
    make_player(&mut world, true);
    make_player(&mut world, false);

    for e in window.events() {
        use piston::input::{ ReleaseEvent, UpdateEvent, PressEvent, RenderEvent};
        *(world.data.services.event.borrow_mut()) = e;
        world.update();
    }
}
