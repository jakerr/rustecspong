#![feature(phase)]
#![feature(if_let)]

#[phase(plugin, link)]
extern crate ecs;
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
use ecs::{
    Components,
    WorldBuilder,
    Entity,
    EntityData,
    EntityBuilder,
    Aspect
};
use ecs::system::entitysystem::{
    BulkEntitySystem,
    BulkEntityProcess,
};

struct DrawSystem {
    gl: RefCell<Gl>,
    event: Rc<RefCell<Event>>,
}

impl DrawSystem {
    fn new(event: Rc<RefCell<Event>>) -> DrawSystem {
        let opengl = piston::shader_version::opengl::OpenGL_3_2;
        DrawSystem {
            gl: RefCell::new(Gl::new(opengl)),
            event: event
        }
    }
}

impl BulkEntityProcess for DrawSystem {
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
    if let &Render(args) = self.event.borrow().deref() {
        let w = args.width as f64;
        let h = args.height as f64;
        let mut gl_cell = self.gl.borrow_mut();
        let gl = gl_cell.deref_mut();
        gl.viewport(0, 0, w as i32, h as i32);
        let c = Context::abs(w, h);
        // Clear background.
        c.rgb(0.0, 0.0, 0.0).draw(gl);

        for e in es.iter() {
          let mut x: f64 = 0.0;
          let mut y: f64 = 0.0;
          let mut shape = Point;
          let mut border = None;
          if let Some(p) = data.borrow::<PositionComponent>(*e) {
            x = p.x;
            y = p.y;
          }
          if let Some(s) = data.borrow::<ShapeComponent>(*e) {
            shape = s.shape;
            border = s.border;
          }
          let r = 0.5;
          let g = 0.2;
          let b = 0.8;
          match (shape, border) {
              (Point, Some(border)) => {
                  c.rgb(r, g, b)
                    .rect(x, y, 1.0, 1.0)
                    .border_radius(border)
                    .draw(gl);
              },
              (Circle(rad), Some(border)) => {
                  c.rgb(r, g, b)
                    .ellipse(x, y, rad, rad)
                    .border_radius(border)
                    .draw(gl);
              },
              (Square(w,h), Some(border)) => {
                  c.rgb(r, g, b)
                    .rect(x, y, w, h)
                    .border_radius(border)
                    .draw(gl);
              },
              (Point, None) => {
                  c.rgb(r, g, b)
                    .rect(x, y, 1.0, 1.0)
                    .draw(gl);
              },
              (Circle(rad), None) => {
                  c.rgb(r, g, b)
                    .ellipse(x, y, rad, rad)
                    .draw(gl);
              },
              (Square(w,h), None) => {
                  c.rgb(r, g, b)
                    .rect(x, y, w, h)
                    .draw(gl);
              }
          }
        }
    }
  }
}

struct MoveSystem {
    event: Rc<RefCell<Event>>
}

impl MoveSystem {
  fn new(event: Rc<RefCell<Event>>) -> MoveSystem {
    MoveSystem {
        event: event
    }
  }
}

impl BulkEntityProcess for MoveSystem {
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
    if let &Update(args) = self.event.borrow().deref() {
        let dt = args.dt;
        for e in es.iter() {
          let xv;
          let yv;
          if let Some(ref mut velocity) = data.borrow::<VelocityComponent>(*e) {
              xv = velocity.x;
              yv = velocity.y;
          } else {
            return
          }
          if let Some(ref mut position) = data.borrow::<PositionComponent>(*e) {
            position.x += xv * dt;
            position.y += yv * dt;
          }
        }
    }
  }
}

component!(PositionComponent {
    x: f64,
    y: f64
})

#[deriving(Clone, PartialEq, Show)]
pub enum Shape {
    Point,
    Circle(f64), // radius
    Square(f64, f64), // width, height
}

impl Default for Shape {
    fn default() -> Shape { Point }
}

component!(ShapeComponent {
    shape: Shape,
    border: Option<f64>
})

component!(VelocityComponent {
    x: f64,
    y: f64
})

fn main() {
    let ecs_event: Rc<RefCell<Event>> =
        Rc::new(
            RefCell::new(Input(Focus(false)))
        );

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

    let mut world_builder = WorldBuilder::new();
    world_builder.register_component::<PositionComponent>();
    world_builder.register_component::<VelocityComponent>();
    world_builder.register_component::<ShapeComponent>();

    let aspect = Aspect::for_all(component_ids!(PositionComponent));
    let draw_sys = DrawSystem::new(ecs_event.clone());
    let sys = BulkEntitySystem::new(box draw_sys, aspect);
    world_builder.register_system(box sys);

    let maspect = Aspect::for_all(component_ids!(PositionComponent, VelocityComponent));
    let move_sys = MoveSystem::new(ecs_event.clone());
    let bsys = BulkEntitySystem::new(box move_sys, maspect);
    world_builder.register_system(box bsys);

    let ref mut world = world_builder.build();

    let num_things: i32 = 300;

    let ref mut rng = rand::task_rng();
    for _ in range(0, num_things) {
        let r = 30.0;
        let x = (800.0 - r) / 2.0;
        let y = (600.0 - r) / 2.0;
        let d = PositionComponent { x: x, y: y };
        let m = VelocityComponent {
          x: rng.gen_range(-80.0, 80.0),
          y: rng.gen_range(-80.0, 80.0),
        };
        let shape = ShapeComponent {
          shape: if rng.gen() {
                     Square(r, r)
                 } else {
                     Circle(r)
                 },
          border: if rng.gen() {
                     Some(rng.gen_range(1.0, 5.0))
                 } else {
                     None
                 }
        };
        world.build_entity(|c: &mut Components, e: Entity| {
            c.add(&e, d);
            c.add(&e, m);
            c.add(&e, shape);
        });
    }
    let event_settings = EventSettings {
        updates_per_second: 120,
        max_frames_per_second: 60,
    };

    for e in EventIterator::new(&mut window, &event_settings) {
        *ecs_event.borrow_mut() = e.clone();
        world.update();
    }
}
