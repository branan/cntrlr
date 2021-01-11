// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Serial peripheral interface

use super::{
    super::{Mk20Dx128, Mk20Dx256},
    sim::{Gate, GatedPeripheral},
};
use crate::register::{Register, Reserved};
use bit_field::BitField;
use core::marker::PhantomData;

#[repr(C)]
struct SpiRegs {
    mcr: Register<u32>,
    _reserved0: Reserved<u32>,
    tcr: Register<u32>,
    ctar0: Register<u32>,
    ctar1: Register<u32>,
    _resreved1: [Reserved<u32>; 6],
    sr: Register<u32>,
    rser: Register<u32>,
    pushr: Register<u32>,
    popr: Register<u32>,
}

/// The handle to an SPI controller
#[allow(dead_code)]
pub struct Spi<M, I, O, C, CS, const N: usize> {
    regs: &'static mut SpiRegs,
    miso: I,
    mosi: O,
    sck: C,
    cs: CS,
    gate: Gate,
    _mcu: PhantomData<M>,
}

impl<M, const N: usize> Spi<M, (), (), (), (), N> {
    /// Enable this SPI for operation.
    pub fn enable<I, O, C, CS>(self, miso: I, mosi: O, sck: C, cs: CS) -> Spi<M, I, O, C, CS, N>
    where
        I: Miso<M, N>,
        O: Mosi<M, N>,
        C: Sck<M, N>,
        CS: Cs<M, N>,
    {
        self.regs.mcr.update(|mcr| {
            *mcr = 0;
            mcr.set_bit(31, true);
            mcr.set_bits(16..21, 0b11111);
        });

        Spi {
            regs: self.regs,
            miso,
            mosi,
            sck,
            cs,
            gate: self.gate,
            _mcu: PhantomData,
        }
    }
}

impl<M, I, O, C, CS, const N: usize> Spi<M, I, O, C, CS, N> {
    /// Read transfer attribute set 0
    pub fn read_ctar0(&self) -> (usize, usize, usize) {
        let ctar = self.regs.ctar0.read();
        (
            ctar.get_bits(16..18) as usize,
            ctar.get_bits(0..4) as usize,
            ctar.get_bits(27..31) as usize,
        )
    }

    /// Read transfer attribute set 1
    pub fn read_ctar1(&self) -> (usize, usize, usize) {
        let ctar = self.regs.ctar1.read();
        (
            ctar.get_bits(16..18) as usize,
            ctar.get_bits(0..4) as usize,
            ctar.get_bits(27..31) as usize,
        )
    }

    /// Set transfer attribute set 0
    pub fn set_ctar0(&mut self, pbr: usize, br: usize, sz: usize) {
        let pbr = match pbr {
            2 => 0,
            3 => 1,
            5 => 2,
            7 => 3,
            _ => panic!("Invalid PBR"),
        };

        let br = match br {
            2 => 0,
            4 => 1,
            6 => 2,
            8 => 3,
            16 => 4,
            32 => 5,
            64 => 6,
            128 => 7,
            256 => 8,
            512 => 9,
            1024 => 10,
            2048 => 11,
            4096 => 12,
            8192 => 13,
            16384 => 14,
            32768 => 15,
            _ => panic!("Invalid BR"),
        };

        self.regs.ctar0.update(|ctar| {
            ctar.set_bits(16..18, pbr);
            ctar.set_bits(0..4, br);
            ctar.set_bits(27..31, sz as u32 - 1);
        });
    }

    /// Set transfer attribute set 1
    pub fn set_ctar1(&mut self, pbr: usize, br: usize, sz: usize) {
        let pbr = match pbr {
            2 => 0,
            3 => 1,
            5 => 2,
            7 => 3,
            _ => panic!("Invalid PBR"),
        };

        let br = match br {
            2 => 0,
            4 => 1,
            6 => 2,
            8 => 3,
            16 => 4,
            32 => 5,
            64 => 6,
            128 => 7,
            256 => 8,
            512 => 9,
            1024 => 10,
            2048 => 11,
            4096 => 12,
            8192 => 13,
            16384 => 14,
            32768 => 15,
            _ => panic!("Invalid BR"),
        };

        self.regs.ctar1.update(|ctar| {
            ctar.set_bits(16..18, pbr);
            ctar.set_bits(0..4, br);
            ctar.set_bits(27..31, sz as u32 - 1);
        });
    }

    /// Check the transfer complete flag
    pub fn is_transfer_complete(&self) -> bool {
        self.regs.sr.read().get_bit(31)
    }

    /// Clear the transfer complete flag
    pub fn clear_transfer_complete(&mut self) {
        self.regs.sr.write(1 << 31);
    }

    /// Clear the TX ready flag
    pub fn clear_tx_ready(&mut self) {
        self.regs.sr.write(1 << 25);
    }

    /// Clear the RX ready flag
    pub fn clear_rx_ready(&mut self) {
        self.regs.sr.write(1 << 17);
    }

    /// Enable the SPI to interrupt when a transfer is completed
    pub fn enable_tc_intr(&mut self) {
        self.regs.rser.update(|rser| {
            rser.set_bit(31, true);
        });
    }

    /// Enable the SPI to interrupt when a slot is available in the TX FIFO
    pub fn enable_tx_intr(&mut self) {
        self.regs.rser.update(|rser| {
            rser.set_bit(25, true);
        });
    }

    /// Enable the SPI to interrupt when a slot is available in the RX FIFO
    pub fn enable_rx_intr(&mut self) {
        self.regs.rser.update(|rser| {
            rser.set_bit(17, true);
        });
    }

