use anyhow::Result;

use crate::rom::Cartridge;
use super::clock::Clock;
use super::device::Device;

pub struct Nes {
    device: Device,
    cartridge: Option<Cartridge>,
}

impl Nes {
    pub fn new() -> Self {
        // TODO: add support for PAL
        let master_clock = Clock::new(11.0 / 236250000.0);
        
        let mut device = Device::new();
        device.attach(master_clock);

        Self {
            device,
            cartridge: None,
        }
    }
    
    pub fn is_cartridge_loaded(&self) -> bool {
        self.cartridge.is_some()
    }
    
    pub fn is_paused(&self) -> bool {
        self.device.is_paused()
    }
    
    pub fn is_running(&self) -> bool {
        self.is_cartridge_loaded() && !self.is_paused()
    }
    
    pub fn pause(&mut self) {
        self.device.pause();
    }
    
    pub fn resume(&mut self) {
        self.device.resume();
    }
    
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }
    
    pub fn eject_cartridge(&mut self) -> Option<Cartridge> {
        self.cartridge.take()
    }
    
    pub fn run(&mut self) -> Result<()> {
        self.device.run()
    }
}