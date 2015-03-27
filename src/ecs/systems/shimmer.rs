use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::scaffold::{Components, Services};
use event::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};

use rand::{self, Rng};

pub struct ShimmerSystem;

impl System for ShimmerSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for ShimmerSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        let event = data.services.event.clone();
        let event =  event.borrow();
        if let Some(update) = event.render_args() {
            for ref e in entities {
                let color = &mut data.colors[e];
                let ref mut rng = rand::thread_rng();
                color.0[0] = rng.gen_range(0.3, 1.0);
                color.0[1] = rng.gen_range(0.3, 1.0);
                color.0[2] = rng.gen_range(0.3, 1.0);
            }
        }
    }
}
