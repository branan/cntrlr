// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

use cntrlr_build::configure_board;
use std::{env, fs, path::PathBuf};

fn main() {
    if let Some(board) = configure_board() {
        let out_dir =
            PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` environment variable was not set"));
        let target = env::var("TARGET").expect("`TARGET` environment variable was not set");

        if board.validate_target(&target) {
            let linker_script = format!("link_scripts/{}.ld", board.mcu);
            fs::copy(&linker_script, out_dir.join("cntrlr.ld")).unwrap();

            println!("cargo:rerun-if-changed={}", linker_script);
            println!("cargo:rustc-link-search={}", out_dir.to_str().unwrap());
            return;
        }
    }
    println!(
        "cargo:warning=CNTRLR_BOARD not specified or target is incompatible with selected board"
    );
}
