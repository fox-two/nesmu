use std::{fs::{File}, path::Path, io::Read};

use log::{debug};

use crate::{mappers::{nrom::BaseMapperError, Cartridge}};
use crate::mappers::nrom;
use crate::mappers::mmc3;

#[derive(Debug)]
pub struct Rom {
    pub mapper_code: u8,
    pub prg_rom: Vec<[u8; 16384]>,
    pub chr_rom: Vec<[u8; 8192]>,
    pub flags_6: u8,
    pub mirroring: Mirroring
}

#[derive(Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical
}

#[derive(Debug)]
pub enum OpenRomError {
    IOError(std::io::Error),
    InvalidMagicConstant
}

impl From<std::io::Error> for OpenRomError {
    fn from(v: std::io::Error) -> Self {
        OpenRomError::IOError(v)
    }
}

#[derive(Debug)]
pub enum GetCpuMapperError {
    UnimplementedMapper,
    MapperError
}

impl From<BaseMapperError> for GetCpuMapperError {
    fn from(_: BaseMapperError) -> Self {
        GetCpuMapperError::MapperError
    }
}

impl From<mmc3::MMC3MapperError> for GetCpuMapperError {
    fn from(_: mmc3::MMC3MapperError) -> Self {
        GetCpuMapperError::MapperError
    }
}

const FLAG6_MIRRORING: u8 = 1;

impl Rom {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Rom, OpenRomError> {
        //format description: https://www.nesdev.org/wiki/INES
        debug!("reading rom file {}", p.as_ref().to_str().unwrap());
        let mut f =  File::open(p)?;

        let mut raw_header: [u8; 16]= [0; 16];
        f.read_exact(&mut raw_header)?;

        if raw_header.as_slice()[..4] != [0x4e, 0x45, 0x53, 0x1a] {
            return Err(OpenRomError::InvalidMagicConstant);
        }
        debug!("valid magic number");

        let prg_rom_pages = raw_header[4];
        let chr_rom_pages = raw_header[5];

        debug!("{} PRG_ROM pages", raw_header[4]);
        debug!("{} CHR_ROM pages", raw_header[5]);

        let mut result = Rom{
            prg_rom: Vec::with_capacity(prg_rom_pages as usize),
            chr_rom: Vec::with_capacity(chr_rom_pages as usize),
            mapper_code: (raw_header[7] & 0xF0) | (raw_header[6] >> 4),
            flags_6: raw_header[6],
            mirroring: if raw_header[6] & FLAG6_MIRRORING != 0 {Mirroring::Vertical } else {Mirroring::Horizontal},
        };

        for _ in 0..prg_rom_pages {
            let mut buffer: [u8; 16384] = [0; 16384];
            f.read_exact(&mut buffer)?;
            result.prg_rom.push(buffer);
        }

        for _ in 0..chr_rom_pages {
            let mut buffer: [u8; 8192] = [0; 8192];
            f.read_exact(&mut buffer)?;
            result.chr_rom.push(buffer);
        }
        
        Ok(result)
    }

    pub fn get_cpu_mapper(&self) -> Result<Box<dyn Cartridge>, GetCpuMapperError> {
        match self.mapper_code {
            0 => {
                let mirroring = 
                    if self.flags_6 & FLAG6_MIRRORING != 0 {nrom::Mirroring::Vertical } else {nrom::Mirroring::Horizontal};
                Ok(Box::new(nrom::Nrom::new(&self.prg_rom, self.chr_rom[0], mirroring)?))
            }
            4 => {
                let m = match self.mirroring {
                    Mirroring::Horizontal => mmc3::Mirroring::Horizontal,
                    Mirroring::Vertical => mmc3::Mirroring::Vertical,
                };
                Ok(Box::new(mmc3::Mmc3::new(&self.prg_rom, &self.chr_rom, m)?))
            },
            _ => {
                Err(GetCpuMapperError::UnimplementedMapper)
            }
        }
        
    }
}