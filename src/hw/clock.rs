use std::time::Duration;

use anyhow::{anyhow, Result};

use super::component::ComponentRef;

struct Divider {
    divider: u64,
    ticks_remaining: u64,
    component: ComponentRef,
}

impl Divider {
    const fn new(divider: u64, component: ComponentRef) -> Self {
        Self {
            divider,
            ticks_remaining: 0,
            component,
        }
    }
}

pub struct Clock {
    period: f64,
    elapsed: Duration,
    dividers: Vec<Divider>,
}

impl Clock {
    pub const fn new(period: f64) -> Self {
        Self {
            period,
            elapsed: Duration::from_secs(0),
            dividers: Vec::new(),
        }
    }

    pub fn link(&mut self, divider: u64, component: ComponentRef) {
        self.dividers.push(Divider::new(divider, component));
    }
    
    pub fn run_for(&mut self, duration: Duration) -> Result<u64> {
        let ticks = (duration.as_secs_f64() / self.period).floor() as u64;
        self.tick(ticks).map(|_| ticks)
    }
    
    pub fn run_to(&mut self, time: Duration) -> Result<u64> {
        self.run_for(time - self.elapsed)
    }
    
    pub fn tick(&mut self, count: u64) -> Result<()> {
        self.elapsed += Duration::from_secs_f64(self.period * count as f64);

        for _ in 0..count {
            for divider in &mut self.dividers {
                divider.ticks_remaining -= 1;
                if divider.ticks_remaining == 0 {
                    let cycles_run = divider.component.write().map_err(|_| anyhow!("RwLock poisoned"))?.step()?;
                    divider.ticks_remaining = divider.divider * cycles_run;
                }
            }
        }

        Ok(())
    }
    
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}