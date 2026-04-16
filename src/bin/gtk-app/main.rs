mod bottom_pane;
mod debug_panel;
mod editor_pane;
mod panes;
mod side_pane;
mod status_bar;
mod top_buttons;

use adw::prelude::*;
use async_channel::Sender;
use editor_pane::disasm_object::DisasmObject;
use gtk::glib::Propagation;
use gtk::{Orientation, gdk, gio, glib};
use komodo::{RegTuple, Registers};
use side_pane::reg_object::RegObject;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use tempfile::NamedTempFile;

enum Signal {
    Log(String),
    Halt(String, Vec<RegTuple>),
}

fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource").unwrap();

    let app = adw::Application::new(Some("com.my-gtk-app"), Default::default());

    app.connect_startup(setup_shortcuts);
    app.connect_activate(|app| {
        let css_provider = gtk::CssProvider::new();
        load_css(&css_provider);
        build_ui(&app, &css_provider)
    });
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    let quit_action = gio::ActionEntry::builder("quit")
        .activate(move |app: &adw::Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit_action]);
    app.set_accels_for_action("app.quit", &["<control>q"]);

    app.set_accels_for_action("win.action-toggle-side", &["<control>b"]);
    app.set_accels_for_action("win.action-toggle-bottom", &["<control>j"]);
    app.set_accels_for_action("win.action-zoom-in", &["<control>equal"]);
    app.set_accels_for_action("win.action-zoom-out", &["<control>minus"]);
    app.set_accels_for_action("win.action-zoom-in", &["<control>equal"]);
    app.set_accels_for_action("win.action-zoom-reset", &["<control>0"]);
}

fn load_css(css_provider: &gtk::CssProvider) {
    // let css_provider = gtk::CssProvider::new();
    css_provider.load_from_resource("/com/my-gtk-app/style.css");

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application, css_provider: &gtk::CssProvider) {
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
    let (top_buttons, run_btn) = top_buttons::create();
    let header = adw::HeaderBar::builder().title_widget(&top_buttons).build();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&container));

    let (editor_scroll, buffer) = editor_pane::create_source(&window, &css_provider);
    let center_box = gtk::CenterBox::builder()
        .hexpand(true)
        .center_widget(&editor_scroll)
        .build();

    let vec_reg_objs = Registers::new()
        .to_ui_format()
        .into_iter()
        .map(|v| RegObject::new(v.0.to_string(), v.1))
        .collect::<Vec<RegObject>>();

    let (b_pane, b_text_view) = bottom_pane::create();
    container.append(&panes::create(
        &window,
        &center_box,
        &side_pane::create(&vec_reg_objs),
        &b_pane,
    ));

    let (revealer, stop_btn) = debug_panel::create();
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

    let (sender, receiver) = async_channel::bounded::<Signal>(1);
    let stopped = Arc::new(Mutex::new(false));
    let first_execution = Arc::new(Mutex::new(true));
    let read_char = Arc::new(Mutex::new(Option::<char>::None));

    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed(glib::clone!(
        #[strong]
        read_char,
        move |_, key, _, _| {
            if let Some(unicode) = key.to_unicode() {
                *read_char.lock().unwrap() = Some(unicode);
            }
            Propagation::Stop
        }
    ));
    b_text_view.add_controller(controller);

    let action_run = gio::ActionEntry::builder("action-run")
        .activate(glib::clone!(
            #[strong]
            sender,
            #[strong]
            buffer,
            #[strong]
            vec_reg_objs,
            #[strong]
            stopped,
            #[strong]
            read_char,
            #[strong]
            first_execution,
            move |_: &adw::ApplicationWindow, _, _| {
                reset_pc(&vec_reg_objs);

                let vec_regs = vec_reg_objs
                    .iter()
                    .map(|obj| (obj.name(), obj.number()))
                    .collect::<Vec<RegTuple>>();
                let buffer_text = buffer_get_text(&buffer);

                gio::spawn_blocking(glib::clone!(
                    #[strong]
                    sender,
                    #[strong]
                    stopped,
                    #[strong]
                    read_char,
                    #[strong]
                    first_execution,
                    move || {
                        on_action_run(
                            &vec_regs,
                            buffer_text,
                            sender.clone(),
                            stopped.clone(),
                            read_char.clone(),
                            first_execution.clone(),
                        );
                        {
                            *first_execution.lock().unwrap() = false;
                        }
                    }
                ));
            }
        ))
        .build();

    stop_btn.connect_clicked(glib::clone!(
        #[strong]
        stopped,
        move |_| *stopped.lock().unwrap() = true
    ));

    glib::spawn_future_local(glib::clone!(
        #[weak]
        run_btn,
        #[strong]
        b_pane,
        async move {
            while let Ok(signal) = receiver.recv().await {
                match signal {
                    Signal::Log(s) => {
                        text_view_append(&b_text_view, s);

                        glib::idle_add_local(glib::clone!(
                            #[strong]
                            b_pane,
                            move || {
                                let vadj = b_pane.vadjustment();
                                vadj.set_value(f64::MAX);
                                b_pane.set_vadjustment(Some(&vadj));
                                return glib::ControlFlow::Break;
                            }
                        ));

                        run_btn.set_sensitive(false);
                    }
                    Signal::Halt(s, vec_regs) => {
                        text_view_append(&b_text_view, s);

                        glib::idle_add_local(glib::clone!(
                            #[strong]
                            b_pane,
                            move || {
                                let vadj = b_pane.vadjustment();
                                vadj.set_value(f64::MAX);
                                b_pane.set_vadjustment(Some(&vadj));
                                return glib::ControlFlow::Break;
                            }
                        ));

                        run_btn.set_sensitive(true);
                        apply_backend_updates(&vec_reg_objs, vec_regs);
                    }
                }
            }
        }
    ));

    window.add_action_entries([action_run]);

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
                let (_, _, instrs) = komodo::disassemble(&cs, input_path).unwrap();

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