    /// How many entries are avilable in the TX FIFO
    pub fn tx_fifo_available(&self) -> usize {
        4 - self.regs.sr.read().get_bits(12..16) as usize
    }

    /// How many entries are avilable in the TX FIFO
    pub fn rx_fifo_available(&self) -> usize {
        self.regs.sr.read().get_bits(4..8) as usize
    }

    /// How many frames have been transferred since reset
    pub fn transfer_count(&self) -> u16 {
        self.regs.tcr.read().get_bits(16..32) as u16
    }
}

impl<M, I, O, C, CS, const N: usize> Spi<M, I, O, C, CS, N>
where
    O: Mosi<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
{
    /// Write one or two bytes to the SPI FIFO
    pub fn write(&mut self, ctar: usize, cs: Option<usize>, hold: bool, data: u16) -> bool {
        if self.tx_fifo_available() > 0 {
            let mut pushr = 0;
            if let Some(cs) = cs {
                pushr.set_bits(16..22, 1 << cs);
            }
            pushr.set_bit(31, hold);
            pushr.set_bits(28..31, ctar as u32);
            pushr.set_bits(0..16, data as u32);
            self.regs.pushr.write(pushr);
            true
        } else {
            false
        }
    }
}

impl<M, I, O, C, CS, const N: usize> Spi<M, I, O, C, CS, N>
where
    I: Miso<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
{
    /// Read data from the SPI FIFO
    pub fn read(&mut self) -> Option<u16> {
        if self.rx_fifo_available() > 0 {
            Some(self.regs.popr.read().get_bits(0..16) as u16)
        } else {
            None
        }
    }
}

impl<M, I, O, C, CS, const N: usize> Spi<M, I, O, C, CS, N>
where
    CS: Cs<M, N>,
{
    /// Check if our CS pins allow the given CS signal
    pub fn cs_allowed(&self, pin: usize) -> bool {
        self.cs.cs_allowed(pin)
    }
}

/// A pin which is appropriate for use as an SPI input
pub trait Miso<M, const N: usize>: Unpin {}

/// A pin which is appropriate for use as an SPI output
pub trait Mosi<M, const N: usize>: Unpin {}

/// A pin which is appropriate for use as an SPI clock
pub trait Sck<M, const N: usize>: Unpin {}

/// A pin which is appropriate for use as an SPI hardware chip select
pub trait Cs<M, const N: usize>: Unpin {
    /// Whether a given chip select is enabled by this pin
    fn cs_allowed(&self, bit: usize) -> bool;
}

impl<A, B, M, const N: usize> Cs<M, N> for (A, B)
where
    A: Cs<M, N>,
    B: Cs<M, N>,
{
    fn cs_allowed(&self, bit: usize) -> bool {
        self.0.cs_allowed(bit) || self.1.cs_allowed(bit)
    }
}

impl<A, B, C, M, const N: usize> Cs<M, N> for (A, B, C)
where
    A: Cs<M, N>,
    B: Cs<M, N>,
    C: Cs<M, N>,
{
    fn cs_allowed(&self, bit: usize) -> bool {
        self.0.cs_allowed(bit) || self.1.cs_allowed(bit) || self.2.cs_allowed(bit)
    }
}

impl<A, B, C, D, M, const N: usize> Cs<M, N> for (A, B, C, D)
where
    A: Cs<M, N>,
    B: Cs<M, N>,
    C: Cs<M, N>,
    D: Cs<M, N>,
{
    fn cs_allowed(&self, bit: usize) -> bool {
        self.0.cs_allowed(bit)
            || self.1.cs_allowed(bit)
            || self.2.cs_allowed(bit)
            || self.3.cs_allowed(bit)
    }
}

impl<A, B, C, D, E, M, const N: usize> Cs<M, N> for (A, B, C, D, E)
where
    A: Cs<M, N>,
    B: Cs<M, N>,
    C: Cs<M, N>,
    D: Cs<M, N>,
    E: Cs<M, N>,
{
    fn cs_allowed(&self, bit: usize) -> bool {
        self.0.cs_allowed(bit)
            || self.1.cs_allowed(bit)
            || self.2.cs_allowed(bit)
            || self.3.cs_allowed(bit)
            || self.4.cs_allowed(bit)
    }
}

impl<C, M, const N: usize> Cs<M, N> for Option<C>
where
    C: Cs<M, N>,
{
    fn cs_allowed(&self, bit: usize) -> bool {
        self.as_ref().map(|cs| cs.cs_allowed(bit)).unwrap_or(false)
    }
}

impl<M, const N: usize> Cs<M, N> for () {
    fn cs_allowed(&self, _bit: usize) -> bool {
        false
    }
}

unsafe impl GatedPeripheral<Mk20Dx128> for Spi<Mk20Dx128, (), (), (), (), 0> {
    const GATE: (usize, usize) = (6, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4002_C000 as *mut _),
            miso: (),
            mosi: (),
            sck: (),
            cs: (),
            gate,
            _mcu: PhantomData,
        }
    }
}
unsafe impl GatedPeripheral<Mk20Dx256> for Spi<Mk20Dx256, (), (), (), (), 0> {
    const GATE: (usize, usize) = (6, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            regs: &mut *(0x4002_C000 as *mut _),
            miso: (),
            mosi: (),
            sck: (),
            cs: (),
            gate,
            _mcu: PhantomData,
        }
    }
}
