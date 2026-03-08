use adw::prelude::*;
use std::cell::Cell;

use gtk::{Orientation, gio, glib};

pub fn create(
    window: &adw::ApplicationWindow,
    main_section: &impl IsA<gtk::Widget>,
    side_section: &impl IsA<gtk::Widget>,
    bottom_section: &impl IsA<gtk::Widget>,
) -> gtk::Paned {
    let bottom_pane = gtk::Paned::builder()
        .orientation(Orientation::Vertical)
        .wide_handle(true)
        .start_child(main_section)
        .end_child(bottom_section)
        .resize_start_child(true)
        .resize_end_child(false)
        .build();

    let side_pane = gtk::Paned::builder()
        .wide_handle(true)
        .vexpand(true)
        .start_child(side_section)
        .end_child(&bottom_pane)
        .resize_start_child(false)
        .resize_end_child(true)
        .build();

    // source: https://gemini.google.com/share/c9f5bf94dc68
    let paned_weak = bottom_pane.downgrade();
    bottom_pane.connect_map(move |_| {
        let paned_weak = paned_weak.clone();

        glib::idle_add_local(move || {
            let upgrade = paned_weak.upgrade().unwrap();
            upgrade.set_position(upgrade.max_position() / 2);
            return glib::ControlFlow::Break;
        });
    });

    let paned_weak = side_pane.downgrade();
    side_pane.connect_map(move |_| {
        let paned_weak = paned_weak.clone();

        glib::idle_add_local(move || {
            let upgrade = paned_weak.upgrade().unwrap();
            upgrade.set_position(upgrade.max_position() / 4);
            return glib::ControlFlow::Break;
        });
    });
    // end source

    let anim_s = adw::TimedAnimation::new(
        &side_pane,
        0.0,
        0.0,
        200, // 200ms
        adw::PropertyAnimationTarget::new(&side_pane, "position"),
    );

    let anim_b = adw::TimedAnimation::new(
        &bottom_pane,
        0.0,
        0.0,
        200, // 200ms
        adw::PropertyAnimationTarget::new(&bottom_pane, "position"),
    );

    let side_initial = Cell::new(0);
    let side_last_known = Cell::new(0);
    let action_toggle_side = gio::ActionEntry::builder("toggle-side")
        .activate(glib::clone!(
            #[strong]
            side_pane,
            move |_: &adw::ApplicationWindow, _, _| {
                let pos = side_pane.position();

                if side_initial.get() == 0 {
                    side_initial.set(pos);
                }

                if pos > 0 {
                    anim_s.set_value_from(pos as f64);
                    anim_s.set_value_to(0.0);
                    side_last_known.set(pos);
                } else {
                    anim_s.set_value_from(0.0);
                    let lk = side_last_known.get();
                    if lk == 0 {
                        anim_s.set_value_to(side_initial.get() as f64);
                    } else {
                        anim_s.set_value_to(lk as f64);
                    }
                }
                anim_s.play();
            }
        ))
        .build();

    let bottom_initial = Cell::new(0);
    let bottom_last_known = Cell::new(0);
    let action_toggle_bottom = gio::ActionEntry::builder("toggle-bottom")
        .activate(move |_: &adw::ApplicationWindow, _, _| {
            let pos = bottom_pane.position();

            if bottom_initial.get() == 0 {
                bottom_initial.set(pos);
            }

            if pos < bottom_pane.max_position() {
                // expand
                anim_b.set_value_from(pos as f64);
                anim_b.set_value_to(bottom_pane.max_position() as f64);
                bottom_last_known.set(pos);
            } else {
                // contract
                anim_b.set_value_from(bottom_pane.max_position() as f64);
                let lk = bottom_last_known.get();
                if lk == 0 {
                    anim_b.set_value_to(bottom_initial.get() as f64);
                } else {
                    anim_b.set_value_to(lk as f64);
                }
            }
            anim_b.play();
        })
        .build();

    window.add_action_entries([action_toggle_side, action_toggle_bottom]);

    return side_pane;
}
