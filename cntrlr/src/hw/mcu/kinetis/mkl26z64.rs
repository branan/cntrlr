pub use super::peripheral::mcg::{Clock, Mcg, OscRange};
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::wdog::Watchdog;

pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mkl26Z64, N, M>;
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mkl26Z64, N>;
pub type Sim = super::peripheral::sim::Sim<super::Mkl26Z64>;
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mkl26Z64, T, R, N>;
