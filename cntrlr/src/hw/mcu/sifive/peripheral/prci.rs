use super::super::Fe310G002;
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct PrciRegs {
    hfrosccfg: Register<u32>,
    hfxosccfg: Register<u32>,
    pllcfg: Register<u32>,
    plloutdiv: Register<u32>,
}

pub struct Prci<M> {
    regs: &'static mut PrciRegs,
    _mcu: PhantomData<M>,
}

static LOCK: Flag = Flag::new(false);

impl Prci<Fe310G002> {
    pub fn get() -> Self {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                panic!("Lock contention");
            }
            Self {
                regs: &mut *(0x1000_8000 as *mut _),
                _mcu: PhantomData,
            }
        }
    }
}

impl<M> Prci<M> {
    // TODO: This should be broken down into a nice state machine like
    // the Kinetis MCG
    pub fn use_pll(&mut self, r: u32, f: u32, q: u32, div: u32) {
        assert!(r >= 1 && r <= 4);
        let r = r - 1;
        assert!(f >= 2 && f <= 128 && f % 2 == 0);
        let f = f / 2 - 2;
        let q = match q {
            2 => 0b01,
            4 => 0b10,
            8 => 0b11,
            _ => panic!("Invalid value for Q"),
        };
        assert!(div == 1 || (div % 2) == 0);
        if self.regs.pllcfg.read().get_bit(16) {
            // Make sure the ring oscillator is enabled and switch to it
            self.regs.hfrosccfg.update(|hfrosccfg| {
                hfrosccfg.set_bit(30, true);
            });
            while !self.regs.hfrosccfg.read().get_bit(31) {}
            self.regs.pllcfg.update(|pllcfg| {
                pllcfg.set_bit(16, false);
            });
        }
        self.regs.pllcfg.update(|pll| {
            pll.set_bit(18, true); // bypass PLL
        });

        // Set final PLL dividers
        if div == 1 {
            self.regs.plloutdiv.write(1 << 8);
        } else {
            self.regs.plloutdiv.write(div / 2 - 1);
        }

        // Enable crystal oscillator
        self.regs.hfxosccfg.update(|hfxosccfg| {
            hfxosccfg.set_bit(30, true);
        });
        while !self.regs.hfxosccfg.read().get_bit(31) {}

        // Set PLL dividers and wait for lock
        self.regs.pllcfg.update(|pllcfg| {
            pllcfg.set_bits(0..2, r);
            pllcfg.set_bits(4..10, f);
            pllcfg.set_bits(10..12, q);
            pllcfg.set_bit(17, true);
            pllcfg.set_bit(18, false);
        });
        while !self.regs.pllcfg.read().get_bit(31) {}

        // Switch to PLL as main clock source
        self.regs.pllcfg.update(|pllcfg| {
            pllcfg.set_bit(16, true);
        });
    }
}

impl<M> Drop for Prci<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
