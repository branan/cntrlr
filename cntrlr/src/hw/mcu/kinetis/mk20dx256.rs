pub use super::peripheral::mcg::{Clock, Mcg, OscRange};
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::sim::{PeripheralClockSource, UsbClockSource};
pub use super::peripheral::wdog::Watchdog;

pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk20Dx256, N, M>;
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk20Dx256, N>;
pub type Sim = super::peripheral::sim::Sim<super::Mk20Dx256>;
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk20Dx256, T, R, N>;
