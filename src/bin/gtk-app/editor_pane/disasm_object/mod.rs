mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct DisasmObject(ObjectSubclass<imp::DisasmObject>);
}

impl DisasmObject {
    pub fn new(address: u32, encoding: u32, source: String) -> Self {
        Object::builder()
            .property("address", address)
            .property("encoding", encoding)
            .property("source", source)
            .build()
    }
}
