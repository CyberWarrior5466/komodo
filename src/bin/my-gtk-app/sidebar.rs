use gtk::Align;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create() -> gtk::ScrolledWindow {
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
            .set_child(Some(&spin_button_create()))
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

    value_factory.connect_bind(glib::clone!(
        #[strong]
        vec,
        move |_, list_item_obj| {
            let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();

            let num_obj = list_item
                .item()
                .and_downcast::<glib::BoxedAnyObject>()
                .unwrap();

            let (reg_i, _, reg_value) = num_obj.borrow::<Reg>().clone();

            let button = list_item.child().and_downcast::<gtk::SpinButton>().unwrap();
            button.set_value(reg_value.into());

            button.connect_value_changed(glib::clone!(
                #[strong]
                vec,
                move |btn| {
                    vec.borrow_mut()[reg_i].borrow_mut::<Reg>().2 = btn.value_as_int();
                }
            ));
        }
    ));

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

fn spin_button_create() -> gtk::SpinButton {
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
