pub struct StatusFlags {
    // condition code flags
    pub negative: bool,
    pub zero: bool,
    pub carry: bool,
    pub overflow: bool,

    pub processor_mode: ProcessorMode,
}

#[derive(Clone, Copy)]
pub enum ProcessorMode {
    User = 0b10000,
    // Fiq = 0b10001,
    // Irq = 0b10010,
    // Supervisor = 0b10011,
    // Abort = 0b10111,
    // Undefined = 0b11011,
    // System = 0b11111,
}

impl StatusFlags {
    pub fn new() -> StatusFlags {
        StatusFlags {
            negative: false,
            zero: false,
            carry: false,
            overflow: false,
            processor_mode: ProcessorMode::User,
        }
    }
}

impl From<i32> for StatusFlags {
    fn from(n: i32) -> Self {
        StatusFlags {
            negative: n & (1 << 31) != 0,
            zero: n & (1 << 30) != 0,
            carry: n & (1 << 29) != 0,
            overflow: n & (1 << 28) != 0,
            processor_mode: ProcessorMode::User,
        }
    }
}

pub fn update_from_flags(apsr: i32, flags: &StatusFlags) -> i32 {
    let mut ans = apsr;
    if flags.negative {
        ans |= 1 << 31;
    }
    if flags.zero {
        ans |= 1 << 30;
    }
    if flags.carry {
        ans |= 1 << 29;
    }
    if flags.overflow {
        ans |= 1 << 28;
    }
    ans |= flags.processor_mode as i32;
    return ans;
}
