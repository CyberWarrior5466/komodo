use gtk::Align;
use gtk::CssProvider;
use gtk::IconTheme;
use gtk::Orientation;
use gtk::gdk::Display;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use sourceview5::prelude::*;

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

    let window = gtk::ApplicationWindow::new(app);

    let display = gtk::gdk::Display::default().unwrap();
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/com/my-gtk-app");

    let container = gtk::Box::new(Orientation::Vertical, 0);

    let button_container = gtk::Box::new(Orientation::Horizontal, 0);
    button_container.add_css_class("linked");
    button_container.set_halign(Align::Center);

    let button1 = gtk::Button::builder()
        .icon_name("execute-from-symbolic")
        .build();

    let button2_box = gtk::Box::new(Orientation::Horizontal, 8);
    let button2_icon = gtk::Image::from_icon_name("bug-symbolic");
    let button2_label = gtk::Label::new(Some("Debug"));
    button2_icon.add_css_class("orange");
    button2_box.append(&button2_icon);
    button2_box.append(&button2_label);
    let button2 = gtk::Button::new();
    button2.set_child(Some(&button2_box));

    button_container.append(&button1);
    button_container.append(&button2);

    container.append(&button_container);

    let side_main_pane = gtk::Paned::new(Orientation::Horizontal);
    side_main_pane.set_vexpand(true);
    let side_pane = gtk::Label::new(Some("Sidebar Content"));
    side_main_pane.set_start_child(Some(&side_pane));

    let main_bottom_pane = gtk::Paned::new(Orientation::Vertical);
    side_main_pane.set_end_child(Some(&main_bottom_pane));

    let content = gtk::Label::new(Some("Main Content Area"));
    let bottom_pane = gtk::Label::new(Some("Bottom pane"));
    main_bottom_pane.set_start_child(Some(&content));
    main_bottom_pane.set_end_child(Some(&bottom_pane));

    container.append(&side_main_pane);

    // let window = gtk::ApplicationWindow::builder()
    //     .application(app)
    //     .default_width(600)
    //     .default_height(400)
    //     .build();

    window.set_child(Some(&container));
    window.present();
}

// fn create_button_container() -> gtk::Box {
//     let button_container = gtk::Box::new(Orientation::Horizontal, 0);
//     button_container.add_css_class("linked");
//     button_container.set_halign(Align::Center);

//     let button1 = gtk::Button::builder()
//         .icon_name("execute-from-symbolic")
//         .build();

//     let button2_box = gtk::Box::new(Orientation::Horizontal, 8);
//     let button2_icon = gtk::Image::from_icon_name("bug-symbolic");
//     let button2_label = gtk::Label::new(Some("Debug"));
//     button2_icon.add_css_class("orange");
//     button2_box.append(&button2_icon);
//     button2_box.append(&button2_label);
//     let button2 = gtk::Button::new();
//     button2.set_child(Some(&button2_box));

//     button_container.append(&button1);
//     button_container.append(&button2);

//     return button_container;
// }
