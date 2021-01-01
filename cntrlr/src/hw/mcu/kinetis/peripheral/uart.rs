//! Universal Asynchronous Receiver/Transmitter

use super::{
    super::{Mk20Dx128, Mk20Dx256, Mk64Fx512, Mk66Fx1M0, Mkl26Z64},
    sim::{Gate, Peripheral},
};
use crate::register::{Register, Reserved};
use bit_field::BitField;
use core::marker::PhantomData;

#[repr(C)]
struct UartRegs {
    bdh: Register<u8>,
    bdl: Register<u8>,
    c1: Register<u8>,
    c2: Register<u8>,
    s1: Register<u8>,
    s2: Register<u8>,
    c3: Register<u8>,
    d: Register<u8>,
    ma1: Register<u8>,
    ma2: Register<u8>,
    c4: Register<u8>,
    c5: Register<u8>,
    ed: Register<u8>,
    modem: Register<u8>,
    ir: Register<u8>,
    _reserved_0: Reserved<u8>,
    pfifo: Register<u8>,
    cfifo: Register<u8>,
    sfifi: Register<u8>,
    twfifo: Register<u8>,
    tcfifo: Register<u8>,
    rwfifo: Register<u8>,
    rcfifo: Register<u8>,
}

/// The handle to a UART
pub struct Uart<M, T, R, const N: usize> {
    regs: &'static mut UartRegs,
    tx: T,
    rx: R,
    gate: Gate,
    _mcu: PhantomData<M>,
}

/// A pin which is appropriate for use as a UART transmitter.
pub trait UartTx<M, const N: usize>: Unpin {}

/// A pin which is appropriate for use as a UART reciever.
pub trait UartRx<M, const N: usize>: Unpin {}

impl<M, const N: usize> Uart<M, (), (), N> {
    /// Set the Uart divisor.
    ///
    /// The UART divider has 13 regular bits and 5 fractional bits,
    /// for a total of 18 bits. Bits above bit 17 will be ignored.
    ///
    /// The final baud rate is 1/16th the rate selected by the divider
    pub fn set_divisor(&mut self, divisor: usize) {
        self.regs.c4.update(|c4| {
            c4.set_bits(0..5, divisor.get_bits(0..5) as u8);
        });

        self.regs.bdh.update(|bdh| {
            bdh.set_bits(0..5, divisor.get_bits(13..18) as u8);
        });
        self.regs.bdl.write(divisor.get_bits(5..13) as u8);
    }
}

impl<M, R, const N: usize> Uart<M, (), R, N> {
    /// Enable this UART for transmitting.
    ///
    /// Once enabled for transmit, the baud rate and wire format
    /// cannot be changed.
    pub fn enable_tx<T>(self, tx: T) -> Uart<M, T, R, N>
    where
        T: UartTx<M, N>,
    {
        self.regs.c2.update(|c2| {
            c2.set_bit(3, true);
        });
        Uart {
            regs: self.regs,
            tx,
            rx: self.rx,
            gate: self.gate,
            _mcu: PhantomData,
        }
    }
}

impl<M, T, const N: usize> Uart<M, T, (), N> {
    /// Enable this UART for recieving.
    ///
    /// Once enabled for recieve, the baud rate and wire format
    /// cannot be changed.
    pub fn enable_rx<R>(self, rx: R) -> Uart<M, T, R, N>
    where
        R: UartRx<M, N>,
    {
        self.regs.c2.update(|c2| {
            c2.set_bit(2, true);
        });
        Uart {
            regs: self.regs,
            tx: self.tx,
            rx,
            gate: self.gate,
            _mcu: PhantomData,
        }
    }
}

impl<M, T, R: UartRx<M, N>, const N: usize> Uart<M, T, R, N> {
    /// Read a byte from the UART.
    ///
    /// Returns [`None`] if there is no data to be read.
    pub fn read_data(&mut self) -> Option<u8> {
        if self.regs.s1.read().get_bit(5) {
            Some(self.regs.d.read())
        } else {
            None
        }
    }

    /// Enable the UART to interupt when a byte is recieved.
    pub fn enable_rx_intr(&mut self) {
        self.regs.c2.update(|c2| {
            c2.set_bit(5, true);
        });
    }

    /// Disable this UART for recieving.
    pub fn disable_rx(self) -> (Uart<M, T, (), N>, R) {
        self.regs.c2.update(|c2| {
            c2.set_bit(5, false);
            c2.set_bit(2, false);
        });
        (
            Uart {
                regs: self.regs,
                tx: self.tx,
                rx: (),
                gate: self.gate,
                _mcu: PhantomData,
            },
            self.rx,
        )
    }
}

impl<M, T: UartTx<M, N>, R, const N: usize> Uart<M, T, R, N> {
    /// Send a byte to the UART.
    ///
    /// If the UART is not ready, behavior is unspecified.
    pub fn write_data(&mut self, data: u8) -> Option<()> {
        if self.regs.s1.read().get_bit(7) {
            self.regs.d.write(data);
            Some(())
        } else {
            None
        }
    }

    /// Check if the UART has transmitted all bytes in the FIFO
    pub fn is_transmit_complete(&self) -> bool {
        self.regs.s1.read().get_bit(6)
    }

    /// Enable the UART to interrupt when it is ready to transmit a byte.
    pub fn enable_tx_intr(&mut self) {
        self.regs.c2.update(|c2| {
            c2.set_bit(7, true);
        });
    }

