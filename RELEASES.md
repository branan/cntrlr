## 0.2.0 - Upcoming

### Changes
* Low-level uart write functions now return `bool` instead of `Option<()>`
* `io::Serial` enable API now supports optional additional settings
* Teensy serial ports can now be inverted, as an option
* Default target for Teensy 3.5 is now FPU-enabled
* `kinetis::peripheral::sim::Peripheral` renamed to `kinetis::peripheral::sim::GatedPeripheral`
* Added new `kinetis::peripheral::Peripheral` trait for ungated peripherals

### Bug Fixes
* Fix baud generation for Teensy LC serial_2
* Can no longer get handles to peripherals for MCUs other than the target
* Missing `Drop` impl on SiFive Uart
* Missing `Drop` impl on SiFive Gpio

## 0.1.0 - 2021-01-03

* Initial Release
