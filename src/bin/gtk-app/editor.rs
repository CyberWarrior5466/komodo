pub fn create() -> gtk::ScrolledWindow {
    let style_scheme = sourceview5::StyleSchemeManager::new()
        .scheme("Adwaita-dark")
        .expect("style scheme Adwaita-dark exists");

    let buffer = sourceview5::Buffer::builder()
        .style_scheme(&style_scheme)
        .build();

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

    return scroll;
}
