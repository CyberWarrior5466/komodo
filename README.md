# Komodo

TODO:


- look into https://github.com/AFLplusplus/LibAFL
- test windows and mac releases using https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads
  - note: windows builds use mingw toolchain

Run unit tests

```shell
cargo test --test test
```

---

how does the `cmp` and `tst` instruction work?

The `cmp` instruction takes in a register, followed by a shifter operand
if the second operand is a register it can be shifted

`cmp r0, r1, LSL #1`

{ vector_index: None, subtracted: false, shift: Lsl(1), op_type: Reg(RegId(67)), access: Some(ReadOnly) }

it sets alu_out to `Rn - shifter_operand`

The `tst` instruction is like `cmp`, as it also takes in Rn and shifter_operand but

it sets alu_out to `Rn and shifter_operand`

The `teq` instruction sets alu_out to `Ru xor shifter_operand`

Many other instructions can also take an `S` for example `adds` is `add` with an `s` meaning
the condition flags are updated

What is the format of the status flags in armv4? (p30)

1. Condition flags (**N**egative, **Z**ero, **C**arry, o**V**erflow)
2. IRQ interrupt mask (I)
3. FIQ interrupt mask (F)
4. Processor mode (last 5 bits of cpsr)

The last 3 are probably not needed since they are hardware specific

what is the capstone value of cpsr?

ARM_REG_CPSR = 3,

Implement `mrs` and `msr` instruction since they are simple?

```
mrs rd, CPSR/SPSR
```

what do I do about spsr since I don't use it,

just add it anyway incase you need it in the future

what is the operand format

`apsr` is application program status register

forget the `msr` instruction, do it later,


```
mov r0, #0
cmp r0, #1
mrs r1, cpsr
// what is cpsr
// 0x80000010
// nzcv
// 0x8 = 1000
// last 4 bits
```

function that takes in `StatusFlags` and converts it to

---

TODO
- **ADD cpsr and spsr to debug fmt**
- add reverse subtract

`cmp` instruction:

- [x] 0, 0
  6
  zero
  carry

- [x] 0, 1
  4
  negative

- [x] 1, 0x80000000
  9
  negative
  overflow

- [x] 0x80000000, 1
  3
  carry
  overflow

- [x] 1, -2
  0

- [x] 2, 1
  2
  carry

`cmn` instruction:

- [x] 0, 0
  4

- [x] 0, 1
  0

- [x] 0, -1
  8

- [x] 0x7fffffff, 1
  9

- [x] 0x80000000, -1
  3
