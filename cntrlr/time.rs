/// Retreive the number of milliseconds the device has been running
///
/// This is a wrapping counter. On the Teensy boards, it is 32-bits
///
/// # Note
/// This count may become inaccurate if the sytem clock is modified
/// after startup.
pub fn millis() -> usize {
    MILLIS.load(Ordering::Relaxed)
}

/// Sleep this task for some number of milliseconds
///
/// This task will be slept, and awoken once the number of
/// milliseconds has pased.
pub async fn sleep_millis(duration: usize) {
    let current = millis();
    let target = current.wrapping_add(duration);
    poll_fn(|ctx| {
        if target < current {
            // We wrapped - first wait until we loop past zero
            if super::MILLIS.load(Ordering::Relaxed) >= current {
                Poll::Pending
            } else {
                if super::MILLIS.load(Ordering::Relaxed) >= target {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        } else {
            if super::MILLIS.load(Ordering::Relaxed) >= target {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    })
    .await;
}

static MILLIS: Value = Value::new(0);
static SYSTICK_WAKERS: WakerSet = WakerSet::new();

pub extern "C" fn systick_intr() {
    let millis = MILLIS.load(Ordering::Relaxed);
    let millis = millis.wrapping_add(1);
    MILLIS.store(millis, Ordering::Relaxed);
    SYSTICK_WAKERS.wake();
}
