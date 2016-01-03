use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::debug;
use ecs::scaffold::{Components, Services};
use piston::input::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};

pub struct GameSystem;

impl System for GameSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for GameSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        for ref e in entities {
            let hits = data.hit_counts[*e].clone();
            let col = &mut data.colors[*e];
            col[0] = hits.count as f32 * 0.1;
        }
    }
}

