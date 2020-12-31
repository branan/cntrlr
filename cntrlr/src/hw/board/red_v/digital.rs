use crate::{hw::mcu::sifive::fe310g002::Gpio, sync::Once};

pub fn gpio() -> &'static Gpio<0> {
    static PORT: Once<Gpio<0>> = Once::new();
    PORT.get_or_init(|| Gpio::get())
}
