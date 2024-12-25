use anyhow::{anyhow, Result};
use binrw::BinRead;
use binrw::helpers::until_eof;
use modular_bitfield::*;
use modular_bitfield::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive as FromPrimitiveTrait;

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 1]
pub enum NametableArrangement {
    Vertical,
    Horizontal,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 2]
enum TimingMode {
    Rp2c02,
    Rp2c07,
    MultipleRegion,
    Ua6538,
}

#[derive(Debug, PartialEq)]
pub enum TvSystem {
    Ntsc,
    Pal,
    Dual,
}

#[derive(Debug, PartialEq, FromPrimitive)]
pub enum Mapper {
    NROM = 0,
    MMC1 = 1,
    UxROM = 2,
    CNROM = 3,
    MMC3_6 = 4,
    MMC5 = 5,
    AxROM = 7,
    MMC2 = 9,
    MMC4 = 10,
    ColorDreams = 11,
    CPROM = 13,
    HundredInOneContraFunction16 = 15,
    BandaiEPROM24C02 = 16,
    JalecoSS8806 = 18,
    Namco163 = 19,
    VRC4a_c = 21,
    VRC2a = 22,
    VRC6a = 24,
    VRC4b_d = 25,
    VRC6b = 26,
    BNROM = 34,
    RAMBO1 = 64,
    GxROM = 66,
    AfterBurner = 68,
    FME7 = 69,
    CamericaCodemasters = 71,
    VRC3 = 73,
    PirateMMC3 = 74,
    VRC1 = 75,
    Namco109 = 76,
    NINA03_06 = 79,
    VRC7 = 85,
    JalecoJF13 = 86,
    SenjouNoOokami = 94,
    NesEvent = 105,
    NTD8 = 113,
    TxSROM = 118,
    TQROM = 119,
    BandaiEPROM24C01 = 159,
    SUBOR166 = 166,
    SUBOR167 = 167,
    CrazyClimber = 180,
    CNROMProtection = 185,
    FS308 = 192,
    DxROM = 206,
    Namco175_340 = 210,
    Action52 = 228,
    CameraCodemastersQuattro = 232,
}

#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
struct INesBits1 {
    nametable_arrangement: NametableArrangement,
    has_persistent_memory: bool,
    has_trainer: bool,
    has_alternative_nametable_layout: bool,
    mapper_id_low: B4,
    is_vs_unisystem: bool,
    has_playchoice_data: bool,
    nes2_format_indicator: B2,
    mapper_id_high: B4,
}

impl INesBits1 {
    fn is_nes2_format(&self) -> bool {
        self.nes2_format_indicator() == 2
    }

    fn mapper_id(&self) -> u8 {
        (self.mapper_id_high() << 4) | self.mapper_id_low()
    }

    fn are_extra_flags_zero(&self) -> bool {
        !self.is_vs_unisystem() && !self.has_playchoice_data() && self.nes2_format_indicator() == 0 && self.mapper_id_high() == 0
    }

    fn has_extended_console_type(&self) -> bool {
        self.is_nes2_format() && self.is_vs_unisystem() && self.has_playchoice_data()
    }
}

#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
struct INesBits2 {
    chr_ram_size_shift: B4,
    chr_nvram_size_shift: B4,
    timing_mode: TimingMode,
    reserved1: B6,
    vs_ppu_or_extended_console_type: B4, // FIXME: implement system types
    vs_hardware_type: B4,
    num_miscellaneous_roms: B2,
    reserved2: B6,
    default_expansion_device: B6, // FIXME: implement expansion devices
    reserved3: B2,
}

impl INesBits2 {
    fn is_all_zeroes(&self) -> bool {
        self.chr_ram_size_shift() == 0 && self.chr_nvram_size_shift() == 0 && self.timing_mode() == TimingMode::Rp2c02 && self.reserved1() == 0
            && self.vs_ppu_or_extended_console_type() == 0 && self.vs_hardware_type() == 0 && self.num_miscellaneous_roms() == 0
            && self.reserved2() == 0 && self.default_expansion_device() == 0 && self.reserved3() == 0
    }
}

const fn decode_rom_size(lsb: u8, msb: u8, is_nes2: bool, nes1_shift: usize) -> usize {
    let lsb = lsb as usize;
    let msb = msb as usize;
    if is_nes2 {
        if msb == 0xf {
            let mult = (lsb & 3) * 2 + 1;
            let exp = lsb >> 2;
            mult << exp
        } else {
            lsb | (msb << 8)
        }
    } else {
        lsb << nes1_shift
    }
}

