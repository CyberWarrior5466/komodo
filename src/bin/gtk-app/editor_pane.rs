use adw::prelude::*;
use gtk::glib;
use sourceview5::{prelude::BufferExt, *};

pub fn create(window: &adw::ApplicationWindow) -> (gtk::CenterBox, sourceview5::Buffer) {
    let center_box = gtk::CenterBox::new();
    center_box.set_hexpand(true);

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

    window.set_title(Some("Komodo, University of Nottingham"));
    center_box.set_center_widget(Some(&scroll));

    let action_view_source = gio::ActionEntry::builder("action-view-source")
        .activate(glib::clone!(
            #[strong]
            center_box,
            move |_: &adw::ApplicationWindow, _, _| {
                center_box.set_center_widget(Some(&scroll));
            }
        ))
        .build();

    let label = gtk::Label::new(Some("diasm"));

    let action_view_disasm = gio::ActionEntry::builder("action-view-disasm")
        .activate(glib::clone!(
            #[strong]
            center_box,
            move |_: &adw::ApplicationWindow, _, _| {
                center_box.set_center_widget(Some(&label));
            }
        ))
        .build();
    window.add_action_entries([action_view_source, action_view_disasm]);

    (center_box, buffer)
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
