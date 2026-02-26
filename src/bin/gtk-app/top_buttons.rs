use adw::prelude::*;
use gtk::{Align, Orientation};

pub fn create() -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["linked"])
        .halign(Align::Center)
        .build();

    let run_button = gtk::Button::builder()
        .icon_name("execute-from-symbolic")
        .build();

    run_button.connect_clicked(move |button| {
        button
            .activate_action("win.run", None)
            .expect("The action does not exist.");
    });

    let debug_button_box = gtk::Box::new(Orientation::Horizontal, 8);

    let debug_button_icon = gtk::Image::builder()
        .icon_name("bug-symbolic")
        .css_classes(["orange"])
        .build();

    let debug_button_label = gtk::Label::new(Some("Debug"));
    debug_button_box.append(&debug_button_icon);
    debug_button_box.append(&debug_button_label);
    let debug_button = gtk::Button::builder().child(&debug_button_box).build();

    container.append(&run_button);
    container.append(&debug_button);

    return container;
}
