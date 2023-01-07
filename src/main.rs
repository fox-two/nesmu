extern crate minifb;

mod cpu;
mod ines_rom_file;
mod joypad;
mod mappers;
mod memory_controller;
use joypad::Button;
use mappers::Cartridge;
use ppu::{PPUDrawingContext};

use std::{
    cmp::Ordering, env,
};

use cpu::Cpu;
use env_logger::{Builder, Target};
use memory_controller::Ram;
use minifb::{Key, Window, WindowOptions};

use crate::{
    cpu::{CpuContext},
    joypad::Joypad,
    mappers::SystemMemoryMapper,
    ppu::PPU,
};

const WIDTH: usize = 256;
const HEIGHT: usize = 240;

mod ppu;

const KEY_CONFIG: [(Key, Button); 8] = [
    (Key::A, Button::A),
    (Key::S, Button::B),
    (Key::Backspace, Button::SELECT),
    (Key::Enter, Button::START),
    (Key::Up, Button::UP),
    (Key::Down, Button::DOWN),
    (Key::Left, Button::LEFT),
    (Key::Right, Button::RIGHT),
];

fn main() {
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() <= 1 {
        println!("Usage: {:} <rom filename>", args[0]);
        println!("mmc3 is partially supported");
        return;
    }

    let x = ines_rom_file::Rom::new(args[1].clone()).unwrap();

    let k = x.get_cpu_mapper().unwrap();

    let mut console = Nes::new(k);

    let mut window = Window::new(
        "Nes Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for v in KEY_CONFIG.iter() {
            console.gamepad.set_state(v.1, window.is_key_down(v.0));
        }

        let frame =console.frame();
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&frame, WIDTH, HEIGHT).unwrap();
    }
}

fn convert_components_to_pixel(components: (u8, u8, u8)) -> u32 {
    return (u32::from(components.0) << 16)
        | (u32::from(components.1) << 8)
        | (u32::from(components.2));
}

const PALLETE: [(u8, u8, u8); 64] = [
    (124, 124, 124),
    (0, 0, 252),
    (0, 0, 188),
    (68, 40, 188),
    (148, 0, 132),
    (168, 0, 32),
    (168, 16, 0),
    (136, 20, 0),
    (80, 48, 0),
    (0, 120, 0),
    (0, 104, 0),
    (0, 88, 0),
    (0, 64, 88),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (188, 188, 188),
    (0, 120, 248),
    (0, 88, 248),
    (104, 68, 252),
    (216, 0, 204),
    (228, 0, 88),
    (248, 56, 0),
    (228, 92, 16),
    (172, 124, 0),
    (0, 184, 0),
    (0, 168, 0),
    (0, 168, 68),
    (0, 136, 136),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (248, 248, 248),
    (60, 188, 252),
    (104, 136, 252),
    (152, 120, 248),
    (248, 120, 248),
    (248, 88, 152),
    (248, 120, 88),
    (252, 160, 68),
    (248, 184, 0),
    (184, 248, 24),
    (88, 216, 84),
    (88, 248, 152),
    (0, 232, 216),
    (120, 120, 120),
    (0, 0, 0),
    (0, 0, 0),
    (252, 252, 252),
    (164, 228, 252),
    (184, 184, 248),
    (216, 184, 248),
    (248, 184, 248),
    (248, 164, 192),
    (240, 208, 176),
    (252, 224, 168),
    (248, 216, 120),
    (216, 248, 120),
    (184, 248, 184),
    (184, 248, 216),
    (0, 252, 252),
    (248, 216, 248),
    (0, 0, 0),
    (0, 0, 0),
];

struct Nes {
    pub ram: Ram,
    pub cpu: Cpu,
    pub ppu: PPU,
    pub gamepad: Joypad,
    pub cartridge: Box<dyn Cartridge>,
    pub events: EventList,
    pub framebuffer_nes: [u8; 240*256]
}

impl Nes {
    fn new(game: Box<dyn Cartridge>) -> Nes {

        let mut ret = Nes {
            ram: Ram::new(),
            cpu: Cpu::new(),
            ppu: PPU::new(),
            gamepad: Joypad::new(),
            cartridge: game,
            events: EventList::new(),
            framebuffer_nes: [0; 240*256]
        };

        ret.cpu_context().reset();
        ret
    }

    fn ppu_drawing_context(&mut self) -> PPUDrawingContext {
        self.ppu.drawing_context(self.cartridge.as_mut().get_ppu_memory(), &mut self.events, &mut self.framebuffer_nes)
    }

    fn cpu_context<'a>(&'a mut self) -> CpuContext<'a, SystemMemoryMapper> {
        self.cpu.context(SystemMemoryMapper::new(&mut self.ram, self.cartridge.as_mut(), &mut self.ppu, &mut self.gamepad))
    }

    fn frame(&mut self) -> [u32; 240*256] {
        let start_of_frame_cycle = self.cpu.cycle_count;
        self.ppu_drawing_context().set_vblank_flag(start_of_frame_cycle);
        self.cartridge.start_of_frame(&mut self.events, self.cpu.cycle_count);

        if self.ppu.nmi_active() {
            self.cpu_context().nmi();
        }

        //execute until the end of frame
        while 3 * (self.cpu.cycle_count - start_of_frame_cycle) < 89342 {
            while let Some(x) = self.events.pop_next_event(3 * (self.cpu.cycle_count - start_of_frame_cycle)) {
                match x.tp {
                    FutureEventType::PPU(event_id) => {
                        let cyc = self.cpu.cycle_count;
                        self.ppu_drawing_context().handle_event(event_id, cyc);
                    },
                    FutureEventType::Cartridge(event_id) => {
                        self.cartridge.on_event(&mut self.cpu, event_id, &mut self.ppu);
                    },
                };
            };

            self.cpu_context().execute_next_instruction().unwrap();
        }

        self.events.clear();
        let mut buffer: [u32; 240*256] = [0; 240*256];

        for (i, pixel) in self.framebuffer_nes.iter().enumerate() {
            buffer[i] = convert_components_to_pixel(PALLETE[(*pixel & 0x3f) as usize]);
        }

        buffer
    }
    

}


pub struct EventList {
    next_events: Vec<FutureEvent>
}

impl EventList {
    fn new() -> EventList {
        EventList { next_events: Vec::new() }
    }
    fn add_event(&mut self, e: FutureEvent) {
        self.next_events.push(e);

        self.next_events.sort_by(|b, a| {
            if a.cycle < b.cycle {
                Ordering::Less
            } else if a.cycle > b.cycle {
                Ordering::Greater
            } else {
                Ordering::Equal
            }

        });
    }

    fn pop_next_event(&mut self, cyc: u64) -> Option<FutureEvent> {
        let item = match self.next_events.last() {
            Some(x) => x,
            _ => {
                return None;
            }
        };

        if item.cycle <= cyc {
            return self.next_events.pop();
        }

        None
    }

    fn clear(&mut self) {
        self.next_events.clear();
    }
}



struct FutureEvent {
    cycle: u64,
    tp: FutureEventType
}


enum FutureEventType {
    PPU(u32),
    Cartridge(u32),
}

#[cfg(test)]
mod nestest;
