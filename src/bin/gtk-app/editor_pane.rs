use adw::prelude::*;
use gtk::{glib, prelude::BoxExt};
use sourceview5::{prelude::BufferExt, *};

pub fn create(window: &adw::ApplicationWindow) -> (gtk::Box, sourceview5::Buffer) {
    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let buffer = sourceview5::Buffer::builder()
        .style_scheme(&get_style_scheme())
        .build();

    let adw_style = adw::StyleManager::default();
    adw_style.connect_dark_notify(glib::clone!(
        #[strong]
        buffer,
        move |_| buffer.set_style_scheme(Some(&get_style_scheme()))
    ));

    let view = sourceview5::View::builder()
        .monospace(true)
        .show_line_numbers(true)
        .highlight_current_line(true)
        .buffer(&buffer)
        .build();

    let scroll = gtk::ScrolledWindow::builder()
        .vscrollbar_policy(gtk::PolicyType::External)
        .vexpand(true)
        .hexpand(true)
        .child(&view)
        .build();

    let banner = gtk::Revealer::builder()
        .child(&create_btn_box())
        .reveal_child(false)
        .transition_type(gtk::RevealerTransitionType::SlideLeft)
        .build();
    banner.add_css_class("darker");

    let action_debug = gio::ActionEntry::builder("action-debug")
        .activate(glib::clone!(
            #[strong]
            banner,
            move |_: &adw::ApplicationWindow, _, _| {
                banner.set_reveal_child(!banner.reveals_child());
            }
        ))
        .build();
    window.add_action_entries([action_debug]);

    box_.append(&scroll);
    box_.append(&banner);

    (box_, buffer)
}

fn get_style_scheme() -> sourceview5::StyleScheme {
    if adw::StyleManager::default().is_dark() {
        sourceview5::StyleSchemeManager::new()
            .scheme("Adwaita-dark")
            .unwrap()
    } else {
        sourceview5::StyleSchemeManager::new()
            .scheme("Adwaita")
            .unwrap()
    }
}

fn create_btn_box() -> gtk::Box {
    let btn_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
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

    btn_box
}
