use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::scaffold::{Components, Services};
use event::{ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};

pub struct FadeSystem;

impl System for FadeSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for FadeSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        let event = data.services.event.clone();
        let event =  event.borrow();
        if let Some(update) = event.update_args() {
            for e in entities {
                let f = data.fades[e].0;
                let delete = {
                    let mut delete = false;
                    let color = &mut data.colors[e];
                    color.0[3] -= f;
                    if color.0[3] <= 0.0 {
                        delete = true;
                    }
                    delete
                };
                if delete {
                    data.remove_entity(**e);
                }
            }
        }
    }
}
