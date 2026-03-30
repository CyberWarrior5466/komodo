use std::cell::Cell;
use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::RegObject)]
pub struct RegObject {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    number: Cell<i32>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for RegObject {
    const NAME: &'static str = "GtkAppRegObject";
    type Type = super::RegObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for RegObject {}
