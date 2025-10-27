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
