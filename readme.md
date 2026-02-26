# Todo

We need to do some refactors

## gtk-app

- [ ] ask Ian if scrolling should be disabled for SpinButton
- [ ] copy over starting code from ~/Projects/project
- [ ] add zoom shortcuts
- [ ] make source view style scheme light/dark system adaptive

## lib

- [ ] move `komodo::run_program` mock false case to `bin/cli`
- [ ] implement status reg updates `mov{s}`
- [ ] add reverse subtract
- [ ] lsr, asr edge cases
- [ ] `mov lr, #1` does not work, (register pseudonyms)

---

See [GUI development with Rust and GTK 4 book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)

Install libraries

```shell
sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev meson desktop-file-utils gcc gtk-update-icon-cache binutils-arm-linux-gnueabi -y
```

Also consider installing developer tools

```shell
sudo apt install libadwaita-1-examples
# run with `adwaita-1-demo`

flatpak install org.gnome.design.IconLibrary
# run with `flatpak run org.gnome.design.IconLibrary`
```
