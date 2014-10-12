#![feature(phase)]
#![feature(if_let)]

#[phase(plugin, link)]
extern crate ecs;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;
use std::cell::RefCell;
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
    RenderArgs,
    UpdateArgs,
};
use piston::graphics::{
    AddBorder,
    AddEllipse,
    AddRectangle,
    AddColor,
    Context,
    Draw,
};
use piston::event::{
    PressEvent,
    RenderEvent,
    UpdateEvent,
    GenericEvent,
};
use piston::input::{
    keyboard,
    Keyboard,
};
use ecs::{
    component,
    Components,
    World,
    WorldBuilder,
    Entity,
    EntityData,
    EntityBuilder,
    EntityModifier,
    Aspect
};
use ecs::system::entitysystem::{
    BulkEntitySystem,
    BulkEntityProcess,
};

static mut update_args: Option<UpdateArgs> = None; 
static mut render_args: Option<RenderArgs> = None; 

struct DrawSystem {
    gl: RefCell<Gl>,
    render_args: Option<RenderArgs>,
}

impl DrawSystem {
    fn new() -> DrawSystem {
        let opengl = piston::shader_version::opengl::OpenGL_3_2;
        DrawSystem {
            gl: RefCell::new(Gl::new(opengl)),
            render_args: None
        }
    }
}

impl BulkEntityProcess for DrawSystem {
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
    println!("{}", self.render_args.unwrap());
    let args = self.render_args.unwrap();
    let w = args.width as f64;
    let h = args.height as f64;
    let mut glCell = self.gl.borrow_mut();
    let gl = glCell.deref_mut();
    gl.viewport(0, 0, w as i32, h as i32);
    let c = Context::abs(w, h);
    // Clear background.
    c.rgb(0.0, 0.0, 0.0).draw(gl);

    let halfw = w / 2.0;
    let halfh = h / 2.0;
    for e in es.iter() {
      if let Some(ref mut position) = data.borrow::<Position>(*e) {
        let dx = (halfw - position.x);
        let dy = (halfh - position.y);
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

impl RenderProcess for DrawSystem {
    fn set_render_args(&mut self, args: Option<RenderArgs>) {
        self.render_args = args;
    }
    fn get_render_args(&self) -> Option<RenderArgs> {
        self.render_args
    }
}


struct MoveSystem {
    update_args: Option<UpdateArgs>
}

impl MoveSystem {
  fn new() -> MoveSystem {
    MoveSystem { update_args: None }
  }
}

impl BulkEntityProcess for MoveSystem {
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
    println!("{}", self.update_args.unwrap());
    if self.update_args.is_none() { return; }
    let dt = self.update_args.unwrap().dt;
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

impl UpdateProcess for MoveSystem {
    fn set_update_args(&mut self, args: Option<UpdateArgs>) {
        self.update_args = args;
    }
    fn get_update_args(&self) -> Option<UpdateArgs> {
        self.update_args
    }
}


trait UpdateProcess: BulkEntityProcess + 'static {
    fn set_update_args(&mut self, args: Option<UpdateArgs>);
    fn get_update_args(&self) -> Option<UpdateArgs>;
}

struct OnUpdateSystem {
    inner: Box<UpdateProcess>
}

impl OnUpdateSystem {
    fn new(inner: Box<UpdateProcess>) -> OnUpdateSystem {
        OnUpdateSystem { inner: inner }
    }
}

impl BulkEntityProcess for OnUpdateSystem {
  fn preprocess(&mut self, _: &World) {
      unsafe {
          self.inner.set_update_args(update_args);
      }
  }
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
      match self.inner.get_update_args() {
          Some(_) => self.inner.process(es, data),
          None => {} 
      }
  }
}

trait RenderProcess: BulkEntityProcess + 'static {
    fn set_render_args(&mut self, args: Option<RenderArgs>);
    fn get_render_args(&self) -> Option<RenderArgs>;
}

struct OnRenderSystem {
    inner: Box<RenderProcess>
}

impl OnRenderSystem {
    fn new(inner: Box<RenderProcess>) -> OnRenderSystem {
        OnRenderSystem { inner: inner }
    }
}

impl BulkEntityProcess for OnRenderSystem {
  fn preprocess(&mut self, _: &World) {
      unsafe {
          self.inner.set_render_args(render_args);
      }
  }
  fn process(&self, es: Vec<&Entity>, data: &mut EntityData) {
      match self.inner.get_render_args() {
          Some(_) => self.inner.process(es, data),
          None => {} 
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

    let mut worldBuilder = WorldBuilder::new();
    worldBuilder.register_component::<Position>();
    worldBuilder.register_component::<Velocity>();

    let aspect = Aspect::for_all(component_ids!(Position));
    let draw_sys = DrawSystem::new();
    let draw_render = OnRenderSystem::new(box draw_sys);
    let sys = BulkEntitySystem::new(box draw_render, aspect);
    worldBuilder.register_system(box sys);

    let maspect = Aspect::for_all(component_ids!(Position, Velocity));
    let mover = MoveSystem::new();
    let mover_update = OnUpdateSystem::new(box mover);
    let bsys = BulkEntitySystem::new(box mover_update, maspect);
    worldBuilder.register_system(box bsys);

    let ref mut world = worldBuilder.build();

    let numThings: i32 = 80;

    let ref mut rng = rand::task_rng();
    for i in range(0, numThings) {
        let r = 30.0;//rng.gen_range(30.0, 60.0); //(10 + i * 10) as f64;
        let x = (800.0 - r) / 2.0; // (800.0 / numThings as f64) * i as f64;
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
        println!("{}", e);
        continue;
        unsafe {
          render_args = None;
          update_args = None;
        }
        e.press(|button| {
            if button == Keyboard(keyboard::G) {
                println!("G");
            } else if button == Keyboard(keyboard::R) {
                println!("Reset");
            }
        });
        e.update(|args| {
          unsafe { update_args = Some(*args); }
          world.update();
        });
        e.render(|args| {
          unsafe { render_args = Some(*args); }
          world.update();
        });
    }
}
