# Komodo

TODO:

- look into https://github.com/AFLplusplus/LibAFL
- test windows and mac releases using https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads
  - note: windows builds use mingw toolchain

Run unit tests

```shell
cargo test --test test
```

to test mvn, I am assuming mvn works by rotating a 8 bit value in a 32 bit word by an even amount of places

0xf << 30 = -1073741824

```
mov r0, #1
// you can shift by 0 to 31
// -1 and 32 give an error
// what happens if there is overflow
mov r0, #0xf
mov r1, r0, LSL #30
// qemu and rust give the same answer
// lets do a more simple example
```
