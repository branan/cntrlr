pub mod io;

const FLASH_SECURITY: u8 = 0xDE;
const FLASH_OPTIONS: u8 = 0xF9;

/// The flash configuration
///
/// This will automatically be included as the standard flash
/// configuration when a board using this MCU is selected.
#[cfg_attr(
    any(
        board = "teensy_30",
        board = "teensy_32",
        board = "teensy_35",
        board = "teensy_36",
        board = "teensy_lc"
    ),
    link_section = ".__CNTRLR_FLASH_CONFIG"
)]
#[cfg_attr(
    any(
        board = "teensy_30",
        board = "teensy_32",
        board = "teensy_35",
        board = "teensy_36",
        board = "teensy_lc"
    ),
    export_name = "__cntrl_flash_configuration"
)]
pub static FLASH_CONFIGURATION: [u8; 16] = [
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    FLASH_SECURITY,
    FLASH_OPTIONS,
    0xFF,
    0xFF,
];
