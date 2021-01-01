// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct Register<T>(UnsafeCell<T>);

impl<T: Copy> Register<T> {
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { core::ptr::write_volatile(self.0.get_mut(), value) }
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.read();
        f(&mut value);
        self.write(value);
    }
}

#[repr(transparent)]
pub struct Reserved<T>(T);
