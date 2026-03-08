use adw::prelude::*;
use gtk::{Align, Orientation};

pub fn create() -> gtk::Box {
    let box_ = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["linked"])
        .halign(Align::Center)
        .build();

    let run_btn = gtk::Button::builder()
        .icon_name("execute-from-symbolic")
        .build();

    run_btn.connect_clicked(move |btn| {
        btn.activate_action("win.action-run", None)
            .expect("The action does not exist.");
    });

    let debug_btn_box = gtk::Box::new(Orientation::Horizontal, 8);

    let debug_btn_icon = gtk::Image::builder()
        .icon_name("bug-symbolic")
        .css_classes(["orange"])
        .build();

    let debug_btn_label = gtk::Label::new(Some("Debug"));
    debug_btn_box.append(&debug_btn_icon);
    debug_btn_box.append(&debug_btn_label);
    let debug_btn = gtk::Button::builder().child(&debug_btn_box).build();

    debug_btn.connect_clicked(move |btn| {
        btn.activate_action("win.action-debug", None)
            .expect("The action does not exist.");
    });

    box_.append(&run_btn);
    box_.append(&debug_btn);

    return box_;
}
