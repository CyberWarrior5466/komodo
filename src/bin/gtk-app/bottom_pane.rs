use std::{cell::Cell, rc::Rc};

use gtk::{glib, prelude::*};

const WIDTH: i32 = 199;
const HEIGHT: i32 = 34;

pub fn create() -> gtk::Fixed {
    let layout = gtk::Fixed::new();

    let box_ = create_btn_box();
    box_.set_size_request(WIDTH, HEIGHT);
    layout.put(&box_, 0.0, 0.0);

    let begin_x = Rc::new(Cell::new(0.0));
    let begin_y = Rc::new(Cell::new(0.0));
    let label_x = Rc::new(Cell::new(0.0));
    let label_y = Rc::new(Cell::new(0.0));
    let in_bounds = Rc::new(Cell::new(false));

    let controller = gtk::GestureDrag::new();

    controller.connect_drag_begin(glib::clone!(
        #[strong]
        box_,
        #[strong]
        begin_x,
        #[strong]
        begin_y,
        #[strong]
        label_x,
        #[strong]
        label_y,
        #[strong]
        in_bounds,
        move |_, x, y| {
            let bounds = box_.bounds().unwrap();
            let (x0, y0, _, _) = bounds;
            if is_in_bounds(bounds, x, y) {
                in_bounds.set(true);
                begin_x.set(x);
                begin_y.set(y);
                label_x.set(x - x0 as f64);
                label_y.set(y - y0 as f64);
            } else {
                in_bounds.set(false);
            }
        }
    ));

    controller.connect_drag_update(glib::clone!(
        #[strong]
        layout,
        #[strong]
        box_,
        #[strong]
        begin_x,
        #[strong]
        begin_y,
        #[strong]
        label_x,
        #[strong]
        label_y,
        #[strong]
        in_bounds,
        move |_, dx, dy| {
            if in_bounds.get() {
                layout.move_(
                    &box_,
                    begin_x.get() + dx - label_x.get(),
                    begin_y.get() + dy - label_y.get(),
                );
            }
        }
    ));
    layout.add_controller(controller);

    return layout;
}

fn is_in_bounds(bounds: (i32, i32, i32, i32), x: f64, y: f64) -> bool {
    let x0 = bounds.0 as f64;
    let y0 = bounds.1 as f64;
    let width = bounds.2 as f64;
    let height = bounds.3 as f64;
    let x1 = x0 + width;
    let y1 = y0 + height;

    (x0 <= x && x <= x1) && (y0 <= y && y <= y1)
}

fn create_btn_box() -> gtk::Box {
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    btn_box.add_css_class("linked");

    let continue_ = gtk::Button::builder()
        .icon_name("skip-forward-large-symbolic")
        .build();
    let step_over = gtk::Button::builder()
        .icon_name("step-over-symbolic")
        .build();
    let step_in = gtk::Button::builder()
        .icon_name("arrow-pointing-at-line-down-symbolic")
        .build();
    let step_out = gtk::Button::builder()
        .icon_name("arrow-pointing-away-from-line-up-symbolic")
        .build();
    let restart = gtk::Button::builder()
        .icon_name("arrow-circular-top-left-symbolic")
        .build();
    let stop = gtk::Button::builder().icon_name("stop-symbolic").build();

    btn_box.append(&continue_);
    btn_box.append(&step_over);
    btn_box.append(&step_out);
    btn_box.append(&step_in);
    btn_box.append(&restart);
    btn_box.append(&stop);

    return btn_box;
}
