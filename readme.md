# Todo

- [ ] follow clippy hints

## gtk-app

- [ ] make buffer persistent

  on buffer change save changes to file
  gnome text editor saves drafts to `~/.local/share/org.gnome.TextEditor/drafts`
  file location saved in gsettings, if gsettings is empty create the file

  If we have tabs then we need to be able to save multiple draft files

  For just implement drafts for a single file, and focus on implementing the debugger
  

- [ ] instead of panicking in `komodo::run_program`, display a graphical error mesage

- [ ] implement debugger
  
  clicking debug should create a movable toolbar with several buttons:
  https://gist.github.com/KurtJacobson/57679e5036dc78e6a7a3ba5e0155dad1
  continue, step over, step in, step out, restart, stop 

- [ ] fix animations bug
- [ ] add keyboard shortcuts for run action
- [ ] ask Ian if scrolling should be disabled for SpinButton
- [ ] copy over starting code from ~/Projects/project
- [ ] add zoom shortcuts

## lib

- [ ] move `komodo::run_program` mock false case to `bin/cli`
- [ ] implement status reg updates `mov{s}`
- [ ] add reverse subtract `rsb`
- [ ] lsr, asr edge cases

---

Debugging using gas, qemu, gdb

```shell
sudo apt install binutils-arm-linux-gnueabi qemu-user gdb-multiarch -y
```

```fish
arm-linux-gnueabi-as -march=armv4 -D hello.s -o hello.o \
  && arm-linux-gnueabi-ld hello.o -o hello \
  && begin;
    qemu-arm -g 1234 ./hello \
      & gdb-multiarch -ex 'set architecture arm64' -ex 'file hello' -ex 'target remote localhost:1234' -ex 'layout split' -ex 'layout regs'
    end;
```

Create `hello.s` file

```asm
.global _start
_start:
  // code here
  // ...
  
  // exit syscall
  mov     r7, #1
  mov     r0, #0
  svc     #0
```

see p226 for msr instruction

---

See [GUI development with Rust and GTK 4 book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)

Install libraries

```shell
sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev meson desktop-file-utils gcc gtk-update-icon-cache binutils-arm-linux-gnueabi -y
```

Also consider installing developer tools

```shell
sudo apt install libadwaita-1-examples -y
# run with `adwaita-1-demo`

flatpak install org.gnome.design.IconLibrary
# run with `flatpak run org.gnome.design.IconLibrary`
```

---

Running old komodo

```shell
kmd -e
```
