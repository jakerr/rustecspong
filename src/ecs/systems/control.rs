use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::scaffold::{Components, Services};

pub struct ControlSystem;

impl System for ControlSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for ControlSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use input::Button::Keyboard;
        use event::{ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};
        const PADDLE_V: f64 = 800.0;
        for ref e in entities {
            let (vx, vy) = {
                let v = &data.velocities[*e];
                (v.x, v.y)
            };
            let event = data.services.event.clone();
            let event =  event.borrow();
            let (up, down) = {
                let controller  = &(data.player_controllers[*e]);
                (Keyboard(controller.up), Keyboard(controller.down))
            };
            let velocity = &mut(data.velocities[*e]);
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

