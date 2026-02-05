use gtk::{gio, glib, prelude::*};
use hover::provider::CustomHoverProvider;
use sourceview5::prelude::*;

mod hover;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.bilelmoussaoui.sourceview5-example"),
        Default::default(),
    );
    application.connect_activate(build_ui);

    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("SourceView5 + Rust"));
    window.set_default_size(500, 500);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let button1 = gtk::Button::builder().label("click me").build();
    container.append(&button1);

    let buffer = sourceview5::Buffer::new(None);
    buffer.set_highlight_syntax(true);
    if let Some(ref language) = sourceview5::LanguageManager::new().language("rust") {
        buffer.set_language(Some(language));
    }
    if let Some(ref scheme) = sourceview5::StyleSchemeManager::new().scheme("solarized-light") {
        buffer.set_style_scheme(Some(scheme));
    }

    let file = gio::File::for_path("buffer.rs");
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
    view.set_background_pattern(sourceview5::BackgroundPatternType::Grid);
    view.set_show_line_numbers(true);
    view.set_highlight_current_line(true);
    view.set_tab_width(4);
    view.set_hexpand(true);

    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .vscrollbar_policy(gtk::PolicyType::External)
        .build();

    scroll.set_child(Some(&view));
    container.append(&scroll);

    window.set_child(Some(&container));
    window.present();
}
