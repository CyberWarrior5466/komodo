mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct CustomHoverProvider(ObjectSubclass<imp::CustomHoverProvider>)
        @implements sourceview5::HoverProvider;
}

impl CustomHoverProvider {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
