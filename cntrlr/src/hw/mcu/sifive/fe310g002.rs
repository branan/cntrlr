pub use super::{
    peripheral::gpio::{UartRx, UartTx},
    peripheral::plic::Plic,
    Fe310G002,
};

pub type Gpio<const N: usize> = super::peripheral::gpio::Gpio<Fe310G002, N>;
pub type Pin<'a, const N: usize, const P: usize> =
    super::peripheral::gpio::Pin<'a, Fe310G002, N, P>;
pub type Prci = super::peripheral::prci::Prci<Fe310G002>;
pub type Spi<T, R, const N: usize> = super::peripheral::spi::Spi<Fe310G002, T, R, N>;
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<Fe310G002, T, R, N>;
