//! Oscillator

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::sync::atomic::Ordering;

#[repr(C)]
struct OscRegs {
    cr: Register<u8>,
}

/// The handle to the OSC
pub struct Osc {
    regs: &'static mut OscRegs,
}

/// An opaque token indicating that the Oscillator has been
/// configured and is not in use.
pub struct OscToken {
    _private: (),
}

static LOCK: Flag = Flag::new(false);

impl Osc {
    /// Get the handle to the OSC
    ///
    /// # Panics
    /// This method will panic if there is an outstanding handle to the
    /// OSC
    pub fn get() -> Self {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                panic!("Lock contention");
            }
            Self {
                regs: &mut *(0x4006_5000 as *mut _),
            }
        }
    }

    /// Enable the internal oscillator with the specified capacitance
    ///
    /// # Panics
    /// This method will panic if the provided capacitance cannot be
    /// represented in the MCU register.
    pub fn enable(&mut self, capacitance: u8) -> Option<OscToken> {
        if capacitance % 2 == 1 || capacitance > 30 {
            panic!("Invalid capacitance value");
        }

        let mut cr = self.regs.cr.read();
        if cr.get_bit(7) {
            None
        } else {
            cr.set_bit(7, true);
            cr.set_bit(3, capacitance.get_bit(1));
            cr.set_bit(2, capacitance.get_bit(2));
            cr.set_bit(1, capacitance.get_bit(3));
            cr.set_bit(0, capacitance.get_bit(4));

            self.regs.cr.write(cr);

            Some(OscToken { _private: () })
        }
    }
}

impl Drop for Osc {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
