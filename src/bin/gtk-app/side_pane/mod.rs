pub mod reg_object;

use std::cell::Cell;
use std::rc::Rc;

use gtk::Align;
use gtk::gio;
use gtk::gio::ListStore;
use gtk::glib;
use gtk::prelude::*;

use reg_object::RegObject;

pub fn create(vec: &Vec<RegObject>) -> gtk::ScrolledWindow {
    let model = gio::ListStore::new::<RegObject>();
    model.extend_from_slice(vec);

    let apsr_obj = model.item(16).and_downcast::<RegObject>().unwrap();
    apsr_obj.set_number(komodo::ProcessorMode::User as i32);

    let column_view = gtk::ColumnView::new(Some(gtk::NoSelection::new(Some(model.clone()))));
    column_view.set_vexpand(true);
    column_view.set_margin_bottom(2);

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
            .set_child(Some(&spin_btn_create()))
    });

    register_factory.connect_bind(|_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
        let int_obj = list_item.item().and_downcast::<RegObject>().unwrap();

        list_item
            .child()
            .and_downcast::<gtk::Label>()
            .unwrap()
            .set_label(&int_obj.name());
    });

    let side_pane_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::builder().child(&side_pane_box).build();
    side_pane_box.append(&column_view);

    let toggle_btns = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let n_toggle = toggle_btn_create("N", "Negative");
    let z_toggle = toggle_btn_create("Z", "Zero");
    let c_toggle = toggle_btn_create("C", "Carry");
    let v_toggle = toggle_btn_create("V", "Overflow");
    toggle_btns.append(&n_toggle);
    toggle_btns.append(&z_toggle);
    toggle_btns.append(&c_toggle);
    toggle_btns.append(&v_toggle);

    let signal_from_toggle = Rc::new(Cell::new(false));
    let signal_from_field = Rc::new(Cell::new(false));
    n_toggle.connect_active_notify(glib::clone!(
        #[strong]
        model,
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |_| {
            if signal_from_field.get() {
                return;
            }
            toggle_update_apsr(&model, signal_from_toggle.clone(), |sf| {
                let mut sf_copy = sf.clone();
                sf_copy.negative = !sf_copy.negative;
                sf_copy
            });
        }
    ));
    z_toggle.connect_active_notify(glib::clone!(
        #[strong]
        model,
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |_| {
            if signal_from_field.get() {
                return;
            }
            toggle_update_apsr(&model, signal_from_toggle.clone(), |sf| {
                let mut sf_copy = sf.clone();
                sf_copy.zero = !sf_copy.zero;
                sf_copy
            });
        }
    ));
    c_toggle.connect_active_notify(glib::clone!(
        #[strong]
        model,
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |_| {
            if signal_from_field.get() {
                return;
            }
            toggle_update_apsr(&model, signal_from_toggle.clone(), |sf| {
                let mut sf_copy = sf.clone();
                sf_copy.carry = !sf_copy.carry;
                sf_copy
            });
        }
    ));
    v_toggle.connect_active_notify(glib::clone!(
        #[strong]
        model,
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |_| {
            if signal_from_field.get() {
                return;
            }
            toggle_update_apsr(&model, signal_from_toggle.clone(), |sf| {
                let mut sf_copy = sf.clone();
                sf_copy.overflow = !sf_copy.overflow;
                sf_copy
            });
        }
    ));

    let psr_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .spacing(6)
        .build();
    psr_box.append(&toggle_btns);

    let dropdown = gtk::DropDown::from_strings(&[
        "User",
        "FIQ",
        "IRQ",
        "Supervisor",
        "Abort",
        "Undefined",
        "System",
    ]);

    dropdown.connect_selected_notify(glib::clone!(
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |dropdown| {
            if signal_from_field.get() {
                return;
            }

            use komodo::ProcessorMode::*;
            let processor_mode = match dropdown.selected() {
                0 => User,
                1 => Fiq,
                2 => Irq,
                3 => Supervisor,
                4 => Abort,
                5 => Undefined,
                6 => System,
                _ => panic!(),
            };

            let apsr_obj = model.item(16).and_downcast::<RegObject>().unwrap();
            let mut sf = komodo::StatusFlags::from(apsr_obj.number());
            sf.processor_mode = processor_mode;
            let new_num = komodo::update_from_flags(apsr_obj.number(), &sf);

            signal_from_toggle.set(true);
            apsr_obj.set_number(new_num);
            signal_from_toggle.set(false);
        }
    ));

    psr_box.append(&dropdown);
    side_pane_box.append(&psr_box);

    value_factory.connect_bind(glib::clone!(
        #[strong]
        n_toggle,
        #[strong]
        z_toggle,
        #[strong]
        c_toggle,
        #[strong]
        v_toggle,
        #[strong]
        dropdown,
        #[strong]
        signal_from_toggle,
        #[strong]
        signal_from_field,
        move |_, list_item_obj| {
            let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
            let int_obj = list_item.item().and_downcast::<RegObject>().unwrap();
            let btn = list_item.child().and_downcast::<gtk::SpinButton>().unwrap();

            if int_obj.name() == "apsr" {
                int_obj.connect_number_notify(glib::clone!(
                    #[strong]
                    n_toggle,
                    #[strong]
                    z_toggle,
                    #[strong]
                    c_toggle,
                    #[strong]
                    v_toggle,
                    #[strong]
                    dropdown,
                    #[strong]
                    signal_from_toggle,
                    #[strong]
                    signal_from_field,
                    move |obj| {
                        if signal_from_toggle.get() {
                            return;
                        }

                        let sf = komodo::StatusFlags::from(obj.number());
                        signal_from_field.set(true);
                        n_toggle.set_active(sf.negative);
                        z_toggle.set_active(sf.zero);
                        c_toggle.set_active(sf.carry);
                        v_toggle.set_active(sf.overflow);

                        use komodo::ProcessorMode::*;
                        match sf.processor_mode {
                            User => dropdown.set_selected(0),
                            Fiq => dropdown.set_selected(1),
                            Irq => dropdown.set_selected(2),
                            Supervisor => dropdown.set_selected(3),
                            Abort => dropdown.set_selected(4),
                            Undefined => dropdown.set_selected(5),
                            System => dropdown.set_selected(6),
                        }
                        signal_from_field.set(false);
                    }
                ));
            }

            int_obj
                .bind_property("number", &btn, "value")
                .bidirectional()
                .sync_create()
                .build();
        }
    ));

    column_view.insert_column(
        0,
        &gtk::ColumnViewColumn::builder()
            .title("Register")
            .factory(&register_factory)
            .resizable(true)
            .build(),
    );
    column_view.insert_column(
        1,
        &gtk::ColumnViewColumn::builder()
            .title("Value")
            .factory(&value_factory)
            .resizable(true)
            .expand(true)
            .build(),
    );

    scroll
}

