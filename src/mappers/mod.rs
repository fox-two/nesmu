use log::error;

use crate::EventList;
use crate::cpu::Cpu;
use crate::joypad::{Joypad};
use crate::ppu::{PPUMemorySpace, PPU, DmaTransferSource};
use crate::{cpu::CpuMemory, memory_controller::Ram};

use crate::memory_controller::MemoryPtr;
pub mod nrom;
pub mod mmc3;

pub trait Cartridge: PPUMemorySpace + CpuMemory {
    fn get_ppu_memory(&mut self) -> &mut dyn PPUMemorySpace;
    fn start_of_frame(&mut self, event_list: &mut EventList, cyc: u64);
    fn on_event(&mut self, cpu: &mut Cpu, event_id: u32, ppu: &mut PPU);
}

impl<T: Cartridge + ?Sized> DmaTransferSource for T {
    fn read_page_for_oam(&mut self, page: u8, cpu: &mut Cpu) -> [u8; 256] {
        let start = (page as u16) << 8;

        let mut result = [0u8; 256];
        for i in 0u16..=0xff {
            result[i as usize] = self.read(MemoryPtr(start + i), cpu);
        }

        result
    }
}

pub struct SystemMemoryMapper<'a> {
    ram: &'a mut Ram,
    cartridge: &'a mut dyn Cartridge,
    ppu: &'a mut PPU,
    gamepad: &'a mut Joypad,
}

impl<'a> SystemMemoryMapper<'a> {
    pub fn new(
        ram: &'a mut Ram,
        cartridge: &'a mut dyn Cartridge,
        ppu: &'a mut PPU,
        gamepad: &'a mut Joypad
    ) -> SystemMemoryMapper<'a> {
        SystemMemoryMapper {
            ram,
            cartridge,
            ppu,
            gamepad,
        }
    }
}

impl<'a> CpuMemory for SystemMemoryMapper<'a> {
    fn read(&mut self, addr: MemoryPtr, c: &mut Cpu) -> u8 {
        if addr.0 < 0x2000 {
            return self.ram.read(addr, c);
        }

        if addr.0 >= 0x2000 && addr.0 <= 0x3fff {
            return self.ppu.context(self.cartridge.get_ppu_memory()).read(addr);
        }

        if addr.0 == 0x4016 {
            return self.gamepad.read(addr);
        }

        return self.cartridge.read(addr, c);
    }
    fn write(&mut self, addr: MemoryPtr, value: u8, c: &mut Cpu) {
        if addr.0 < 0x2000 {
            self.ram.write(addr, value, c);
            return;
        }

        if addr.0 >= 0x2000 && addr.0 <= 0x3fff {
            self.ppu
                .context(self.cartridge.get_ppu_memory())
                .write(addr, value, c);
            return;
        }

        if addr.0 == 0x4014 {
            if value < 0x20 {
                self.ppu.dma_transfer(value, self.ram, c);
            } else if value >= 40 {
                self.ppu.dma_transfer(value, self.cartridge, c);
            } else {
                error!("tried dma page of ppu registers");
            }

            return;
        }

        if addr.0 == 0x4016 {
            return self.gamepad.write(addr, value);
        }

        return self.cartridge.write(addr, value, c);
    }
}
