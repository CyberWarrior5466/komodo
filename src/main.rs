use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::AdwApplicationWindowExt;
use adw::prelude::AnimationExt;
use gtk::Align;
use gtk::CssProvider;
use gtk::IconTheme;
use gtk::Orientation;
use gtk::gdk::Display;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let app = adw::Application::new(Some("com.my-gtk-app"), Default::default());

    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    let quit_action = gio::ActionEntry::builder("quit")
        .activate(move |app: &adw::Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit_action]);
    app.set_accels_for_action("app.quit", &["<control>q"]);

    app.set_accels_for_action("win.toggle-side", &["<control>b"]);
    app.set_accels_for_action("win.toggle-bottom", &["<control>j"]);

    app.run()
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_string(
        // Orange 2
        "
        .orange { color: #ffa348; }
        .font-12 { font-size: 12px; }
        .no-min-height { min-height: 0px; }
        listview cell { padding: 1px 8px; }",
    );

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application) {
    gio::resources_register_include!("compiled.gresource").unwrap();

    let display = gtk::gdk::Display::default().unwrap();
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/com/my-gtk-app");

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .build();

    let container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .build();

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder()
        .title_widget(&create_button_container())
        .build();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&container));

    let main_section = create_main_section();
    let side_section = create_side_section();
    container.append(&create_panes(&window, &main_section, &side_section));
    toolbar.add_bottom_bar(&create_bottom_bar());

    window.set_content(Some(&toolbar));
    window.present();
}

fn create_button_container() -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["linked"])
        .halign(Align::Center)
        .build();

    let execute_btn = gtk::Button::builder()
        .icon_name("execute-from-symbolic")
        .build();

    let debug_btn_box = gtk::Box::new(Orientation::Horizontal, 8);

    let debug_btn_icon = gtk::Image::builder()
        .icon_name("bug-symbolic")
        .css_classes(["orange"])
        .build();

    let debug_btn_label = gtk::Label::new(Some("Debug"));
    debug_btn_box.append(&debug_btn_icon);
    debug_btn_box.append(&debug_btn_label);
    let debug_btn = gtk::Button::builder().child(&debug_btn_box).build();

    container.append(&execute_btn);
    container.append(&debug_btn);

    return container;
}

fn create_panes(
    window: &adw::ApplicationWindow,
    main_section: &impl IsA<gtk::Widget>,
    side_section: &impl IsA<gtk::Widget>,
) -> gtk::Paned {
    let bottom_pane = gtk::Paned::builder()
        .orientation(Orientation::Vertical)
        .wide_handle(true)
        .start_child(main_section)
        .end_child(&gtk::Label::new(Some("Bottom")))
        .build();

    let side_pane = gtk::Paned::builder()
        .wide_handle(true)
        .vexpand(true)
        .start_child(side_section)
        .end_child(&bottom_pane)
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

fn create_main_section() -> gtk::ScrolledWindow {
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

fn create_side_section() -> gtk::ScrolledWindow {
    type Reg = (usize, String, i32);
    let registers: Vec<Reg> = vec![
        (0, "r0".into(), 0),
        (1, "r1".into(), 0),
        (2, "r2".into(), 0),
        (3, "r3".into(), 0),
        (4, "r4".into(), 0),
        (5, "r5".into(), 0),
        (6, "r6".into(), 0),
        (7, "r7".into(), 0),
        (8, "r8".into(), 0),
        (9, "r9".into(), 0),
        (10, "r10".into(), 0),
        (11, "r11".into(), 0),
        (12, "r12".into(), 0),
        (13, "r13_sp".into(), 0),
        (14, "r14_lr".into(), 0),
        (15, "r15_pc".into(), 0),
        (16, "apsr".into(), 0),
    ];

    let vec = Rc::new(RefCell::new(
        registers
            .into_iter()
            .map(|v| glib::BoxedAnyObject::new(v))
            .collect::<Vec<glib::BoxedAnyObject>>(),
    ));
    let model = gio::ListStore::new::<glib::BoxedAnyObject>();
    model.extend_from_slice(&vec.borrow());

    let column_view = gtk::ColumnView::new(Some(gtk::NoSelection::new(Some(model.clone()))));

    let register_factory = gtk::SignalListItemFactory::new();
    let value_factory = gtk::SignalListItemFactory::new();

    register_factory.connect_setup(|_, list_item_obj| {
        list_item_obj
            .downcast_ref::<gtk::ColumnViewCell>()
            .unwrap()
            .set_child(Some(
                &gtk::Label::builder()
                    .halign(Align::Start)
                    .css_classes(["font-12"])
                    .width_chars(5)
                    .build(),
            ));
    });
    value_factory.connect_setup(|_, list_item_obj| {
        list_item_obj
            .downcast_ref::<gtk::ColumnViewCell>()
            .unwrap()
            .set_child(Some(&create_spin_button()))
    });

    register_factory.connect_bind(|_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();

        let num_obj = list_item
            .item()
            .and_downcast::<glib::BoxedAnyObject>()
            .unwrap();
        let (_, reg_name, _) = num_obj.borrow::<Reg>().clone();

        list_item
            .child()
            .and_downcast::<gtk::Label>()
            .unwrap()
            .set_label(&reg_name);
    });

    let vec_clone = vec.clone();
    value_factory.connect_bind(move |_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();

        let num_obj = list_item
            .item()
            .and_downcast::<glib::BoxedAnyObject>()
            .unwrap();

        let (reg_i, _, reg_value) = num_obj.borrow::<Reg>().clone();

        let button = list_item.child().and_downcast::<gtk::SpinButton>().unwrap();
        button.set_value(reg_value.into());

        let vec_clone2 = vec_clone.clone();
        button.connect_value_changed(move |btn| {
            let v = vec_clone2.borrow_mut();
            v[reg_i].borrow_mut::<Reg>().2 = btn.value_as_int();

            // let regs = v
            //     .iter()
            //     .map(|reg| reg.borrow::<Reg>().clone())
            //     .collect::<Vec<Reg>>();
            // println!("{:?}", regs)
        });
    });

    let register_column = gtk::ColumnViewColumn::builder()
        .title("Register")
        .factory(&register_factory)
        .resizable(true)
        .build();
    let value_column = gtk::ColumnViewColumn::builder()
        .title("Value")
        .factory(&value_factory)
        .resizable(true)
        .expand(true)
        .build();
    column_view.insert_column(0, &register_column);
    column_view.insert_column(1, &value_column);

    let scroll = gtk::ScrolledWindow::builder().child(&column_view).build();

    return scroll;
}

fn create_bottom_bar() -> gtk::HeaderBar {
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

fn create_spin_button() -> gtk::SpinButton {
    let adjustment = gtk::Adjustment::new(0.0, i32::MIN.into(), i32::MAX.into(), 1.0, 0.0, 0.0);
    let spin_button = gtk::SpinButton::builder()
        .adjustment(&adjustment)
        .css_classes(["font-12", "no-min-height"])
        .valign(Align::Start)
        .hexpand(true)
        .build();
    let last_child = spin_button.last_child().unwrap();
    let second_last_child = last_child.prev_sibling().unwrap();
    last_child.set_visible(false);
    second_last_child.set_visible(false);
    return spin_button;
}
