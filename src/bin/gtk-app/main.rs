mod bottom_pane;
mod editor_pane;
mod panes;
mod side_pane;
mod status_bar;
mod top_buttons;

use adw::prelude::*;
use gtk::{Orientation, gdk, gio, glib};
use komodo::{RegTuple, Registers};
use side_pane::row_object::RowObject;
use std::io::Write;
use tempfile::NamedTempFile;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource").unwrap();

    let app = adw::Application::new(Some("com.my-gtk-app"), Default::default());

    app.connect_startup(|app| {
        load_css();
        setup_shortcuts(app);
    });
    app.connect_activate(build_ui);

    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    let quit_action = gio::ActionEntry::builder("quit")
        .activate(move |app: &adw::Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit_action]);
    app.set_accels_for_action("app.quit", &["<control>q"]);

    app.set_accels_for_action("win.toggle-side", &["<control>b"]);
    app.set_accels_for_action("win.toggle-bottom", &["<control>j"]);
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/com/my-gtk-app/style.css");

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application) {
    let display = gdk::Display::default().unwrap();
    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path("/com/my-gtk-app");

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(850)
        .default_height(650)
        .build();

    let container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .build();

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder()
        .title_widget(&top_buttons::create())
        .build();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&container));

    let (editor_scroll, buffer) = editor_pane::create(&window);

    let vec_objs = Registers::new()
        .to_ui_format()
        .into_iter()
        .map(|v| RowObject::new(v.0.to_string(), v.1))
        .collect::<Vec<RowObject>>();

    container.append(&panes::create(
        &window,
        &editor_scroll,
        &side_pane::create(vec_objs.clone()),
        &bottom_pane::create(),
    ));
    toolbar.add_bottom_bar(&status_bar::create());

    let action_run = gio::ActionEntry::builder("action-run")
        .activate(move |_: &adw::ApplicationWindow, _, _| {
            // set r15/pc of vec_objs to 0

            let vec_regs = vec_objs
                .iter()
                .map(|obj| (obj.name(), obj.number()))
                .collect::<Vec<RegTuple>>();

            let mut input_file = NamedTempFile::new().unwrap();
            write!(input_file, "{}", buffer_get_text(&buffer)).unwrap();

            let mut registers = Registers::new();
            registers.apply_ui_updates(&vec_regs);
            komodo::run_program(&mut input_file, &mut registers, true);

            let vec_regs_return = registers.to_ui_format();
            apply_backend_updates(&vec_objs, vec_regs_return);
        })
        .build();
    window.add_action_entries([action_run]);

    window.set_content(Some(&toolbar));
    window.present();
}

fn buffer_get_text(buffer: &sourceview5::Buffer) -> String {
    let bounds = buffer.bounds();
    let text = buffer.text(&bounds.0, &bounds.1, true);
    return text.to_string();
}

fn apply_backend_updates(vec_objs: &Vec<RowObject>, vec_regs: Vec<RegTuple>) {
    let vec_objs_clone = vec_objs.clone();
    for (i, obj) in vec_objs_clone.iter().enumerate() {
        obj.set_number(vec_regs[i].1);
    }
}
