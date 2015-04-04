pub mod systems;
pub mod components;

use self::components::*;
use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;

pub mod debug {
    use ecsrs::*;
    use ecs::scaffold::*;
    use ecs::components::*;
    use graphics::{self};

    pub fn ghost(d: &mut DataHelper<Components, Services>, entity: &EntityData<Components>) {
        let ghost = entity.clone();
        d.create_entity(ghost);
    }

    pub fn line(d: &mut DataHelper<Components, Services>, line: [f64; 4], speed: f32) {
        d.create_entity(|entity: BuildData<Components>, data: &mut Components| {
            data.positions.add(&entity,
                Position{
                    x: (line[0] + line[2]) / 2.0,
                    y: (line[1] + line[3]) / 2.0
                }
            );
            data.shapes.add(&entity,
                Shape {
                    variant: ShapeVariant::Line(line),
                    border: None
                }
            );
            data.colors.add(&entity, [0.0, 0.8, 0.0, 1.0]);
            data.fades.add(&entity, Fade(speed));
        });
    }
}

pub mod scaffold {
    use ecs;
    use event::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};
    use ecsrs::system::{EntityProcess, EntitySystem};
    use ecsrs::*;
    use super::components::*;
    use std::cell::RefCell;
    use std::cell::Cell;
    use std::rc::Rc;

    components! {
        Components {
            #[hot] clamps: WindowClamp,
            #[hot] colors: Color,
            #[hot] fades: Fade,
            #[hot] hit_counts: HitCount,
            #[hot] player_controllers: PlayerController,
            #[hot] positions: Position,
            #[hot] shapes: Shape,
            #[hot] shimmers: Shimmer,
            #[hot] velocities: Velocity,
        }
    }

    systems! {
        Systems<Components, Services> {
            fade: EntitySystem<ecs::systems::FadeSystem> = EntitySystem::new(
                ecs::systems::FadeSystem,
                aspect!(<Components> all: [colors, fades])
            ),
            control: EntitySystem<ecs::systems::ControlSystem> = EntitySystem::new(
                ecs::systems::ControlSystem,
                aspect!(<Components> all: [player_controllers, velocities])
            ),
            collisions: EntitySystem<ecs::systems::CollisionSystem> = EntitySystem::new(
                ecs::systems::CollisionSystem,
                aspect!(<Components> all: [positions, shapes, velocities])
            ),
            moves: EntitySystem<ecs::systems::MoveSystem> = EntitySystem::new(
                ecs::systems::MoveSystem,
                aspect!(<Components> all: [positions, shapes, velocities, clamps])
            ),
            shimmer: EntitySystem<ecs::systems::ShimmerSystem> = EntitySystem::new(
                ecs::systems::ShimmerSystem,
                aspect!(<Components> all: [colors, shimmers])
            ),
            draw: EntitySystem<ecs::systems::DrawSystem> = EntitySystem::new(
                ecs::systems::DrawSystem{ gl: None },
                aspect!(<Components> all: [positions, shapes, colors])
            ),
            game: EntitySystem<ecs::systems::GameSystem> = EntitySystem::new(
                ecs::systems::GameSystem,
                aspect!(<Components> all: [hit_counts])
            )
        }
    }

    services! {
        Services {
            event: RefCell<Event> =
                RefCell::new(Event::Update(UpdateArgs { dt: 3.14 }))
        }
    }

    impl<'a> EntityBuilder<Components> for EntityData<'a, Components> {
        fn build<'b>(&mut self, b: BuildData<'b, Components>, t: &mut Components) {
            if t.colors.has(self) {
                let color = t.colors[*self];
                t.colors.add(&b, color);
            }
            if t.shapes.has(self) {
                let shape = t.shapes[*self].clone();
                t.shapes.add(&b, shape);
            }
            if t.positions.has(self) {
                let pos = t.positions[*self].clone();
                t.positions.add(&b, pos);
            }
            t.fades.add(&b, Fade(0.01));
        }
    }
}
