# Todo

## gtk-app

- [ ] cache disassembly stage for faster executions
  
- [ ] make default buffer a simple hello world program
- [ ] make buffer persistent

  on buffer change save changes to file
  gnome text editor saves drafts to `~/.local/share/org.gnome.TextEditor/drafts`
  file location saved in gsettings, if gsettings is empty create the file

  If we have tabs then we need to be able to save multiple draft files

  For just implement drafts for a single file, and focus on implementing the debugger

- [ ] implement debugger
  
- [ ] fix animations bug
- [ ] add keyboard shortcuts for run action
- [ ] disable scrolling for SpinButton
- [ ] copy over starting code from ~/Projects/project

## lib

- [ ] **blt bug**
- [ ] swich from `arm-linux-gnueabi` to `arm-none-eabi`
- [ ] ldr instruction other cases
- [ ] add reverse subtract `rsb`

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
      & gdb-multiarch -ex 'set architecture arm' -ex 'file hello' -ex 'target remote localhost:1234' -ex 'layout split' -ex 'layout regs'
    end;
```

Debugging libc linked asm file

```fish
arm-linux-gnueabi-gcc -march=armv4 -ggdb hello_libc.s -o hello_libc \
  && begin;
    qemu-arm -g 1234 ./hello_libc \
      & gdb-multiarch -ex 'set architecture arm' -ex 'file hello_libc' -ex 'target remote localhost:1234' -ex 'layout split' -ex 'layout regs'
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
