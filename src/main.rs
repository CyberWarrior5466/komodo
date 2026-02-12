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

    app.run()
}
fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_string(
        // Orange 2
        "
        .orange { color: #ffa348; }
        ",
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

    let container = gtk::Box::new(Orientation::Vertical, 0);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(400)
        .build();

    container.append(&create_button_container());
    container.append(&create_panes(&window));
    container.append(&create_gutter());

    // window.add_action_entries([action_toggle_side, action_toggle_bottom]);

    window.set_child(Some(&container));
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

fn create_panes(window: &gtk::ApplicationWindow) -> gtk::Paned {
    let bottom = gtk::Label::new(Some("Bottom"));
    let bottom_pane = gtk::Paned::builder()
        .orientation(Orientation::Vertical)
        .start_child(&gtk::Label::new(Some("Main")))
        .end_child(&bottom)
        .build();

    let sidebar = gtk::Label::new(Some("Sidebar"));
    let side_pane = gtk::Paned::builder()
        .vexpand(true)
        .start_child(&sidebar)
        .end_child(&bottom_pane)
        .build();

    let action_toggle_side = gio::ActionEntry::builder("toggle-side")
        .activate(move |_: &gtk::ApplicationWindow, _, _| {
            sidebar.set_visible(!sidebar.is_visible())
        })
        .build();

    let action_toggle_bottom = gio::ActionEntry::builder("toggle-bottom")
        .activate(move |_: &gtk::ApplicationWindow, _, _| {
            bottom.set_visible(!bottom.is_visible());
        })
        .build();

    window.add_action_entries([action_toggle_side, action_toggle_bottom]);

    return side_pane;
}

fn create_gutter() -> gtk::HeaderBar {
    let header = gtk::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("")))
        .show_title_buttons(false)
        .build();

    let toggle_left = gtk::Button::builder()
        .icon_name("dock-left-symbolic")
        .halign(Align::Start)
        .build();
    let toggle_bottom = gtk::Button::builder()
        .icon_name("dock-bottom-symbolic")
        .halign(Align::End)
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
