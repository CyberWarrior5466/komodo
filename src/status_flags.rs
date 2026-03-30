#[derive(Debug, Clone)]
pub struct StatusFlags {
    // condition code flags
    pub negative: bool,
    pub zero: bool,
    pub carry: bool,
    pub overflow: bool,

    pub processor_mode: ProcessorMode,
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessorMode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl From<i32> for ProcessorMode {
    fn from(value: i32) -> Self {
        match value {
            0b10000 => ProcessorMode::User,
            0b10001 => ProcessorMode::Fiq,
            0b10010 => ProcessorMode::Irq,
            0b10011 => ProcessorMode::Supervisor,
            0b10111 => ProcessorMode::Abort,
            0b11011 => ProcessorMode::Undefined,
            0b11111 => ProcessorMode::System,
            _ => ProcessorMode::User,
        }
    }
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
            processor_mode: ProcessorMode::from(n & 0b11111),
        }
    }
}

pub fn update_from_flags(apsr: i32, flags: &StatusFlags) -> i32 {
    let mut ans = apsr;

    if flags.negative {
        ans |= 1 << 31;
    } else {
        ans &= -1 ^ (1 << 31)
    }

    if flags.zero {
        ans |= 1 << 30;
    } else {
        ans &= -1 ^ (1 << 30)
    }

    if flags.carry {
        ans |= 1 << 29;
    } else {
        ans &= -1 ^ (1 << 29)
    }

    if flags.overflow {
        ans |= 1 << 28;
    } else {
        ans &= -1 ^ (1 << 28)
    }

    ans &= -1 ^ 0b11111;
    ans |= flags.processor_mode as i32;
    ans
}
