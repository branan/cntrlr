// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! NXP Kinetis family microcontrollers

/// Marker for peripheral instances configured for the mk20dx128
pub struct Mk20Dx128;

/// Marker for peripheral instances configured for the mk20dx256
pub struct Mk20Dx256;

/// Marker for peripheral instances configured for the mk64fx512
pub struct Mk64Fx512;

/// Marker for peripheral instances configured for the mk66fx1m0
pub struct Mk66Fx1M0;

/// Marker for peripheral instances configured for the mkl26z64
pub struct Mkl26Z64;

pub mod peripheral;

pub mod mk20dx128;
pub mod mk20dx256;
pub mod mk64fx512;
pub mod mk66fx1m0;
pub mod mkl26z64;
