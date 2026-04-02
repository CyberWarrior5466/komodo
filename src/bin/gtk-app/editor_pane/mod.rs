pub mod disasm_object;
use gtk::gdk::pango;
use gtk::{gio, glib, prelude::*};
use sourceview5::prelude::*;

use disasm_object::DisasmObject;

pub fn create_source(
    window: &adw::ApplicationWindow,
    css_provider: &gtk::CssProvider,
) -> (gtk::ScrolledWindow, sourceview5::Buffer) {
    let buffer = sourceview5::Buffer::builder()
        .style_scheme(&get_style_scheme())
        .build();

    let adw_style = adw::StyleManager::default();
    adw_style.connect_dark_notify(glib::clone!(
        #[strong]
        buffer,
        move |_| buffer.set_style_scheme(Some(&get_style_scheme()))
    ));

    let view = sourceview5::View::builder()
        .monospace(true)
        .show_line_numbers(true)
        .highlight_current_line(true)
        .buffer(&buffer)
        .build();

    let action_zoom_in = gio::ActionEntry::builder("action-zoom-in")
        .activate(glib::clone!(
            #[strong]
            view,
            #[strong]
            css_provider,
            move |_: &adw::ApplicationWindow, _, _| {
                let font_size = get_zoom(&view.pango_context());
                set_zoom(
                    &css_provider,
                    f64::max(3.0 / pango::SCALE as f64, font_size * pango::SCALE_LARGE),
                );
            }
        ))
        .build();

    let action_zoom_out = gio::ActionEntry::builder("action-zoom-out")
        .activate(glib::clone!(
            #[strong]
            view,
            #[strong]
            css_provider,
            move |_: &adw::ApplicationWindow, _, _| {
                let font_size = get_zoom(&view.pango_context());
                set_zoom(
                    &css_provider,
                    f64::max(3.0 / pango::SCALE as f64, font_size * pango::SCALE_SMALL),
                );
            }
        ))
        .build();

    let action_zoom_reset = gio::ActionEntry::builder("action-zoom-reset")
        .activate(glib::clone!(
            #[strong]
            css_provider,
            move |_: &adw::ApplicationWindow, _, _| {
                set_zoom(&css_provider, 15019.0 / pango::SCALE as f64);
            }
        ))
        .build();
    window.add_action_entries([action_zoom_in, action_zoom_out, action_zoom_reset]);

    let scroll = gtk::ScrolledWindow::builder()
        .vscrollbar_policy(gtk::PolicyType::External)
        .vexpand(true)
        .hexpand(true)
        .child(&view)
        .build();

    (scroll, buffer)
}

pub fn create_disasm(model: &gio::ListStore) -> gtk::ScrolledWindow {
    let column_view = gtk::ColumnView::new(Some(gtk::NoSelection::new(Some(model.clone()))));

    let address_factory = gtk::SignalListItemFactory::new();
    let encoding_factory = gtk::SignalListItemFactory::new();
    let source_factory = gtk::SignalListItemFactory::new();

    address_factory.connect_setup(|_, list_item_obj| {
        list_item_obj
            .downcast_ref::<gtk::ColumnViewCell>()
            .unwrap()
            .set_child(Some(&gtk::Label::new(None)));
    });
    encoding_factory.connect_setup(|_, list_item_obj| {
        list_item_obj
            .downcast_ref::<gtk::ColumnViewCell>()
            .unwrap()
            .set_child(Some(&gtk::Label::new(None)));
    });
    source_factory.connect_setup(|_, list_item_obj| {
        list_item_obj
            .downcast_ref::<gtk::ColumnViewCell>()
            .unwrap()
            .set_child(Some(
                &gtk::Label::builder().halign(gtk::Align::Start).build(),
            ));
    });

    address_factory.connect_bind(|_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
        let disasm_obj = list_item.item().and_downcast::<DisasmObject>().unwrap();
        let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
        label.set_text(&disasm_obj.address().to_string());
    });
    encoding_factory.connect_bind(|_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
        let disasm_obj = list_item.item().and_downcast::<DisasmObject>().unwrap();
        let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
        label.set_text(&format!("{:08x}", disasm_obj.encoding()));
    });
    source_factory.connect_bind(|_, list_item_obj| {
        let list_item = list_item_obj.downcast_ref::<gtk::ColumnViewCell>().unwrap();
        let disasm_obj = list_item.item().and_downcast::<DisasmObject>().unwrap();
        let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
        label.set_text(&disasm_obj.source());
    });

    column_view.insert_column(
        0,
        &gtk::ColumnViewColumn::builder()
            .title("Address")
            .factory(&address_factory)
            .build(),
    );
    column_view.insert_column(
        1,
        &gtk::ColumnViewColumn::builder()
            .title("Encoding")
            .factory(&encoding_factory)
            .build(),
    );
    column_view.insert_column(
        2,
        &gtk::ColumnViewColumn::builder()
            .title("Source")
            .factory(&source_factory)
            .build(),
    );

    gtk::ScrolledWindow::builder()
        .child(&column_view)
        .hexpand(true)
        .build()
}

fn get_style_scheme() -> sourceview5::StyleScheme {
    if adw::StyleManager::default().is_dark() {
        sourceview5::StyleSchemeManager::new()
            .scheme("Adwaita-dark")
            .unwrap()
    } else {
        sourceview5::StyleSchemeManager::new()
            .scheme("Adwaita")
            .unwrap()
    }
}

fn get_zoom(context: &pango::Context) -> f64 {
    context.font_description().unwrap().size() as f64 / pango::SCALE as f64
}

fn set_zoom(css_provider: &gtk::CssProvider, value: f64) {
    let font_css = format!(
        "textview {{
        font-size: {}px;
        }}",
        value
    );
    css_provider.load_from_string(font_css.as_str());
}
