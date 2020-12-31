use super::super::Fe310G002;
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct UartRegs {
    txdata: Register<u32>,
    rxdata: Register<u32>,
    txctrl: Register<u32>,
    rxctrl: Register<u32>,
    ie: Register<u32>,
    ip: Register<u32>,
    div: Register<u32>,
}

pub struct Uart<M, T, R, const N: usize> {
    regs: &'static mut UartRegs,
    tx: T,
    rx: R,
    mcu: PhantomData<M>,
}

pub trait UartTx<M, const N: usize>: Unpin {}
pub trait UartRx<M, const N: usize>: Unpin {}

static LOCKS: [Flag; 2] = [Flag::new(false), Flag::new(false)];

impl Uart<Fe310G002, (), (), 0> {
    pub fn get() -> Self {
        unsafe { Self::do_get(0x1001_3000) }
    }
}

impl Uart<Fe310G002, (), (), 1> {
    pub fn get() -> Self {
        unsafe { Self::do_get(0x1002_3000) }
    }
}

impl<M, const N: usize> Uart<M, (), (), N> {
    unsafe fn do_get(addr: usize) -> Self {
        if LOCKS[N].swap(true, Ordering::Acquire) {
            panic!("Lock contention");
        }

        Self {
            regs: &mut *(addr as *mut _),
            tx: (),
            rx: (),
            mcu: PhantomData,
        }
    }

    pub fn set_divisor(&mut self, div: usize) {
        assert!(div >= 1);
        self.regs.div.write(div as u32 - 1);
    }
}

impl<M, T, const N: usize> Uart<M, T, (), N> {
    pub fn enable_rx<R: UartRx<M, N>>(self, rx: R) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|rxctrl| {
            rxctrl.set_bit(0, true);
        });
        Uart {
            regs: self.regs,
            tx: self.tx,
            rx,
            mcu: self.mcu,
        }
    }
}

impl<M, R, const N: usize> Uart<M, (), R, N> {
    pub fn enable_tx<T: UartTx<M, N>>(self, tx: T) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|txctrl| {
            txctrl.set_bit(0, true);
        });
        Uart {
            regs: self.regs,
            tx,
            rx: self.rx,
            mcu: self.mcu,
        }
    }
}

impl<M, T, R: UartRx<M, N>, const N: usize> Uart<M, T, R, N> {
    pub fn read_data(&mut self) -> Option<u8> {
        let data = self.regs.rxdata.read();
        if data.get_bit(31) {
            None
        } else {
            Some(data.get_bits(0..8) as u8)
        }
    }

    pub fn enable_rx_intr(&mut self) {
        self.regs.ie.update(|ie| {
            ie.set_bit(1, true);
        });
    }
}

impl<M, T: UartTx<M, N>, R, const N: usize> Uart<M, T, R, N> {
    pub fn write_data(&mut self, data: u8) -> Option<()> {
        if self.regs.txdata.read().get_bit(31) {
            None
        } else {
            self.regs.txdata.write(data as u32);
            Some(())
        }
    }

    pub fn enable_tx_intr(&mut self) {
        self.regs.ie.update(|ie| {
            ie.set_bit(0, true);
        });
    }
}

impl<M, T, R, const N: usize> Uart<M, T, R, N> {
    pub fn set_watermarks(&mut self, tx: u32, rx: u32) {
        assert!(tx <= 7);
        assert!(rx <= 7);

        self.regs.txctrl.update(|txctrl| {
            txctrl.set_bits(16..19, tx);
        });
        self.regs.rxctrl.update(|rxctrl| {
            rxctrl.set_bits(16..19, rx);
        });
    }
}
