use std::sync::{Arc, Mutex};

use super::schedulable::SchedulableComponent;
use num::rational::Ratio;
use ringbuffer::AllocRingBuffer;

pub struct AudioContext {
    pub host_sample_rate: Ratio<u32>,
    pub channels: Mutex<Vec<AllocRingBuffer<i16>>>,
}

impl AudioContext {
    pub fn new(host_sample_rate: Ratio<u32>) -> Arc<Self> {
        Arc::new(Self {
            host_sample_rate,
            channels: Mutex::new(Vec::new()),
        })
    }
}

// It doesn't really make sense to have a piece of audio hardware thats not on the schedule
pub trait AudioComponent: SchedulableComponent {}
