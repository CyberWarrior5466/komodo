# Todo

- [ ] follow clippy hints
- [ ] swich from `arm-linux-gnueabi` to `arm-none-eabi`

## gtk-app

- [ ] make default buffer a simple hello world program
- [ ] make buffer persistent

  on buffer change save changes to file
  gnome text editor saves drafts to `~/.local/share/org.gnome.TextEditor/drafts`
  file location saved in gsettings, if gsettings is empty create the file

  If we have tabs then we need to be able to save multiple draft files

  For just implement drafts for a single file, and focus on implementing the debugger
  

- [ ] instead of panicking in `komodo::run_program`, display a graphical error mesage

- [ ] implement debugger
  
- [ ] fix animations bug
- [ ] add keyboard shortcuts for run action
- [ ] ask Ian if scrolling should be disabled for SpinButton
- [ ] copy over starting code from ~/Projects/project
- [ ] add zoom shortcuts

## lib

- [ ] be able to print hello world
- [ ] move `komodo::run_program` mock false case to `bin/cli`
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

---

running `objdump -d` shows

```
Disassembly of section .text:

00010078 <_start>:
   10078:	e59f1014 	ldr	r1, [pc, #20]	@ 10094 <_start+0x20>
   ...
   10090:	ef000000 	svc	0x00000000
   10094:	00011098 	.word	0x00011098  <------
```

looking for `.word 0x00011098`, by running `objdump -sj .data`

```
Contents of section .data:
 11098 48656c6c 6f20576f 726c6421 0a        Hello World!.
```

example for debugging

```
.section .data
label:
  .ascii "hi\n"
  
.section .text
_start:
  ldr r0, =label
```