#[derive(BinRead, Debug)]
#[br(magic = b"NES\x1A")]
pub struct INes {
    prg_rom_size: u8,
    chr_rom_size: u8,
    flags6_7: INesBits1,
    prg_ram_size: u8,
    flags9: u8,
    flags10: u8,
    nes2_flags: INesBits2,
    #[br(if(flags6_7.has_trainer()))]
    trainer: Option<[u8; 512]>,
    #[br(count = decode_rom_size(prg_rom_size, flags9 & 0xf, flags6_7.is_nes2_format(), 14))]
    prg_rom: Vec<u8>,
    #[br(count = decode_rom_size(chr_rom_size, flags9 >> 4, flags6_7.is_nes2_format(), 13))]
    chr_rom: Vec<u8>,
    #[br(parse_with = until_eof)]
    miscellaneous_rom: Vec<u8>,
}

impl INes {
    fn is_nes2_format(&self) -> bool {
        self.flags6_7.is_nes2_format()
    }

    fn are_extra_flags_valid(&self) -> bool {
        self.flags6_7.is_nes2_format() || self.nes2_flags.is_all_zeroes()
    }

    pub fn trainer(&self) -> Option<&[u8; 512]> {
        self.trainer.as_ref()
    }

    pub fn set_trainer(&mut self, trainer: Option<[u8; 512]>) {
        self.trainer = trainer;
        self.flags6_7.set_has_trainer(trainer.is_some());
    }

    pub fn prg_rom(&self) -> &[u8] {
        self.prg_rom.as_slice()
    }

    pub fn chr_rom(&self) -> &[u8] {
        self.chr_rom.as_slice()
    }

    pub fn num_miscellaneous_roms(&self) -> usize {
        if self.is_nes2_format() {
            self.nes2_flags.num_miscellaneous_roms() as usize
        } else {
            0
        }
    }

    pub fn miscellaneous_rom(&self) -> &[u8] {
        self.miscellaneous_rom.as_slice()
    }

    pub fn chr_ram_size(&self) -> usize {
        if self.is_nes2_format() {
            let shift = self.nes2_flags.chr_ram_size_shift() as usize;
            if shift == 0 {
                0
            } else {
                64 << shift
            }
        } else {
            8192
        }
    }

    pub fn chr_nvram_size(&self) -> usize {
        if self.is_nes2_format() {
            let shift = self.nes2_flags.chr_nvram_size_shift() as usize;
            if shift == 0 {
                0
            } else {
                64 << shift
            }
        } else {
            0
        }
    }

    pub fn submapper_id(&self) -> Option<u8> {
        self.flags6_7.is_nes2_format().then_some(self.prg_ram_size >> 4)
    }

    pub fn mapper_id(&self) -> usize {
        let mapper_id = self.flags6_7.mapper_id() as usize;
        if !self.are_extra_flags_valid() {
            // probably junk data in the extra flags, including the high nibble of the mapper ID
            mapper_id & 0xf
        } else if self.flags6_7.is_nes2_format() {
            mapper_id | ((self.prg_ram_size as usize & 0xf) << 8)
        } else {
            mapper_id
        }
    }
    
    pub fn mapper(&self) -> Result<Mapper> {
        let mapper_id = self.mapper_id();
        Mapper::from_usize(mapper_id).ok_or_else(|| anyhow!("Unknown INES mapper ID {}", mapper_id))
    }

    pub fn tv_system(&self) -> Option<TvSystem> {
        if self.flags6_7.is_nes2_format() {
            Some(match self.nes2_flags.timing_mode() {
                TimingMode::Rp2c02 => TvSystem::Ntsc,
                TimingMode::Rp2c07 | TimingMode::Ua6538 => TvSystem::Pal,
                TimingMode::MultipleRegion => TvSystem::Dual,
            })
        } else if self.are_extra_flags_valid() {
            Some(match self.flags10 & 3 {
                1 | 3 => TvSystem::Dual,
                2 => TvSystem::Pal,
                _ => {
                    if self.flags9 & 1 == 1 {
                        TvSystem::Pal
                    } else {
                        TvSystem::Ntsc
                    }
                }
            })
        } else {
            None
        }
    }
}