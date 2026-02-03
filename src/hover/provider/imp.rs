use std::pin::Pin;

use gtk::subclass::prelude::*;
use gtk::{Label, glib};
use sourceview5::subclass::prelude::*;
use sourceview5::{HoverContext, HoverDisplay, HoverProvider};

// Object holding the state
#[derive(Default)]
pub struct CustomHoverProvider;

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for CustomHoverProvider {
    const NAME: &'static str = "CustomHoverProvider";
    type Type = super::CustomHoverProvider;
    type Interfaces = (HoverProvider,);
}

// Trait shared by all GObjects
impl ObjectImpl for CustomHoverProvider {}

impl HoverProviderImpl for CustomHoverProvider {
    fn populate_future(
        &self,
        context: &HoverContext,
        display: &HoverDisplay,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), glib::Error>> + 'static>> {
        let context = context.clone();
        let display = display.clone();
        Box::pin(async move {
            if let Some((begin, end)) = context.bounds() {
                let text = begin.slice(&end);
                let label = Label::new(Some(text.as_str()));
                display.append(&label);
            }
            Ok(())
        })
    }
}
