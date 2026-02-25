use adw::prelude::*;

pub fn create() -> gtk::HeaderBar {
    let header = gtk::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("")))
        .show_title_buttons(false)
        .build();

    let toggle_left = gtk::Button::builder()
        .icon_name("dock-left-symbolic")
        .build();
    let toggle_bottom = gtk::Button::builder()
        .icon_name("dock-bottom-symbolic")
        .build();

    toggle_left.connect_clicked(move |button| {
        button
            .activate_action("win.toggle-side", None)
            .expect("The action does not exist.");
    });

    toggle_bottom.connect_clicked(move |button| {
        button
            .activate_action("win.toggle-bottom", None)
            .expect("The action does not exist.");
    });

    header.pack_start(&toggle_left);
    header.pack_end(&toggle_bottom);

    return header;
}
