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

    window.set_title(Some("SourceView5 + Rust"));
    window.set_default_size(500, 500);

    let display = gtk::gdk::Display::default().unwrap();
    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/com/my-gtk-app");

    let container = gtk::Box::new(Orientation::Vertical, 0);

    let button_container = gtk::Box::new(Orientation::Horizontal, 0);
    button_container.add_css_class("linked");
    button_container.set_halign(Align::Center);
    container.append(&button_container);

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

    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);
    // if let Some(ref language) = sourceview5::LanguageManager::new().language("") {
    //     buffer.set_language(Some(language));
    // }
    if let Some(ref scheme) = sourceview5::StyleSchemeManager::new().scheme("Adwaita-dark") {
        buffer.set_style_scheme(Some(scheme));
    }

    let file = gio::File::for_path("file.asm");
    let file = sourceview5::File::builder().location(&file).build();
    let loader = sourceview5::FileLoader::new(&buffer, &file);
    loader.load_async_with_callback(
        glib::Priority::default(),
        gio::Cancellable::NONE,
        move |current_num_bytes, total_num_bytes| {
            println!(
                "loading: {:?}",
                (current_num_bytes as f32 / total_num_bytes as f32) * 100f32
            );
        },
        |res| {
            println!("loaded: {:?}", res);
        },
    );

    let view = sourceview5::View::with_buffer(&buffer);
    view.set_monospace(true);
    view.set_show_line_numbers(true);
    view.set_highlight_current_line(true);
    view.set_tab_width(4);
    view.set_hexpand(true);

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .vscrollbar_policy(gtk::PolicyType::External)
        .build();

    scroll.set_child(Some(&view));

    let bottom_sheet = adw::BottomSheet::new();
    bottom_sheet.set_modal(false);
    bottom_sheet.set_vexpand(true);
    bottom_sheet.set_content(Some(&scroll));
    let sheet_tbv = adw::ToolbarView::new();
    bottom_sheet.set_sheet(Some(&sheet_tbv));
    let sheet_sp = adw::StatusPage::builder()
        .icon_name("go-down-symbolic")
        .build();
    sheet_tbv.set_content(Some(&sheet_sp));
    // let terminal_icon = gtk::Image::from_icon_name("dock-bottom-symbolic");
    // bottom_sheet.set_bottom_bar(Some(&terminal_icon));

    container.append(&bottom_sheet);

    let temp_button = gtk::Button::builder()
        .icon_name("dock-bottom-symbolic")
        .build();
    temp_button.connect_clicked(move |_| bottom_sheet.set_open(!bottom_sheet.is_open()));
    container.append(&temp_button);

    window.set_child(Some(&container));
    window.present();
}