fn on_action_run(
    vec_regs: &Vec<RegTuple>,
    buffer_text: String,
    sender: Sender<Signal>,
    stopped: Arc<Mutex<bool>>,
    read_char: Arc<Mutex<Option<char>>>,
    first_execution: Arc<Mutex<bool>>,
) {
    let msg: String;
    {
        msg = if *first_execution.lock().unwrap() {
            "> assembling".to_string()
        } else {
            "\n\n> assembling".to_string()
        }
    }
    sender.send_blocking(Signal::Log(msg)).unwrap();

    let cs = komodo::new_capstone();
    let mut input_file: NamedTempFile = NamedTempFile::new().unwrap();
    write!(input_file, "{}", buffer_text).unwrap();
    let input_path = input_file.path().as_os_str().to_owned();

    match komodo::disassemble(&cs, input_path) {
        Ok((data_section, text_section, instrs)) => {
            sender
                .send_blocking(Signal::Log("\n> executing\n".to_string()))
                .unwrap();

            let mut regs = Registers::new();
            regs.apply_ui_updates(&vec_regs);
            let read_char = || {
                loop {
                    let mut handle = read_char.lock().unwrap();
                    if let Some(c) = *handle {
                        *handle = None;
                        return c;
                    }
                }
            };
            let mut print = |str: String| sender.send_blocking(Signal::Log(str)).unwrap();
            let is_stopped = || *stopped.lock().unwrap();

            komodo::run_program(
                &cs,
                data_section,
                text_section,
                instrs,
                &mut regs,
                &read_char,
                &mut print,
                is_stopped,
            );

            let vec_regs_ret = regs.to_ui_format();

            {
                let mut stopped_handle = stopped.lock().unwrap();
                if *stopped_handle {
                    sender
                        .send_blocking(Signal::Halt("[stopped]".to_string(), vec_regs_ret))
                        .unwrap();
                    *stopped_handle = false;
                } else {
                    sender
                        .send_blocking(Signal::Halt("[exited]".to_string(), vec_regs_ret))
                        .unwrap();
                }
            }
        }
        Err(s) => {
            sender
                .send_blocking(Signal::Halt(format!("\n{}[failure]", s), vec_regs.clone()))
                .unwrap();
        }
    }
}

fn buffer_get_text(buffer: &sourceview5::Buffer) -> String {
    let bounds = buffer.bounds();
    let text = buffer.text(&bounds.0, &bounds.1, true);
    return text.to_string();
}

fn apply_backend_updates(vec_objs: &Vec<RegObject>, vec_regs: Vec<RegTuple>) {
    for (i, obj) in vec_objs.iter().enumerate() {
        obj.set_number(vec_regs[i].1);
    }
}

fn reset_pc(vec_objs: &Vec<RegObject>) {
    for obj in vec_objs {
        if obj.name() == "r15/pc" {
            obj.set_number(0);
            return;
        }
    }
}

fn text_view_append(text_view: &gtk::TextView, text: String) {
    let buffer = text_view.buffer();
    buffer.insert(&mut buffer.end_iter(), text.as_str());
}
