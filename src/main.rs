#![feature(phase)]
#![feature(if_let)]

#[phase(plugin, link)]
extern crate ecs;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;
use std::cell::RefCell;
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

        let halfw = w / 2.0;
        let halfh = h / 2.0;
        for e in es.iter() {
          if let Some(ref mut position) = data.borrow::<Position>(*e) {
            let dx = halfw - position.x;
            let dy = halfh - position.y;
            let dx2 = dx * dx;
            let dy2 = dy * dy;
            let maxd2 = halfw * halfw + halfh * halfh;
            let d2 = dx2 + dy2;
            let rad = position.rad * (1.0 - d2 / maxd2);
            let ref mut rng = rand::task_rng();
            let l = rng.gen_range(0.8, 1.0);
            c.rgb(0.5 * l, 0.2 * l, 0.8 * l)
              .rect(position.x, position.y, rad, rad)
              .border_width(rng.gen_range(1.0, 3.0))
              .draw(gl);
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
          if let Some(ref mut velocity) = data.borrow::<Velocity>(*e) {
              let ref mut rng = rand::task_rng();
              if rng.gen_range(0u, 100u) >= 99 {
                //velocity.y = -velocity.y;
              }
              if rng.gen_range(0u, 100u) >= 99 {
                //velocity.x = -velocity.x;
              }
              xv = velocity.x;
              yv = velocity.y;
          } else {
            return
          }
          if let Some(ref mut position) = data.borrow::<Position>(*e) {
            position.x += xv * dt;
            position.y += yv * dt;
          }
        }
    }
  }
}

component!(Position {
    x: f64,
    y: f64,
    rad: f64
})

component!(Velocity{
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
    world_builder.register_component::<Position>();
    world_builder.register_component::<Velocity>();

    let aspect = Aspect::for_all(component_ids!(Position));
    let draw_sys = DrawSystem::new(ecs_event.clone());
    let sys = BulkEntitySystem::new(box draw_sys, aspect);
    world_builder.register_system(box sys);

    let maspect = Aspect::for_all(component_ids!(Position, Velocity));
    let move_sys = MoveSystem::new(ecs_event.clone());
    let bsys = BulkEntitySystem::new(box move_sys, maspect);
    world_builder.register_system(box bsys);

    let ref mut world = world_builder.build();

    let num_things: i32 = 80;

    let ref mut rng = rand::task_rng();
    for _ in range(0, num_things) {
        let r = 30.0;
        let x = (800.0 - r) / 2.0;
        let y = (600.0 - r) / 2.0;
        let d = Position { x: x, y: y, rad: r };
        let m = Velocity {
          x: rng.gen_range(-80.0, 80.0),
          y: rng.gen_range(-80.0, 80.0),
        };
        world.build_entity(|c: &mut Components, e: Entity|{ 
            c.add(&e, d);
            c.add(&e, m);
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
