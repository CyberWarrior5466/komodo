use adw::prelude::*;
use gtk::Orientation;

pub fn create() -> (gtk::Revealer, gtk::Button) {
    let btn_box = gtk::Box::new(Orientation::Vertical, 8);
    btn_box.add_css_class("darker");

    let toolbar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    toolbar.add_css_class("linked");

    let continue_ = gtk::Button::builder()
        .icon_name("skip-forward-large-symbolic")
        .build();
    let step_over = gtk::Button::builder()
        .icon_name("step-over-symbolic")
        .build();
    // let step_in = gtk::Button::builder()
    //     .icon_name("arrow-pointing-at-line-down-symbolic")
    //     .build();
    // let step_out = gtk::Button::builder()
    //     .icon_name("arrow-pointing-away-from-line-up-symbolic")
    //     .build();
    let restart = gtk::Button::builder()
        .icon_name("arrow-circular-top-left-symbolic")
        .build();
    let stop = gtk::Button::builder().icon_name("stop-symbolic").build();

    toolbar.append(&continue_);
    toolbar.append(&step_over);
    // toolbar.append(&step_out);
    // toolbar.append(&step_in);
    toolbar.append(&restart);
    toolbar.append(&stop);

    let toggle_group = adw::ToggleGroup::builder()
        .orientation(Orientation::Vertical)
        .build();
    toggle_group.add(
        adw::Toggle::builder()
            .icon_name("view-dual-symbolic")
            .name("view-source")
            .build(),
    );
    toggle_group.add(
        adw::Toggle::builder()
            .icon_name("view-grid-symbolic")
            .name("view-disasm")
            .build(),
    );

    toggle_group.connect_active_name_notify(|tg| {
        let name = tg.active_name().unwrap();
        if name == "view-source" {
            tg.activate_action("win.action-view-source", None).unwrap();
        } else if name == "view-disasm" {
            tg.activate_action("win.action-view-disasm", None).unwrap();
        } else {
            panic!();
        }
    });

    btn_box.append(&toolbar);
    btn_box.append(&toggle_group);

    (
        gtk::Revealer::builder()
            .child(&btn_box)
            .reveal_child(false)
            .transition_type(gtk::RevealerTransitionType::SlideLeft)
            .build(),
        stop,
    )
}
