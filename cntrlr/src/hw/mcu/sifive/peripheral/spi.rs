use super::super::Fe310G002;
use crate::{
    register::{Register, Reserved},
    sync::Flag,
};
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct SpiRegs {
    sckdiv: Register<u32>,
    sckmode: Register<u32>,
    _reserved0: [Reserved<u32>; 2],
    csid: Register<u32>,
    csdef: Register<u32>,
    csmode: Register<u32>,
    _reserved1: [Reserved<u32>; 3],
    delay0: Register<u32>,
    delay1: Register<u32>,
    _reserved2: [Reserved<u32>; 4],
    fmt: Register<u32>,
    _reserved3: Reserved<u32>,
    txdata: Register<u32>,
    rxdata: Register<u32>,
    txmark: Register<u32>,
    rxmark: Register<u32>,
    _reserved4: [Reserved<u32>; 2],
    fctrl: Register<u32>,
    ffmt: Register<u32>,
    _reserved5: [Reserved<u32>; 2],
    ie: Register<u32>,
    ip: Register<u32>,
}

pub struct Spi<M, T, R, const N: usize> {
    regs: &'static mut SpiRegs,
    _tx: T,
    _rx: R,
    _mcu: PhantomData<M>,
}

static LOCKS: [Flag; 3] = [Flag::new(false), Flag::new(false), Flag::new(false)];

impl Spi<Fe310G002, (), (), 0> {
    pub fn get() -> Self {
        unsafe { Self::do_get(0x1001_4000) }
    }
}

impl<M, const N: usize> Spi<M, (), (), N> {
    unsafe fn do_get(addr: usize) -> Self {
        if LOCKS[N].swap(true, Ordering::Acquire) {
            panic!("Lock contention");
        }
        Self {
            regs: &mut *(addr as *mut _),
            _tx: (),
            _rx: (),
            _mcu: PhantomData,
        }
    }

    pub fn set_divider(&mut self, div: usize) {
        assert!(div % 2 == 0 && div >= 2 && div <= 8192);
        self.regs.sckdiv.write((div / 2 - 1) as u32);
    }
}

impl<M, T, R, const N: usize> Drop for Spi<M, T, R, N> {
    fn drop(&mut self) {
        LOCKS[N].store(false, Ordering::Release);
    }
}
