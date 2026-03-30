mod bottom_pane;
mod debug_panel;
mod editor_pane;
mod panes;
mod side_pane;
mod status_bar;
mod top_buttons;

use adw::prelude::*;
use editor_pane::disasm_object::DisasmObject;
use gtk::{Orientation, gdk, gio, glib};
use komodo::{RegTuple, Registers};
use side_pane::reg_object::RegObject;
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
        .default_height(694)
        .build();

    let container = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .vexpand(true)
        .build();

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder()
        .title_widget(&top_buttons::create())
        .build();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&container));

    let (editor_scroll, buffer) = editor_pane::create_source();
    let center_box = gtk::CenterBox::builder()
        .hexpand(true)
        .center_widget(&editor_scroll)
        .build();

    let vec_reg_objs = Registers::new()
        .to_ui_format()
        .into_iter()
        .map(|v| RegObject::new(v.0.to_string(), v.1))
        .collect::<Vec<RegObject>>();

    container.append(&panes::create(
        &window,
        &center_box,
        &side_pane::create(&vec_reg_objs),
        &bottom_pane::create(),
    ));

    let revealer = debug_panel::create();
    container.append(&revealer);

    let action_debug = gio::ActionEntry::builder("action-debug")
        .activate(glib::clone!(
            #[strong]
            revealer,
            move |_: &adw::ApplicationWindow, _, _| {
                revealer.set_reveal_child(!revealer.reveals_child());
            }
        ))
        .build();
    window.add_action_entries([action_debug]);

    toolbar.add_bottom_bar(&status_bar::create());

    let action_run = gio::ActionEntry::builder("action-run")
        .activate(glib::clone!(
            #[strong]
            buffer,
            move |_: &adw::ApplicationWindow, _, _| {
                reset_pc(vec_reg_objs.clone());

                let vec_regs = vec_reg_objs
                    .iter()
                    .map(|obj| (obj.name(), obj.number()))
                    .collect::<Vec<RegTuple>>();

                let cs = komodo::new_capstone();
                let mut input_file: NamedTempFile = NamedTempFile::new().unwrap();
                write!(input_file, "{}", buffer_get_text(&buffer)).unwrap();
                let input_path = input_file.path().as_os_str().to_owned();
                let print_dism = |str| glib::g_printerr!("{str}");
                let (data_section, text_section, instrs) =
                    komodo::disassemble(&cs, input_path, print_dism);

                let mut regs = Registers::new();
                regs.apply_ui_updates(&vec_regs);
                let mut print = |str: String| glib::g_print!("{}", str);
                komodo::run_program(
                    &cs,
                    data_section,
                    text_section,
                    &mut regs,
                    instrs,
                    &mut print,
                );

                let vec_regs_return = regs.to_ui_format();
                apply_backend_updates(vec_reg_objs.clone(), vec_regs_return);
            }
        ))
        .build();
    window.add_action_entries([action_run]);

    // ---

    let action_view_source = gio::ActionEntry::builder("action-view-source")
        .activate(glib::clone!(
            #[strong]
            center_box,
            move |_: &adw::ApplicationWindow, _, _| {
                center_box.set_center_widget(Some(&editor_scroll));
            }
        ))
        .build();

    let vec_disasm_objs: Vec<DisasmObject> = Vec::new();
    let model = gio::ListStore::new::<DisasmObject>();
    model.extend_from_slice(&vec_disasm_objs);

    let label = editor_pane::create_disasm(&model);
    let action_view_disasm = gio::ActionEntry::builder("action-view-disasm")
        .activate(glib::clone!(
            #[strong]
            center_box,
            #[strong]
            buffer,
            #[strong]
            model,
            move |_: &adw::ApplicationWindow, _, _| {
                let cs = komodo::new_capstone();
                let mut input_file: NamedTempFile = NamedTempFile::new().unwrap();
                write!(input_file, "{}", buffer_get_text(&buffer)).unwrap();
                let input_path = input_file.path().as_os_str().to_owned();
                let print_dism = |str| glib::g_printerr!("{str}");
                let (_, _, instrs) = komodo::disassemble(&cs, input_path, print_dism);

                model.remove_all();
                for i in instrs.iter() {
                    let mut bytes: Vec<u8> = Vec::new();
                    for &b in i.bytes().iter().rev() {
                        bytes.push(b);
                    }
                    let encoding = u32::from_be_bytes(bytes.clone().try_into().unwrap());

                    model.append(&DisasmObject::new(
                        i.address() as u32,
                        encoding,
                        i.mnemonic().unwrap().to_string() + " " + i.op_str().unwrap(),
                    ));
                }

                center_box.set_center_widget(Some(&label));
            }
        ))
        .build();
    window.add_action_entries([action_view_source, action_view_disasm]);

    window.set_title(Some("Komodo, University of Nottingham"));
    window.set_content(Some(&toolbar));
    window.present();
}

fn buffer_get_text(buffer: &sourceview5::Buffer) -> String {
    let bounds = buffer.bounds();
    let text = buffer.text(&bounds.0, &bounds.1, true);
    return text.to_string();
}

fn apply_backend_updates(vec_objs: Vec<RegObject>, vec_regs: Vec<RegTuple>) {
    for (i, obj) in vec_objs.iter().enumerate() {
        obj.set_number(vec_regs[i].1);
    }
}

fn reset_pc(vec_objs: Vec<RegObject>) {
    for obj in vec_objs {
        if obj.name() == "r15/pc" {
            obj.set_number(0);
            return;
        }
    }
}
