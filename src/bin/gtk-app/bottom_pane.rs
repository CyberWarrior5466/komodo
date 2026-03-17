use gtk::prelude::BoxExt;

pub fn create() -> gtk::Box {
    let box_ = gtk::Box::builder().css_classes(["darker"]).build();
    box_.append(
        &gtk::Label::builder()
            .label("")
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Start)
            .build(),
    );
    box_
}
