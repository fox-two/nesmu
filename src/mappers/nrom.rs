use crate::{cpu::{CpuMemory, Cpu}, memory_controller::MemoryPtr, ppu::{PPUMemorySpace, PPU}, EventList};

use super::Cartridge;

pub enum Mirroring {
    Horizontal,
    Vertical,
}

pub struct Nrom {
    prg_rom: [u8; 32768],
    chr_rom: [u8; 8192],
    nametables: [[u8; 0x400]; 2],
    mirroring: Mirroring,
}

pub enum BaseMapperError {
    NoPrgRomPages,
    TooManyPrgRomPages,
}

impl Nrom {
    pub fn new(
        prg_rom: &Vec<[u8; 16384]>,
        chr_rom: [u8; 8192],
        mirror: Mirroring,
    ) -> Result<Nrom, BaseMapperError> {
        if prg_rom.len() == 0 {
            return Err(BaseMapperError::NoPrgRomPages);
        }
        if prg_rom.len() > 2 {
            return Err(BaseMapperError::TooManyPrgRomPages);
        }

        let mut result_prg_rom: [u8; 32768] = [0; 32768];

        for i in 0..2 {
            result_prg_rom[(i * 16384)..((i + 1) * 16384)]
                .copy_from_slice(&prg_rom[i % prg_rom.len()]);
        }

        Ok(Nrom {
            prg_rom: result_prg_rom,
            chr_rom: chr_rom,
            nametables: [[0; 0x400]; 2],
            mirroring: mirror,
        })
    }
}

impl Cartridge for Nrom {
    fn get_ppu_memory(&mut self) -> &mut dyn PPUMemorySpace {
        self
    }
    fn on_event(&mut self, _: &mut Cpu, _: u32, _: &mut PPU) {
        
    }
    fn start_of_frame(&mut self, _: &mut EventList, _: u64) {
        
    }
}

impl CpuMemory for Nrom {
    fn read(&mut self, addr: MemoryPtr, _: &mut Cpu) -> u8 {
        self.prg_rom[(addr.0 & 0x7fff) as usize]
    }
    fn write(&mut self, _: MemoryPtr, _: u8, _: &mut Cpu) {
    }
}

impl PPUMemorySpace for Nrom {
    fn ppu_read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            return self.chr_rom[addr as usize];
        }

        match self.mirroring {
            Mirroring::Vertical => {
                self.nametables[((addr >> 10) & 0x1) as usize][(addr & 0x3ff) as usize]
            }
            Mirroring::Horizontal => {
                self.nametables[((addr >> 11) & 0x1) as usize][(addr & 0x3ff) as usize]
            }
        }
    }
    fn ppu_write(&mut self, addr: u16, v: u8) {
        if addr < 0x2000 || addr >= 0x3000 {
            return;
        }
        match self.mirroring {
            Mirroring::Vertical => {
                self.nametables[((addr >> 10) & 0x1) as usize][(addr & 0x3ff) as usize] = v;
            }
            Mirroring::Horizontal => {
                self.nametables[((addr >> 11) & 0x1) as usize][(addr & 0x3ff) as usize] = v;
            }
        }
    }
}
