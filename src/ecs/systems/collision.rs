use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::scaffold::{Components, Services};

pub struct CollisionSystem;

impl System for CollisionSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for CollisionSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use vecmath::*;
        use ecs::components::ShapeVariant::*;
        use piston::input::{ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};

        let event = data.services.event.clone();
        let event =  event.borrow();
        if let Some(update) = event.update_args() {
            let evec: Vec<EntityData<Components>> = entities.collect();
            for (i, e1) in evec.iter().enumerate() {
                let shape1 = data.shapes[*e1].clone();

                let targets = evec.iter().skip(i+1);
                for e2 in targets {
                    let p1 = data.positions[*e1].clone();
                    let p2 = data.positions[*e2].clone();
                    let dist2 = {
                        let d2 = (p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y);
                        d2
                    };
                    let shape2 = data.shapes[*e2].clone();
                    match (e1, e2, p1, p2, &shape1.variant, &shape2.variant) {
                        (_, _, _, _, &Circle(r1), &Circle(r2)) => {
                            if (r1 + r2)*(r1 + r2) > dist2 {
                            }
                        }
                        (s, c, square_center, circle_center, &Square(w,h), &Circle(r)) |
                        (c, s, circle_center, square_center, &Circle(r), &Square(w, h)) => {
                            let mut px = circle_center.clone();
                            if circle_center.x < square_center.x - w { px.x = square_center.x - w; }
                            if circle_center.x > square_center.x + w { px.x = square_center.x + w; }
                            if circle_center.y < square_center.y - h { px.y = square_center.y - h; }
                            if circle_center.y > square_center.y + h { px.y = square_center.y + h; }

                            let center: Vector2<f64> = [circle_center.x, circle_center.y];
                            let px: Vector2<f64> = [px.x, px.y];
                            let aligned_r = vec2_scale(vec2_normalized(vec2_sub(px, center)), r);
                            let cedge = vec2_add(center, aligned_r);
                            let disp = vec2_scale(vec2_sub(px, cedge), ::DISP_FUDGE);
                            let neg = vec2_scale(disp, -1.0);

                            let dx = px[0] - center[0];
                            let dy = px[1] - center[1];
                            let dist2 = dx * dx + dy * dy;

                            if r*r > dist2 {
                                if data.hit_counts.has(c) {
                                    if !data.hit_counts[*c].recent {
                                        data.hit_counts[*c].count += 1;
                                        data.hit_counts[*c].recent = true;
                                    }
                                }
                                if data.hit_counts.has(s) {
                                    if !data.hit_counts[*s].recent {
                                        data.hit_counts[*s].count += 1;
                                        data.hit_counts[*s].recent = true;
                                    }
                                }
                                *(&mut(data.positions[*c].x)) += disp[0];
                                *(&mut(data.positions[*c].y)) += disp[1];
                                *(&mut(data.velocities[*s].x)) *= 0.5;
                                *(&mut(data.velocities[*s].y)) *= 0.5;
                                super::super::debug::ghost(data, c);
                                super::super::debug::line(data, [center[0], center[1], center[0] + disp[0], center[1] + disp[1]], 0.01);

                                let v = &mut data.velocities[*c];
                                if disp[0] > 0.0 {
                                    if v.x < 0.0 { v.x *= -1.0 }
                                } else if disp[0] < 0.0 {
                                    if v.x > 0.0 { v.x *= -1.0 }
                                }
                                if disp[1] > 0.0 {
                                    if v.y < 0.0 { v.y *= -1.0 }
                                } else if disp[1] < 0.0 {
                                    if v.y > 0.0 { v.y *= -1.0 }
                                }
                            } else {
                                if data.hit_counts.has(c) {
                                    data.hit_counts[*c].recent = false;
                                }
                                if data.hit_counts.has(s) {
                                    data.hit_counts[*s].recent = false;
                                }
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}
