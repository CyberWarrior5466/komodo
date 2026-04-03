pub fn create() -> (gtk::ScrolledWindow, gtk::TextView) {
    let text_view = gtk::TextView::builder()
        .monospace(true)
        // .can_focus(false)
        .editable(false)
        .css_name("bottom_textview")
        .build();

    // let controller = gtk::EventControllerKey::new();
    // controller.connect_key_pressed(|a, b, c, d| {
    //     println!("{:?} {:?} {:?} {:?}", a, b, c, d);
    //     Propagation::Stop
    // });
    // text_view.add_controller(controller.clone());

    let scroll = gtk::ScrolledWindow::builder().child(&text_view).build();

    (scroll, text_view)
}
