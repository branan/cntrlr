# Cntrlr - Simple, asynchronous embedded

[![Crates.io](https://img.shields.io/crates/d/cntrlr)](https://crates.io/crates/cntrlr)
[![Docs.rs](https://docs.rs/cntrlr/badge.svg)](https://docs.rs/cntrlr)

Cntrlr is an all-in-one embedded platform for writing simple
asynchronous applications on top of common hobbyist development
boards.

## Examples

### Hello World to a serial port

```
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

```
#![no_std]
#![no_main]

use cntrlr::prelude::*;

#[entry]
async fn main() -> ! {
    loop {
        digital_write(13, true);
        sleep(500).await;
        digital_wrrite(13, false);
        sleep(500).await;
    }
}
```

