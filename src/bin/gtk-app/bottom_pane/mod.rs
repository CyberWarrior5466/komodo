pub fn create() -> (gtk::ScrolledWindow, gtk::TextView) {
    let text_view = gtk::TextView::builder()
        .monospace(true)
        .can_focus(false)
        .css_name("bottom_textview")
        .build();

    let scroll = gtk::ScrolledWindow::builder().child(&text_view).build();

    // scroll.scroll

    (scroll, text_view)
}
