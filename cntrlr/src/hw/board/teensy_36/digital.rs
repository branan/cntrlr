use crate::{
    hw::mcu::kinetis::mk66fx1m0::{Port, Sim},
    sync::Once,
};

/// Port A
///
/// The global instance of PORT A, used to share port ownership among
/// different board modules.
pub fn port_a() -> &'static Port<0> {
    static PORT: Once<Port<0>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port B
///
/// The global instance of PORT B, used to share port ownership among
/// different board modules.
pub fn port_b() -> &'static Port<1> {
    static PORT: Once<Port<1>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port C
///
/// The global instance of PORT C, used to share port ownership among
/// different board modules.
pub fn port_c() -> &'static Port<2> {
    static PORT: Once<Port<2>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port D
///
/// The global instance of PORT D, used to share port ownership among
/// different board modules.
pub fn port_d() -> &'static Port<3> {
    static PORT: Once<Port<3>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port E
///
/// The global instance of PORT E, used to share port ownership among
/// different board modules.
pub fn port_e() -> &'static Port<4> {
    static PORT: Once<Port<4>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}
