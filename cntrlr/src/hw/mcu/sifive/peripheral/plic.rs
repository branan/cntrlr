use crate::register::{Register, Reserved};
use bit_field::BitField;

#[repr(C)]
struct PlicRegs {
    _reserved0: Reserved<u32>,
    priority: [Register<u32>; 52],
    _reserved1: [Reserved<u32>; 971],
    pending: [Register<u32>; 2],
    _reserved2: [Reserved<u32>; 1022],
    enable: [Register<u32>; 2],
    _reserved: [Reserved<u32>; 522238],
    threshold: Register<u32>,
    claim: Register<u32>,
}

pub struct Plic {
    regs: &'static mut PlicRegs,
}

impl Plic {
    pub unsafe fn get() -> Self {
        Self {
            regs: &mut *(0x0C00_0000 as *mut _),
        }
    }

    pub fn mask_all(&mut self) {
        for reg in &mut self.regs.enable {
            reg.write(0);
        }
        for reg in &mut self.regs.priority {
            reg.write(0);
        }
    }

    pub fn enable(&mut self, intr: usize) {
        let reg = intr / 32;
        let bit = intr % 32;
        self.regs.enable[reg].update(|reg| {
            reg.set_bit(bit, true);
        });
    }

    pub fn claim(&mut self) -> u32 {
        self.regs.claim.read()
    }

    pub fn complete(&mut self, intr: u32) {
        self.regs.claim.write(intr);
    }

    pub fn set_threshold(&mut self, threshold: u32) {
        assert!(threshold < 8);
        self.regs.threshold.write(threshold);
    }

    pub fn set_priority(&mut self, intr: usize, priority: u32) {
        assert!(priority < 8);
        self.regs.priority[intr - 1].write(priority);
    }
}
