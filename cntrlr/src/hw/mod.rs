// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Hardware support
//!
//! Generally, applications should not have to access this module. It
//! is provided to allow specialized applications to directly access
//! board- or mcu-specific functionality.

pub mod board;
pub mod mcu;
