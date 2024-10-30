use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use std::time::{Duration, Instant};

pub struct FramerateTracker {
    last_frame: Instant,
    last_frame_timings: ConstGenericRingBuffer<Duration, 8>,
}

impl Default for FramerateTracker {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            last_frame_timings: ConstGenericRingBuffer::new(),
        }
    }
}

impl FramerateTracker {
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;
        self.last_frame_timings.push(delta);
    }

    pub fn average_framerate(&self) -> Duration {
        self.last_frame_timings
            .iter()
            .sum::<Duration>()
            .checked_div(self.last_frame_timings.len() as u32)
            .unwrap_or_default()
    }
}
