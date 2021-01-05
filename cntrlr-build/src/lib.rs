// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Build support for Cntrlr boards

#![deny(missing_docs)]

use std::{env, str::FromStr};

/// Set up the rust build environment for the selected board.
///
/// Based on the `CNTRLR_BOARD` environment variable, this function
/// will set the `board=` and `mcu=` rust configurations, which can be
/// used to customize your application behavior based on the target
/// environment. It also returns information about the selected board.
pub fn configure_board() -> Option<Board> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CNTRLR_BOARD");
    env::var("CNTRLR_BOARD")
        .ok()
        .and_then(|board| Board::from_str(&board).ok())
        .map(|board| {
            println!("cargo:rustc-cfg=board=\"{}\"", board.name);
            println!("cargo:rustc-cfg=mcu=\"{}\"", board.mcu);
            board
        })
}

/// The utility used to flash a board
pub enum Flash {
    /// This board is flashed with `avrdude`, using the specified programmer (-c)
    AvrDude(&'static str),

    /// This board is flashed with `teensy-loader-cli`, using the MCU specified in the board config
    TeensyLoader,

    /// This board is flashed with `openocd`, using the specified configuration file
    OpenOcd(&'static str),
}

/// Information about a target board
pub struct Board {
    /// The name of the board, as used in Cntrlr's source code.
    pub name: &'static str,

    /// The name of the mcu used in the board, as used in Cntrlr's
    /// source code.
    pub mcu: &'static str,

    /// The set of Rust targets for which this board is compatible.
    pub targets: Vec<&'static str>,

    /// The preferred method for flashing an image to the board
    pub flash: Flash,
}

impl FromStr for Board {
    type Err = ();
    fn from_str(board: &str) -> Result<Self, ()> {
        match normalize_board(board).as_str() {
            "teensy30" | "teensy3" => Ok(Self {
                name: "teensy_30",
                mcu: "mk20dx128",
                targets: vec![
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensy31" | "teensy32" => Ok(Self {
                name: "teensy_32",
                mcu: "mk20dx256",
                targets: vec![
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensy35" => Ok(Self {
                name: "teensy_35",
                mcu: "mk64fx512",
                targets: vec![
                    "thumbv7em-none-eabihf",
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensy36" => Ok(Self {
                name: "teensy_36",
                mcu: "mk66fx1m0",
                targets: vec![
                    "thumbv7em-none-eabihf",
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensy40" | "teensy4" => Ok(Self {
                name: "teensy_40",
                mcu: "imxrt1062",
                targets: vec![
                    "thumbv7em-none-eabihf",
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensy41" => Ok(Self {
                name: "teensy_41",
                mcu: "imxrt1062",
                targets: vec![
                    "thumbv7em-none-eabihf",
                    "thumbv7em-none-eabi",
                    "thumbv7m-none-eabi",
                    "thumbv6m-none-eabi",
                ],
                flash: Flash::TeensyLoader,
            }),
            "teensylc" => Ok(Self {
                name: "teensy_lc",
                mcu: "mkl26z64",
                targets: vec!["thumbv6m-none-eabi"],
                flash: Flash::TeensyLoader,
            }),
            "arduinouno" => Ok(Self {
                name: "arduino_uno",
                mcu: "atmega328p",
                targets: vec!["avr-none-atmega328"],
                flash: Flash::AvrDude("arduino"),
            }),
            "redv" => Ok(Self {
                name: "red_v",
                mcu: "fe310g002",
                targets: vec![
                    "riscv32imac-unknown-none-elf",
                    "riscv32imc-unknown-none-elf",
                    "riscv32i-unknown-none-elf",
                ],
                flash: Flash::OpenOcd("board/sifive-hifive1-revb.cfg"),
            }),
            _ => Err(()),
        }
    }
}

impl Board {
    /// Check if the specified target is valid for this board.
    pub fn validate_target(&self, target: &str) -> bool {
        self.targets.iter().any(|item| item == &target)
    }
}

fn normalize_board(board: &str) -> String {
    board
        .to_lowercase()
        .chars()
        .filter(|&c| c != '_' && c != '-' && c != '.')
        .collect()
}
