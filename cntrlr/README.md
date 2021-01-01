# Cntrlr - Simple, asynchronous embedded
Cntrlr is an all-in-one embedded platform for writing simple
asynchronous applications on top of common hobbyist development
boards.

## Examples

### Hello World to a serial port

```rust
#![no_std]
#![no_main]

use cntrlr::prelude::*;
use core::future::pending;

#[entry]
async fn main() -> ! {
    serial_1().enable(9600).unwrap();
    writeln!(serial_1(), \"Hello, World\").await.unwrap();

    // Hang forever once we've sent our message
    pending().await
}
```

### Blinking LED

```rust
#![no_std]
#![no_main]

use cntrlr::prelude::*;

#[entry]
async fn main() -> ! {
    pin_mode(13, PinMode::Output);
    loop {
        digital_write(13, true);
        sleep(500).await;
        digital_wrrite(13, false);
        sleep(500).await;
    }
}
```

