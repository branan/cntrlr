// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! SPI and QSPI
//!
//! On the FE310 series, the SPI communications and QSPI memory
//! devices provide identical interfaces.

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

/// An SPI interface
pub struct Spi<M, T, R, const N: usize> {
    regs: &'static mut SpiRegs,
    _tx: T,
    _rx: R,
    _mcu: PhantomData<M>,
}

static LOCKS: [Flag; 3] = [Flag::new(false), Flag::new(false), Flag::new(false)];

#[cfg(any(doc, mcu = "fe310g002"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = "fe310g002")))]
impl super::Peripheral for Spi<super::super::Fe310G002, (), (), 0> {
    fn get() -> Option<Self> {
        unsafe {
            if LOCKS[0].swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Self {
                    regs: &mut *(0x1001_4000 as *mut _),
                    _tx: (),
                    _rx: (),
                    _mcu: PhantomData,
                })
            }
        }
    }
}

impl<M, const N: usize> Spi<M, (), (), N>
where
    Spi<M, (), (), N>: super::Peripheral,
{
    /// Get the handle to the SPI
    ///
    /// Returns 'None' if the SPI is already in use.
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl<M, const N: usize> Spi<M, (), (), N> {
    /// Set the SPI divisor
    ///
    /// The SPI is clocked at `CPU_FREQ / div`. The divisor must be an
    /// even number between 2 and 8192.
    pub fn set_divisor(&mut self, div: usize) {
        assert!(div % 2 == 0 && div >= 2 && div <= 8192);
        self.regs.sckdiv.write((div / 2 - 1) as u32);
    }
}

impl<M, T, R, const N: usize> Drop for Spi<M, T, R, N> {
    fn drop(&mut self) {
        LOCKS[N].store(false, Ordering::Release);
    }
}
