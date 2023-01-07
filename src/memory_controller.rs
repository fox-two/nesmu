use std::{ops};

use crate::{cpu::Cpu, ppu::DmaTransferSource};

use super::cpu::CpuMemory;

pub struct Ram {
    ram: [u8; 2048]
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct MemoryPtr(pub u16);


impl ops::Add<u16> for MemoryPtr {
    type Output = MemoryPtr;
    fn add(self, rhs: u16) -> MemoryPtr {
        return MemoryPtr(self.0.wrapping_add(rhs))
    }
}

impl ops::AddAssign<u16> for MemoryPtr {
    fn add_assign(&mut self, other: u16) {
        self.0 = self.0.wrapping_add(other);
    }
}

impl Ram {
    pub fn new() -> Ram {
        Ram { ram: [0; 2048] }
    }
    #[allow(dead_code)]
    pub fn set_ram_state(&mut self, state: [u8; 2048]) {
        self.ram = state;
    }
    #[allow(dead_code)]
    pub fn dump_ram(&self) -> [u8; 2048] {
        self.ram
    }
}

impl CpuMemory for Ram {
    fn read(&mut self, addr: MemoryPtr, _: &mut Cpu) -> u8 {
        return self.ram[(addr.0 & 0x7ff) as usize];
    }
    fn write(&mut self, addr: MemoryPtr, value: u8, _: &mut Cpu){
        self.ram[(addr.0 & 0x7ff) as usize] = value;
    }
}

impl DmaTransferSource for Ram {
    fn read_page_for_oam(&mut self, page: u8, _: &mut Cpu) -> [u8; 256] {
        let start = (page as usize) << 8;
        self.ram.as_slice()[start..(start+0x100)].try_into().unwrap()
    }
}
