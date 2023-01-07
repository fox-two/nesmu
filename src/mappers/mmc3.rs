use crate::{cpu::{CpuMemory, Cpu}, memory_controller::MemoryPtr, ppu::{PPUMemorySpace, PPU, PPUMASK_SHOW_BACKGROUND, PPUMASK_SHOW_SPRITE}, EventList, FutureEvent, FutureEventType};

use super::Cartridge;

pub enum Mirroring {
    Horizontal,
    Vertical,
    #[allow(dead_code)]
    Hardwired //not implemented
}

pub struct Mmc3 {
    prg_ram: [u8; 8192],
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    registers: [u8; 8],
    nametables: [[u8; 0x400]; 4],
    next_register_update: u8,
    mirroring: Mirroring,
    prg_bank_mode: bool,
    chr_bank_mode: bool,
    enable_interrupt: bool,

    irq_latch: u8,
    irq_value: u8,
    reload: bool
}

pub enum MMC3MapperError {
}

impl Mmc3 {
    pub fn new(
        prg_rom: &Vec<[u8; 16384]>,
        chr_rom: &Vec<[u8; 8192]>,
        mirroring: Mirroring,
    ) -> Result<Mmc3, MMC3MapperError> {
        Ok(Mmc3 {
            prg_rom: prg_rom.concat(),
            chr_rom: chr_rom.concat(),
            mirroring: mirroring,
            registers: [0; 8],
            next_register_update: 0,
            prg_bank_mode: false,
            chr_bank_mode: false,
            nametables: [[0; 0x400]; 4],
            enable_interrupt: false,
            irq_latch: 0,
            irq_value: 0,
            prg_ram: [0; 8192],
            reload: false,
        })
    }

    fn read_prg_bank(&self, register: i32, addr: u16) -> u8 {
        match register {
            6 | 7 => {
                self.prg_rom[self.registers[register as usize] as usize * 0x2000 + ((addr as usize) & 0x1fff) ]
            },
            -2 => {
                self.prg_rom[(self.prg_rom.len() - 0x4000) + ((addr as usize) & 0x1fff) ]
            },
            -1 => {
                self.prg_rom[(self.prg_rom.len() - 0x2000) + ((addr as usize) & 0x1fff) ]
            },
            _ => {
                unreachable!("invalid banknumber");
            }
        }
    }

    fn read_big_chr_bank(&self, addr: u16) -> u8 {
        let register_number = match addr & 0xfff {
            0x000..=0x7ff => 0,
            0x800..=0xfff => 1,
            _ => unreachable!()
        };
        self.chr_rom[((self.registers[register_number] & !0x1) as usize) * 0x400 + ((addr as usize) & 0x7ff)]
    }

    fn read_small_chr_bank(&self, addr: u16) -> u8 {
        let register_number = match addr & 0xfff {
            0x000..=0x3ff => 2,
            0x400..=0x7ff => 3,
            0x800..=0xbff => 4,
            0xc00..=0xfff => 5,
            _ => unreachable!()
        };

        self.chr_rom[(self.registers[register_number] as usize) * 0x400 + ((addr as usize) & 0x3ff)]
    }

}

impl Cartridge for Mmc3 {
    fn get_ppu_memory(&mut self) -> &mut dyn PPUMemorySpace {
        self
    }
    fn on_event(&mut self, cpu: &mut Cpu, _: u32, ppu: &mut PPU) {
        if self.reload {
            self.reload = false;
            self.irq_value = self.irq_latch;
        } else if self.irq_value == 0 {
            self.irq_value = self.irq_latch;
        } else {
            if ppu.current_state.ppumask & PPUMASK_SHOW_BACKGROUND == 0 && ppu.current_state.ppumask & PPUMASK_SHOW_SPRITE == 0{
                return;
            }
            self.irq_value -= 1;

            if self.irq_value == 0 && self.enable_interrupt {
                cpu.irq_requested = true;
                self.irq_value = self.irq_latch;
            }
        }
    }
    fn start_of_frame(&mut self, event_list: &mut EventList, _: u64) {
        for i in 0..241 {
            event_list.add_event(FutureEvent { 
                cycle: ((i + 22) * 341 + 260), 
                tp: FutureEventType::Cartridge(0),
            });
        }
    }
}

