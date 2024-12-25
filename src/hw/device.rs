use std::time::{Duration, Instant};

use anyhow::Result;

use super::clock::Clock;

pub struct Device {
    clocks: Vec<Clock>,
    /// total time that the device had been active prior to the current time it was started 
    prior_elapsed: Duration,
    /// timestamp at which the device was most recently started, if the device is not currently paused
    current_start: Option<Instant>,
}

impl Device {
    pub fn new() -> Self {
        Self {
            clocks: Vec::new(),
            prior_elapsed: Duration::from_secs(0),
            current_start: None,
        }
    }
    
    pub fn attach(&mut self, clock: Clock) {
        self.clocks.push(clock);
    }
    
    pub fn is_paused(&self) -> bool {
        self.current_start.is_none()
    }
    
    pub fn pause(&mut self) {
        if let Some(start) = self.current_start.take() {
            self.prior_elapsed += start.elapsed();
        }
    }
    
    pub fn resume(&mut self) {
        if self.is_paused() {
            self.current_start = Some(Instant::now());
        }
    }
    
    /// Total time that the device has been active
    pub fn total_elapsed(&self) -> Duration {
        self.current_start.map(|ref i| i.elapsed()).unwrap_or_default() + self.prior_elapsed
    }
    
    pub fn run(&mut self) -> Result<()> {
        let total_elapsed = self.total_elapsed();
        for clock in &mut self.clocks {
            clock.run_to(total_elapsed)?;
        }
        Ok(())
    }
}