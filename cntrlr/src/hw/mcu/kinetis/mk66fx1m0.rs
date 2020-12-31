pub use super::peripheral::mcg::{Clock, Mcg, OscRange}; // TODO: MK66 has a different MCG
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::wdog::Watchdog;

pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk66Fx1M0, N, M>;
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk66Fx1M0, N>;
pub type Sim = super::peripheral::sim::Sim<super::Mk66Fx1M0>;
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk66Fx1M0, T, R, N>;

const FLASH_SECURITY: u8 = 0xDE;
const FLASH_OPTIONS: u8 = 0xF9;

/// The flash configuration
///
/// This will automatically be included as the standard flash
/// configuration when a board using this MCU is selected.
#[cfg_attr(mcu = "mk66fx1m0", link_section = ".__CNTRLR_FLASH_CONFIG")]
#[cfg_attr(mcu = "mk66fx1m0", export_name = "__cntrlr_flash_configuration")]
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