fn spin_btn_create() -> gtk::SpinButton {
    let adjustment = gtk::Adjustment::new(0.0, i32::MIN.into(), i32::MAX.into(), 1.0, 0.0, 0.0);
    let spin_btn = gtk::SpinButton::builder()
        .adjustment(&adjustment)
        .css_classes(["font-12", "no-min-height"])
        .valign(Align::Start)
        .hexpand(true)
        .build();
    let last_child = spin_btn.last_child().unwrap();
    let second_last_child = last_child.prev_sibling().unwrap();
    last_child.set_visible(false);
    second_last_child.set_visible(false);
    return spin_btn;
}

fn toggle_btn_create(str: &'static str, tooltip: &'static str) -> gtk::ToggleButton {
    gtk::ToggleButton::builder()
        .label(str)
        .tooltip_text(tooltip)
        .hexpand(true)
        .build()
}

fn toggle_update_apsr(
    model: &ListStore,
    signal_from_toggle: Rc<Cell<bool>>,
    sf_transform: fn(komodo::StatusFlags) -> komodo::StatusFlags,
) {
    let apsr_obj = model.item(16).and_downcast::<RegObject>().unwrap();

    let mut sf = komodo::StatusFlags::from(apsr_obj.number());
    sf = sf_transform(sf);

    let new_num = komodo::update_from_flags(apsr_obj.number(), &sf);
    signal_from_toggle.set(true);
    apsr_obj.set_number(new_num);
    signal_from_toggle.set(false);
}
