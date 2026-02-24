mod bottom_bar;
mod editor;
mod panes;
mod sidebar;
mod top_buttons;

use adw::prelude::*;
use gtk::{Orientation, gdk, gio, glib};

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
    let provider = gtk::CssProvider::new();
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
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application) {
    gio::resources_register_include!("compiled.gresource").unwrap();

    let display = gdk::Display::default().unwrap();
    let icon_theme = gtk::IconTheme::for_display(&display);
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
        .title_widget(&top_buttons::create())
        .build();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&container));

    let editor = editor::create();
    let sidebar = sidebar::create();
    container.append(&panes::create(&window, &editor, &sidebar));
    toolbar.add_bottom_bar(&bottom_bar::create());

    window.set_content(Some(&toolbar));
    window.present();
}
