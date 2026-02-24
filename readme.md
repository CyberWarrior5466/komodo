# Todo

- [ ] ask Ian if scrolling should be disabled for SpinButton
- [ ] make spin button update registers

- [ ] copy over starting code from ~/Projects/project
- [ ] add zoom shortcuts
- [ ] make source view style scheme light/dark system adaptive

---

Animations

```rust
let button_container = create_button_container();
let button = gtk::Button::with_label("grow");
let anim = adw::TimedAnimation::new(
    &button,
    20.0,
    100.0,
    250,
    adw::PropertyAnimationTarget::new(&button, "width_request"),
);
button_container.append(&button);
button.connect_clicked(move |_| anim.play());
```

---

See [GUI development with Rust and GTK 4](https://gtk-rs.org/gtk4-rs/stable/latest/book/) book

Install gtk libraries

```shell
sudo apt install libgtk-4-dev libadwaita-1-dev meson desktop-file-utils gcc gtk-update-icon-cache
```

Install sourceview lib

```shell
sudo apt install libgtksourceview-5-dev
```

Install libadwaita demo

```shell
sudo apt install libadwaita-1-examples
adwaita-1-demo
```

Install icon library

```shell
flatpak install org.gnome.design.IconLibrary
```

also may need to install libxml

```shell
sudo apt install libxml2-utils
```
