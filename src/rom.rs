use std::io::Read;

use anyhow::{anyhow, Result};
use binrw::BinReaderExt;

mod ines;
use ines::*;

#[derive(Debug)]
pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Cartridge {
    pub fn from_rom<F: Read + BinReaderExt>(mut f: F) -> Result<Self> {
        // only INES format supported at the moment
        let rom: INes = f.read_le()?;
        // only NTSC currently supported
        if rom.tv_system() != Some(TvSystem::Ntsc) {
            return Err(anyhow!("Only NTSC is currently supported"));
        }
        // only NROM mapper currently supported
        if rom.mapper()? != Mapper::NROM {
            return Err(anyhow!("Only NROM mapper is currently supported"));
        }
        // no submappers currently supported
        if rom.submapper_id().unwrap_or(0) != 0 {
            return Err(anyhow!("Submappers are not currently supported"));
        }
        // trainers not supported
        if rom.trainer().is_some() {
            return Err(anyhow!("Trainers are not currently supported"));
        }
        
        Ok(Self {
            prg_rom: rom.prg_rom().to_owned(),
            chr_rom: rom.chr_rom().to_owned(),
        })
    }
}