impl CpuMemory for Mmc3 {
    fn read(&mut self, addr: MemoryPtr, _: &mut Cpu) -> u8 {
        match addr.0 {
            0x8000..=0x9fff => {
                if !self.prg_bank_mode {
                    self.read_prg_bank(6, addr.0)
                } else {
                    self.read_prg_bank(-2, addr.0)
                }
            },
            0xa000..=0xbfff => {
                let k = self.read_prg_bank(7, addr.0);
                k
            },
            0xc000..=0xdfff => {
                if !self.prg_bank_mode {
                    self.read_prg_bank(-2, addr.0)
                } else {
                    self.read_prg_bank(6, addr.0)
                }
            },
            0xe000..=0xffff => {
                self.read_prg_bank(-1, addr.0)
            },
            (0x6000..=0x7fff) => {
                self.prg_ram[addr.0 as usize & 0x1fff]
            },
            _ => {
                0
            }
        }
    }
    fn write(&mut self, addr: MemoryPtr, v: u8, c: &mut Cpu) {
        match (addr.0, addr.0 % 2 == 0) {
            (0x8000..=0x9fff, true) =>  {
                //bank select
                self.next_register_update = v & 0x7;
                self.prg_bank_mode = (v & 0x40) != 0;
                self.chr_bank_mode = (v & 0x80) != 0;
            },
            (0x8000..=0x9fff, false) =>  {
                self.registers[self.next_register_update as usize] = v;
            },
            (0xa000..=0xbfff, true) => {
                if let Mirroring::Hardwired = self.mirroring {
                    return;
                }
                self.mirroring = if (v & 0x1) != 0 {Mirroring::Horizontal} else {Mirroring::Vertical};
            },
            (0xa000..=0xbfff, false) => {
                //prg-ram protect; not implemented
            },
            (0xc000..=0xdfff, true) => {
                self.irq_latch = v;
            },
            (0xc000..=0xdfff, false) => {
                //irq reload
                self.reload = true;
            },
            (0xe000..=0xffff, true) => {
                //irq disable
                self.enable_interrupt = false;
                c.irq_requested = false;
            },
            (0xe000..=0xffff, false) => {
                //irq enable
                self.enable_interrupt = true;
            }
            (0x6000..=0x7fff, _) => {
                self.prg_ram[addr.0 as usize & 0x1fff] = v;
            },
            _ => {
                //debug!("unimplemented write to {:x}", addr.0);
            }
        }

    }
}

impl PPUMemorySpace for Mmc3 {
    fn ppu_read(&self, addr: u16) -> u8 {
        match (addr, self.chr_bank_mode) {
            (0x0000..=0x0fff, false) => {
                self.read_big_chr_bank(addr)
            },
            (0x0000..=0x0fff, true) => {
                self.read_small_chr_bank(addr)
            },
            (0x1000..=0x1fff, false) => {
                self.read_small_chr_bank(addr)
            },
            (0x1000..=0x1fff, true) => {
                self.read_big_chr_bank(addr)
            },
            _ => {
                match self.mirroring {
                    Mirroring::Vertical => {
                        self.nametables[((addr >> 10) & 0x1) as usize][(addr & 0x3ff) as usize]
                    },
                    Mirroring::Horizontal => {
                        self.nametables[((addr >> 11) & 0x1) as usize][(addr & 0x3ff) as usize]
                    },
                    Mirroring::Hardwired => {
                        self.nametables[((addr >> 10) & 0x3) as usize][(addr & 0x3ff) as usize]
                    }
                }
            }
        }
    }
    fn ppu_write(&mut self, addr: u16, v: u8) {
        if addr < 0x2000 {
            return;
        }
        match self.mirroring {
            Mirroring::Vertical => {
                self.nametables[((addr >> 10) & 0x1) as usize][(addr & 0x3ff) as usize] = v;
            },
            Mirroring::Horizontal => {
                self.nametables[((addr >> 11) & 0x1) as usize][(addr & 0x3ff) as usize] = v;
            },
            Mirroring::Hardwired => {
                self.nametables[((addr >> 10) & 0x3) as usize][(addr & 0x3ff) as usize] = v;
            }
        }
    }

}

