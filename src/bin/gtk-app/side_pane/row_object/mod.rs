mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct RowObject(ObjectSubclass<imp::RowObject>);
}

impl RowObject {
    pub fn new(name: String, number: i32) -> Self {
        Object::builder()
            .property("name", name)
            .property("number", number)
            .build()
    }
}
