use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::debug;
use ecs::scaffold::{Components, Services};
use event::{Event, ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};

pub struct MoveSystem;

impl System for MoveSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for MoveSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use graphics::*;
        use ecs::components::ShapeVariant as shape;
        use ecs::components::ClampVariant::*;
        for ref e in entities {
            let (vx, vy) = {
                let v = &data.velocities[*e];
                (v.x, v.y)
            };
            let shape = data.shapes[*e].clone();
            let clamp = data.clamps[*e].clone();
            let event = data.services.event.clone();
            let event =  event.borrow();
                if let Some(update) = event.update_args() {
                    let dt = update.dt;
                    let view_width = ::WINDOW_W - 2.0 * ::WINDOW_PADDING;
                    let view_height = ::WINDOW_H - 2.0 * ::WINDOW_PADDING;

                    let (px, py) = {
                        let position = &mut(data.positions[*e]);
                        position.x += vx * dt;
                        position.y += vy * dt;
                        (position.x, position.y)
                    };

                    let (w, h) = match shape.variant {
                        shape::Circle(r) => (r, r),
                        shape::Square(w,h) => (w, h),
                        shape::Point => (1.0, 0.0),
                        shape::Line(_) => (0.0, 0.0)
                    };

                    let velocity_mult = match clamp.variant {
                      Bounce => -1.0,
                      Stop => 0.0,
                      _ => 1.0
                    };

                    match clamp.variant {
                      Bounce | Stop => {
                        if px + w > view_width {
                          {
                              let position = &mut(data.positions[*e]);
                              position.x = view_width - w - ::DISP_FUDGE;
                          }
                          let velocity  = &mut(data.velocities[*e]);
                          velocity.x *= velocity_mult;
                        } else if px - w < 0.0 {
                          {
                              let position = &mut(data.positions[*e]);
                              position.x = w + ::DISP_FUDGE;
                          }
                          let velocity  = &mut(data.velocities[*e]);
                          velocity.x *= velocity_mult;
                        }
                        if py + h > view_height {
                          {
                              let position = &mut(data.positions[*e]);
                              position.y = view_height - h - ::DISP_FUDGE;
                          }
                          let velocity  = &mut(data.velocities[*e]);
                          velocity.y *= velocity_mult;
                        } else if py - h < 0.0 {
                          {
                              let position = &mut(data.positions[*e]);
                              position.y = h + ::DISP_FUDGE;
                          }
                          let velocity  = &mut(data.velocities[*e]);
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
                let (v, pos)  = (&data.velocities[*e].clone(), &data.positions[*e].clone());
                debug::line(data, [pos.x, pos.y, pos.x + v.x * dt * 10.0, pos.y + v.y * dt * 10.0], 1.0);
            }
        }
    }
}
