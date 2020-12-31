# Cargo-cntrlr

Cargo-cntrlr provides wrappers for a few common Cargo subcommands that
enable tighter integration with [Cntrlr](https://crates.io/crates/cntrlr).

## Subcommands

### cargo cntrlr build --board <BOARD> [Additional Arguments]

Builds exactly like `cargo build` except that it additionally sets up
the appropriate target and rustc configuration for the selected board.

### cargo cntrlr flash --board <BOARD> [--port <PORT>] [Additional Arguments]

As `cargo cntrlr build`, but also attempts to flash the built binary
to the board using an appropriate flashing utility. `--port` is
required for some boards.

If more than one binary is selected, they will all be built but
flashing will not take place.

### cargo cntrlr new [Additional Arguments]

Creates a new project just like `cargo new`, but modifies
`Cargo.toml`, `build.rs` and `main.rs` for a Cntrlr application.

### cargo cntrlr init [Additional Arguments]

Creates a new project just like `cargo init`, but modifies
`Cargo.toml`, `build.rs` and `main.rs` for a Cntrlr application.
