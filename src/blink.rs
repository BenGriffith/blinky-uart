use core::fmt::Write as _;
use embassy_rp::uart::{Blocking, Uart};
use embassy_time::Duration;

pub struct BlinkStats {
    count: usize,
    on_time: Duration,
    off_time: Duration,
}

impl BlinkStats {
    pub fn new(count: usize, on_time: Duration, off_time: Duration) -> Self {
        Self {
            count,
            on_time,
            off_time,
        }
    }
    pub fn current_blink(&self, uart: &mut Uart<'_, Blocking>) {
        let mut msg: heapless::String<64> = heapless::String::new();
        core::write!(
            &mut msg,
            "Count: {}, On: {}, Off: {}\r\n",
            self.count,
            self.on_time.as_millis(),
            self.off_time.as_millis()
        )
        .unwrap();
        uart.blocking_write(msg.as_bytes()).unwrap();
    }
}
