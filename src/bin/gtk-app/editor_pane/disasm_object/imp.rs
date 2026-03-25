use std::cell::Cell;
use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::DisasmObject)]
pub struct DisasmObject {
    #[property(get, set)]
    address: Cell<u32>,
    #[property(get, set)]
    encoding: Cell<u32>,
    #[property(get, set)]
    source: RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for DisasmObject {
    const NAME: &'static str = "MyGtkAppDisasmObj";
    type Type = super::DisasmObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for DisasmObject {}
