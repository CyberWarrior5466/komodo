use adw::prelude::*;

use crate::bottom_pane::custom_button::CustomBin;
mod custom_button;

pub fn create() -> gtk::Box {
    let box_ = gtk::Box::builder().css_classes(["darker"]).build();

    // https://discourse.gnome.org/t/how-is-gtkwidget-focus-on-click-supposed-to-work/19919

    let bin = CustomBin::new();
    // bin.set_label(gtk::Label::new(Some("")));
    bin.set_hexpand(true);
    bin.set_focusable(true);
    bin.set_focus_on_click(true);
    bin.add_css_class("fz");

    // bin.child;

    // let b = custom_button::CustomBin::new();
    // bin.set_child(Some(&b));

    // let gesture_click = gtk::GestureClick::new();
    // gesture_click.connect_pressed(glib::clone!(
    //     #[strong]
    //     bin,
    //     move |_, _, _, _| {
    //         bin.grab_focus();
    //         println!("pressed")
    //     }
    // ));
    // bin.add_controller(gesture_click);

    // gtk::Snapshot::new().append_color(color, bounds);

    // let cursor = gdk::Cursor::from_name("text", None).unwrap();
    // bin.set_cursor(Some(&cursor));

    // let label = gtk::Label::new(Some("label"));

    // label.set_can_focus(true);
    // label.set_focusable(true);
    // label.set_sensitive(true);
    // label.set_focus_on_click(true);

    // let context = gtk::IMMulticontext::new();
    // let controller = gtk::EventControllerKey::new();
    // controller.set_im_context(Some(&context));

    // context.connect_commit(|_, _| {
    //     println!("commit");
    // });
    // controller.connect_key_pressed(|_, _, _, _| {
    //     println!("key pressed");
    //     glib::Propagation::Proceed
    // });

    // label.add_controller(controller);

    box_.append(&bin);
    box_
}
