# Todo

We need to do some refactors

## gtk-app

- [ ] merge readmes
- [ ] convert registers into a list of tuples
- [ ] refactor lib.rs to take in soru

- [ ] ask Ian if scrolling should be disabled for SpinButton

- [ ] copy over starting code from ~/Projects/project
- [ ] add zoom shortcuts
- [ ] make source view style scheme light/dark system adaptive

## lib

- [ ] add reverse subtract
- [ ] lsr, asr edge cases
- [ ] `mov lr, #1` does not work, (register pseudonyms)

---

See [GUI development with Rust and GTK 4](https://gtk-rs.org/gtk4-rs/stable/latest/book/) book

Install libraries

```shell
sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev meson desktop-file-utils gcc gtk-update-icon-cache binutils-arm-linux-gnueabi
```

Also consider install developer tools

```shell
sudo apt install libadwaita-1-examples
# run with `adwaita-1-demo`

flatpak install org.gnome.design.IconLibrary
# run with `flatpak run org.gnome.design.IconLibrary`
```
