use ecsrs::*;
use ecsrs::system::{EntityProcess, EntitySystem};
use ecs::debug;
use ecs::scaffold::{Components, Services};

use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;
use opengl_graphics::{
    Gl,
};

pub struct DrawSystem {
    pub gl: Option<RefCell<Gl>>,
}


impl System for DrawSystem {
    type Components = Components;
    type Services = Services;
}

impl EntityProcess for DrawSystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        use ::graphics;
        use graphics::*;
//        use graphics::types::{Color};
        use ecs::components::ShapeVariant as shape;
        use event::{ReleaseEvent, UpdateEvent, PressEvent, RenderEvent, RenderArgs, UpdateArgs};
        let pad = ::WINDOW_PADDING;
        if let Some(ref gl_cell) = self.gl {
            let mut gl = gl_cell.borrow_mut();
            let event = data.services.event.borrow();
            if let Some(render) = event.render_args() {
                let view_width = render.width as f64 - 2.0 * pad;
                let view_height = render.height as f64 - 2.0 * pad;
                gl.draw([pad as i32, pad as i32, view_width as i32, view_height as i32], |c, gl| { // viewport
                    graphics::clear([0.2, 0.2, 0.2, 1.0], gl);
                    graphics::rectangle([0.0, 1.0, 0.0, 1.0],
                                        [0.0, 0.0, view_width, view_height],
                                        c.transform,
                                        gl);
                    graphics::rectangle([0.0, 0.0, 0.0, 1.0],
                                        [1.0, 1.0, view_width - 2.0,  view_height - 2.0],
                                        c.transform,
                                        gl);
                    for ref e in entities {
                        let position = &data.positions[*e];
                        let shape = &data.shapes[*e];
                        let color = data.colors[*e];
                        match shape.variant {
                            shape::Circle(rad) => {
                                let circle = Ellipse::new(color);
                                circle.draw(
                                    graphics::ellipse::centered([
                                        position.x,
                                        position.y,
                                        rad, rad
                                    ]),
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            }
                            shape::Square(w, h) => {
                                let square = Rectangle::new(color);
                                square.draw(
                                    graphics::rectangle::centered([
                                        position.x,
                                        position.y,
                                        w, h
                                    ]),
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            },
                            shape::Point => {
                                let pixel = Rectangle::new(color);
                                pixel.draw(
                                    [
                                        position.x,
                                        position.y,
                                        0.5, 0.5
                                    ],
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            }
                            shape::Line(l) => {
                                let line = Line::new(color, 1.0);
                                line.draw(
                                    l,
                                    &c.draw_state,
                                    c.transform,
                                    gl
                                );
                            }
                        }
                    }
                });
            } // if let Some(render)
        } //gl cell
    }
}
