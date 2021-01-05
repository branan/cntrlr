## 0.2.0 - Upcoming

### Changes
* Low-level uart write functions now return `bool` instead of `Option<()>`
* `io::Serial` enable API now supports optional additional settings
* Teensy serial ports can now be inverted, as an option
* Default target for Teensy 3.5 is now FPU-enabled

### Bug Fixes
* Fix baud generation for Teensy LC serial_2

## 0.1.0 - 2021-01-03

* Initial Release
