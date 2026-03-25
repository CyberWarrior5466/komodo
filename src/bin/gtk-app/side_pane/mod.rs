pub mod reg_object;

use gtk::Align;
use gtk::gio;
use gtk::prelude::*;

use reg_object::RegObject;

pub fn create(vec: &Vec<RegObject>) -> gtk::ScrolledWindow {
    let model = gio::ListStore::new::<RegObject>();
    model.extend_from_slice(vec);

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

    value_factory.connect_bind(move |_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
        let int_obj = list_item.item().and_downcast::<RegObject>().unwrap();
        let btn = list_item.child().and_downcast::<gtk::SpinButton>().unwrap();

        int_obj
            .bind_property("number", &btn, "value")
            .bidirectional()
            .sync_create()
            .build();
    });

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

    gtk::ScrolledWindow::builder().child(&column_view).build()
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
