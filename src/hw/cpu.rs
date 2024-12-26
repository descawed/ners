use super::component::Component;

struct Registers {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    p: u8,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xFFFC,
            s: 0xFD,
            p: 0b0010_0100,
        }
    }
}

macro_rules! status {
    ($getter:ident, $setter:ident, $bit:expr) => {
        const fn $getter(&self) -> bool {
            self.status($bit)
        }

        const fn $setter(&mut self, value: bool) {
            self.set_status($bit, value);
        }
    };
}

impl Registers {
    const fn pcl(&self) -> u8 {
        (self.pc & 0xff) as u8
    }
    
    const fn set_pcl(&mut self, value: u8) {
        self.pc = (self.pc & 0xff00) | value as u16;
    }
    
    const fn pch(&self) -> u8 {
        (self.pc >> 8) as u8
    }
    
    const fn set_pch(&mut self, value: u8) {
        self.pc = (self.pc & 0x00ff) | (value as u16) << 8;
    }
    
    const fn status(&self, bit: u8) -> bool {
        self.p & (1 << bit) != 0
    }

    const fn set_status(&mut self, bit: u8, value: bool) {
        self.p = (self.p & !(1 << bit)) | (value as u8) << bit;
    }

    status!(carry, set_carry, 0);
    status!(zero, set_zero, 1);
    status!(interrupt_disable, set_interrupt_disable, 2);
    status!(decimal_mode, set_decimal_mode, 3);
    status!(break_command, set_break_command, 4);
    status!(overflow, set_overflow, 6);
    status!(negative, set_negative, 7);
}

pub struct Cpu {
    regs: Registers,
    ram: [u8; 0x800],
}