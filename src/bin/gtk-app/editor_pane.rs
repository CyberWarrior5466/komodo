use gtk::glib;
use sourceview5::{prelude::BufferExt, *};

pub fn create() -> (gtk::ScrolledWindow, sourceview5::Buffer) {
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
        .child(&view)
        .build();

    // let action_run = gio::ActionEntry::builder("run")
    //     .activate(move |_: &adw::ApplicationWindow, _, _| {
    //         let bounds = buffer.bounds();
    //         let text = buffer.text(&bounds.0, &bounds.1, true);
    //         println!("{}", text);

    //         komodo::run_program(input_file, regs, mock);
    //     })
    //     .build();
    // window.add_action_entries([action_run]);

    return (scroll, buffer);
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
