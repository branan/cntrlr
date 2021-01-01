// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Board support

#[cfg(any(doc, board = "red_v"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "red_v")))]
pub mod red_v;

#[cfg(any(doc, board = "teensy_30"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "teensy_30")))]
pub mod teensy_30;

#[cfg(any(doc, board = "teensy_32"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "teensy_32")))]
pub mod teensy_32;

#[cfg(any(doc, board = "teensy_35"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "teensy_35")))]
pub mod teensy_35;

#[cfg(any(doc, board = "teensy_36"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "teensy_36")))]
pub mod teensy_36;

#[cfg(any(doc, board = "teensy_lc"))]
#[cfg_attr(feature = "doc-cfg", doc(cfg(board = "teensy_lc")))]
pub mod teensy_lc;

#[cfg(any(
    doc,
    board = "teensy_30",
    board = "teensy_32",
    board = "teensy_35",
    board = "teensy_36",
    board = "teensy_lc"
))]
#[cfg_attr(
    feature = "doc-cfg",
    doc(cfg(
        board = "teensy_30",
        board = "teensy_32",
        board = "teensy_35",
        board = "teensy_36",
        board = "teensy_lc"
    ))
)]
pub mod teensy_common;