    /// Enable the UART to interrupt when it has completed trasmitting
    /// all buffered bytes
    pub fn enable_complete_intr(&mut self) {
        self.regs.c2.update(|c2| {
            c2.set_bit(6, true);
        });
    }

    /// Disable this uart for transmitting
    pub fn disable_tx(self) -> (Uart<M, (), R, N>, T) {
        self.regs.c2.update(|c2| {
            c2.set_bit(7, false);
            c2.set_bit(3, false);
        });
        (
            Uart {
                regs: self.regs,
                tx: (),
                rx: self.rx,
                gate: self.gate,
                _mcu: PhantomData,
            },
            self.tx,
        )
    }
}

impl<M, T, R, const N: usize> Uart<M, T, R, N>
where
    Uart<M, T, R, N>: Fifo,
{
    /// Enable the transmit FIFO and set its watermark
    ///
    /// See the chip documentation for valid values.
    pub fn enable_tx_fifo(&mut self, depth: u8, watermark: u8) {
        assert!(depth <= Self::DEPTH);
        let fifo_ctl = match depth {
            1 => 0,
            4 => 1,
            8 => 2,
            16 => 3,
            32 => 4,
            64 => 5,
            128 => 6,
            _ => panic!("Invalid uart FIFO depth"),
        };
        assert!(watermark <= depth);

        self.regs.pfifo.update(|pfifo| {
            pfifo.set_bit(7, true);
            pfifo.set_bits(4..7, fifo_ctl);
        });
        self.regs.twfifo.write(watermark);
    }

    /// Enable the recieve FIFO and set its watermark
    ///
    /// See the chip documentation for valid values.
    pub fn enable_rx_fifo(&mut self, depth: u8, watermark: u8) {
        assert!(depth <= Self::DEPTH);
        let fifo_ctl = match depth {
            1 => 0,
            4 => 1,
            8 => 2,
            16 => 3,
            32 => 4,
            64 => 5,
            128 => 6,
            _ => panic!("Invalid uart FIFO depth"),
        };
        assert!(watermark <= depth);

        self.regs.pfifo.update(|pfifo| {
            pfifo.set_bit(3, true);
            pfifo.set_bits(0..3, fifo_ctl);
        });
        self.regs.rwfifo.write(watermark);
    }
}

unsafe impl Peripheral<Mk20Dx128> for Uart<Mk20Dx128, (), (), 0> {
    const GATE: (usize, usize) = (4, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Uart<Mk20Dx128, (), (), 1> {
    const GATE: (usize, usize) = (4, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Uart<Mk20Dx128, (), (), 2> {
    const GATE: (usize, usize) = (4, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_C000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Uart<Mk20Dx256, (), (), 0> {
    const GATE: (usize, usize) = (4, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Uart<Mk20Dx256, (), (), 1> {
    const GATE: (usize, usize) = (4, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Uart<Mk20Dx256, (), (), 2> {
    const GATE: (usize, usize) = (4, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_C000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 0> {
    const GATE: (usize, usize) = (4, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 1> {
    const GATE: (usize, usize) = (4, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 2> {
    const GATE: (usize, usize) = (4, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_C000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 3> {
    const GATE: (usize, usize) = (4, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_D000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 4> {
    const GATE: (usize, usize) = (1, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x400E_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Uart<Mk64Fx512, (), (), 5> {
    const GATE: (usize, usize) = (1, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x400E_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Uart<Mk66Fx1M0, (), (), 0> {
    const GATE: (usize, usize) = (4, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Uart<Mk66Fx1M0, (), (), 1> {
    const GATE: (usize, usize) = (4, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Uart<Mk66Fx1M0, (), (), 2> {
    const GATE: (usize, usize) = (4, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_C000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Uart<Mk66Fx1M0, (), (), 3> {
    const GATE: (usize, usize) = (4, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_D000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Uart<Mk66Fx1M0, (), (), 4> {
    const GATE: (usize, usize) = (1, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x400E_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Uart<Mkl26Z64, (), (), 0> {
    const GATE: (usize, usize) = (4, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_A000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Uart<Mkl26Z64, (), (), 1> {
    const GATE: (usize, usize) = (4, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_B000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Uart<Mkl26Z64, (), (), 2> {
    const GATE: (usize, usize) = (4, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4006_C000 as *mut _),
            rx: (),
            tx: (),
            gate,
            _mcu: PhantomData,
        }
    }
}

/// This is a marker trait to indicate whether a given UART supports a
/// FIFO.
pub unsafe trait Fifo {
    /// The maximum FIFO depth for this UART instance.
    const DEPTH: u8;
}

unsafe impl<T, R> Fifo for Uart<Mk20Dx128, T, R, 0> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk20Dx128, T, R, 1> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk20Dx128, T, R, 2> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk20Dx256, T, R, 0> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk20Dx256, T, R, 1> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk20Dx256, T, R, 2> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 0> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 1> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 2> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 3> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 4> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk64Fx512, T, R, 5> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk66Fx1M0, T, R, 0> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk66Fx1M0, T, R, 1> {
    const DEPTH: u8 = 8;
}
unsafe impl<T, R> Fifo for Uart<Mk66Fx1M0, T, R, 2> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk66Fx1M0, T, R, 3> {
    const DEPTH: u8 = 1;
}
unsafe impl<T, R> Fifo for Uart<Mk66Fx1M0, T, R, 4> {
    const DEPTH: u8 = 1;
}
