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

    toggle_left.connect_clicked(move |btn| {
        btn.activate_action("win.action-toggle-side", None).unwrap();
    });
    toggle_bottom.connect_clicked(move |btn| {
        btn.activate_action("win.action-toggle-bottom", None)
            .unwrap();
    });

    header.pack_start(&toggle_left);
    header.pack_end(&toggle_bottom);

    return header;
}
