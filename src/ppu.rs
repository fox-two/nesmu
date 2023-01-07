use log::debug;

use crate::{cpu::{Cpu}, EventList, FutureEvent, FutureEventType};

pub const PPUMASK_SHOW_SPRITE: u8 = 1 << 4;
pub const PPUMASK_SHOW_SPRITE_LEFT: u8 = 1 << 2;
pub const PPUMASK_SHOW_BACKGROUND: u8 = 1 << 3;
pub const PPUMASK_SHOW_BACKGROUND_LEFT: u8 = 1<<1;
const EVENT_TYPE_SPRITE0: u32 = 0;
const EVENT_TYPE_VBLANKEND: u32 = 1;
const EVENT_TYPE_SCANLINE_END: u32 = 2;

const PPUSTATUS_VBLANK: u8 = 1 << 7;
const PPUSTATUS_SPRITE0_HIT: u8 = 1 << 6;
const PPUCTRL_VRAM_INCREMENT: u8 = 1 << 2;
const PPUCTRL_VBLANK: u8 = 1 << 7;
#[derive(Clone, Copy)]
pub struct PPUState {
    pub ppustatus: u8,
    pub ppuctrl: u8,
    pub ppumask: u8,
    pub oamaddr: u8,
    pub ppuscroll: Scroll,
    
    pub oam: [u8; 256],
    pub pallete: [u8; 32],
    pub last_read_byte: u8,


    pub next_write_latch: Latch,

    pub temp_addr: u16
}

impl PPUState {
    pub fn new() -> PPUState {
        PPUState { 
            ppustatus: 0, 
            ppuctrl: 0, 
            ppumask: 0, 
            oamaddr: 0, 
            ppuscroll: Scroll { x: 0, y: 0 }, 


            next_write_latch: Latch::Low, 
            oam: [0; 256], 
            last_read_byte: 0,
            pallete: [0; 32],
            temp_addr: 0,
        }
    }
}

pub struct PPU {
    pub current_state: PPUState,
    frame_start_cyc: u64,
}


pub trait PPUMemorySpace {
    fn ppu_write(&mut self, addr: u16, v: u8);
    fn ppu_read(&self, addr: u16, ) -> u8;
}

#[derive(Clone, Copy)]
pub enum Latch {
    Low,
    High,    
}

#[derive(Clone, Copy)]
pub struct Scroll {
    x: u8,
    y: u8,
}

pub trait DmaTransferSource {
    fn read_page_for_oam(&mut self, page: u8, cpu: &mut Cpu) -> [u8; 256];
}

impl PPU {
    pub fn new() -> PPU {
        PPU { 
            current_state: PPUState::new(),
            frame_start_cyc: 0,
        }
    }

    pub fn nmi_active(&self) -> bool {
        self.current_state.ppuctrl & PPUCTRL_VBLANK != 0
    }

    pub fn dma_transfer<T: DmaTransferSource + ?Sized >(&mut self, page: u8, source: &mut T, cpu: &mut Cpu) {
        self.current_state.oam = source.read_page_for_oam(page, cpu);
        cpu.cycle_count += 513;
    }

    pub fn context<'a>(&'a mut self, cart: &'a mut dyn PPUMemorySpace) -> PPUContext {
        PPUContext{
            ppu: self,
            cartridge: cart,
        }
    }

    pub fn drawing_context<'a>(&'a mut self, cart: &'a mut dyn PPUMemorySpace, eventlist: &'a mut EventList, framebuffer: &'a mut [u8; 240*256]) -> PPUDrawingContext {
        PPUDrawingContext{
            ppu: self,
            cartridge: cart,
            event_list: eventlist,
            framebuffer,
        }
    }

    pub fn set_sprite0_flag(&mut self) {
        self.current_state.ppustatus |= PPUSTATUS_SPRITE0_HIT;
    }

    fn compute_scanline(&self, cyc: u64) -> i64 {
        (((cyc-self.frame_start_cyc) * 3)/341) as i64 - 22
    }

}

pub struct PPUContext<'a> {
    cartridge: &'a mut dyn PPUMemorySpace,
    ppu: &'a mut PPU,
}

impl<'a> PPUContext<'a> {    
    fn write_ppudata(&mut self, v: u8) {
        let current_addr = self.ppu.current_state.get_addr();
        if current_addr >= 0x3f00 {
            let pallete_index = (current_addr & 0x1f) as usize;
            self.ppu.current_state.pallete[pallete_index] = v & 0x3f;

            if pallete_index % 4 == 0 {
                self.ppu.current_state.pallete[(pallete_index + 0x10) & 0x1f] = v & 0x3f;
            }
        } else {
            self.cartridge.ppu_write(current_addr, v);
        }

        if self.ppu.current_state.ppuctrl & PPUCTRL_VRAM_INCREMENT != 0 {
            self.ppu.current_state.set_addr(current_addr.wrapping_add(32));
        } else {
            self.ppu.current_state.set_addr(current_addr.wrapping_add(1));
        }
    }

    fn read_ppudata(&mut self) -> u8 {
        let ptr = self.ppu.current_state.get_addr();
        let value = self.cartridge.ppu_read(ptr);

        if self.ppu.current_state.ppuctrl & PPUCTRL_VRAM_INCREMENT != 0 {
            self.ppu.current_state.set_addr(ptr.wrapping_add(32));
        } else {
            self.ppu.current_state.set_addr(ptr.wrapping_add(1));
        }

        if ptr <= 0x3eff {
            let old_value = self.ppu.current_state.last_read_byte;
            self.ppu.current_state.last_read_byte = value;
            old_value
        } else {
            //When reading the pallete (addresses from 0x3f00 to 0x3fff), the byte written to last_read_byte is not actually the pallete value,
            //but the data that would appear mirrored "underneath" the pallete. Because of this, we don't need to change the value of last_read_byte.
            self.ppu.current_state.last_read_byte = value;
            self.ppu.current_state.pallete[(ptr & 0x1f) as usize] & 0x3f
        }
    }

    pub fn read(&mut self, addr: crate::memory_controller::MemoryPtr) -> u8 {
        let v = match addr.0 & 0x7 {
            2 => self.ppu.current_state.read_ppustatus(), //0x2000
            4 => self.ppu.current_state.oam[self.ppu.current_state.oamaddr as usize], //0x2004
            7 => self.read_ppudata(), //0x2007
            _ => {
                debug!("invalid read from ppu register (addr: ${:x})", addr.0);
                0
            }
        };
        v
    }
    pub fn write(&mut self, addr: crate::memory_controller::MemoryPtr, value: u8, _: &mut Cpu) {
        match addr.0 & 0x7 {
            0 => {
                let mut tmp = parse_addr(self.ppu.current_state.temp_addr);

                tmp.nametable = value & 0x3;

                self.ppu.current_state.temp_addr = tmp.addr();
                self.ppu.current_state.ppuctrl = (value & !0x3) | (self.ppu.current_state.ppuctrl & 0x3);

                //self.ppu.current_state.ppuctrl = value;
            }, //0x2000
            1 => {
                self.ppu.current_state.ppumask =  value; //0x2001
            },
            3 => self.ppu.current_state.oamaddr = value,            
            4 => self.ppu.current_state.write_oam_byte(value),            
            5 => {
                self.ppu.current_state.write_scroll(value);
            },
            6 => {
                self.ppu.current_state.write_addr(value);
            },
            7 => {
                self.write_ppudata(value);
            }

            _ => debug!("invalid write to ppu register addr {:x}", addr.0),
        }

    }

}


pub struct PPUDrawingContext<'a> {
    cartridge: &'a mut dyn PPUMemorySpace,
    ppu: &'a mut PPU,
    event_list: &'a mut EventList,
    framebuffer: &'a mut [u8; 240 * 256]
}

impl<'a> PPUDrawingContext<'a> {
    pub fn after_vblank(&mut self) {
        self.ppu.current_state.ppustatus &= !PPUSTATUS_SPRITE0_HIT;
        
        let s0 = DrawingContext { cartridge: self.cartridge, ppu: &self.ppu.current_state }.draw_scanline(
            self.framebuffer[..256].as_mut(),
            0,
        );
        if let Some((x, y)) = s0 {
            self.event_list.add_event(FutureEvent { cycle: (341*(y+22) + x) as u64, tp: FutureEventType::PPU(EVENT_TYPE_SPRITE0)});
        }
    }

    pub fn set_vblank_flag(&mut self, cyc: u64) {
        self.ppu.current_state.ppustatus |= PPUSTATUS_VBLANK;
        self.ppu.frame_start_cyc = cyc;

        self.event_list.add_event(FutureEvent { 
            cycle: 7502, tp: FutureEventType::PPU(EVENT_TYPE_VBLANKEND),
        });

        
        for i in 0..239 {
            self.event_list.add_event(FutureEvent { 
                cycle: 341*(i + 23), tp: FutureEventType::PPU(EVENT_TYPE_SCANLINE_END),
            });
        }
    }

    pub fn handle_event(&mut self, event_type_id: u32, cyc: u64) {
        match event_type_id {
            EVENT_TYPE_SCANLINE_END => {
                if self.ppu.current_state.ppumask & PPUMASK_SHOW_BACKGROUND != 0 {
                    let data = parse_addr(self.ppu.current_state.temp_addr);

                    //increment y position taking into account the current nametable
                    let nametable_base_y =  (if self.ppu.current_state.ppuctrl & 0x03 & 2 != 0 {0xf0} else {0u16}).wrapping_add(self.ppu.current_state.ppuscroll.y as u16).wrapping_add(1) % 480;
                    self.ppu.current_state.ppuctrl = self.ppu.current_state.ppuctrl & !0x2;
                    if nametable_base_y >= 240 {
                        self.ppu.current_state.ppuctrl = self.ppu.current_state.ppuctrl | 0x2;
                        self.ppu.current_state.ppuscroll.y = (nametable_base_y - 240) as u8;
                    } else {
                        self.ppu.current_state.ppuscroll.y = nametable_base_y as u8;
                    }
                    

                    self.ppu.current_state.ppuscroll.x = (data.x_pos) | (self.ppu.current_state.ppuscroll.x & 0x7);
                    self.ppu.current_state.ppuctrl = (self.ppu.current_state.ppuctrl & !0x1) | (data.nametable & 0x1);

                    let scanline = self.ppu.compute_scanline(cyc);

                    if scanline >=1 {
                        
                        let s0 = DrawingContext { cartridge: self.cartridge, ppu: &self.ppu.current_state }.draw_scanline(
                            self.framebuffer[scanline as usize*256..(scanline as usize+1)*256].as_mut(), 
                            scanline as u8
                        );
                        if let Some((x, y)) = s0 {
                            self.event_list.add_event(FutureEvent { cycle: (341*(y+22) + x) as u64, tp: FutureEventType::PPU(EVENT_TYPE_SPRITE0)});
                        }
                    }
                }
                
            },
            EVENT_TYPE_VBLANKEND => {
                if (self.ppu.current_state.ppumask & PPUMASK_SHOW_BACKGROUND != 0) || (self.ppu.current_state.ppumask & PPUMASK_SHOW_SPRITE != 0) {
                    let parsed = parse_addr(self.ppu.current_state.temp_addr);
                    self.ppu.current_state.ppuscroll.y = parsed.y_pos;

                    self.ppu.current_state.ppuctrl = (self.ppu.current_state.ppuctrl & !0x2) | (parsed.nametable & 0x2);
                }
                
                self.after_vblank();
            },
            EVENT_TYPE_SPRITE0 => {
                self.ppu.set_sprite0_flag();
            }
            _ => {unreachable!();}
        };
    }

}

pub struct DrawingContext<'a> {
    cartridge: &'a dyn PPUMemorySpace,
    ppu: &'a PPUState
}

impl<'a> DrawingContext<'a> {

    fn draw_scanline(&mut self, framebuffer: &mut [u8], scanline: u8) -> Option<(u16, u16)> {
        self.draw_background(
            framebuffer,
            self.ppu.ppuscroll.x as u16, 
            self.ppu.ppuscroll.y as u16, 
            256, 
            1
        );

        self.draw_sprites(
            framebuffer,
            256,
            1,
            scanline
        )
    }

    fn draw_background(&self, output: &mut [u8], x: u16, y: u16, w: u16, h: u16) {
        let nametable = self.ppu.ppuctrl & 0x03;

        for v in output.iter_mut() {
            *v = self.ppu.pallete[0];
        }

        let nametable_base_x =  (if nametable & 1 != 0 {0x100u16} else {0u16}).wrapping_add(x).wrapping_sub(8) % 512;
        let nametable_base_y =  (if nametable & 2 != 0 {0xf0} else {0u16}).wrapping_add(y) % 480;

        let tiles_w = if w >= 8 {(w / 8) + 2 } else {1};
        let tiles_h = if h >= 8 { h/8 } else {1};

        for a in 0..(tiles_w * tiles_h) {
            let nametable_x = (nametable_base_x.wrapping_sub(nametable_base_x % 8)).wrapping_add((a % 34) * 8);
            let nametable_y = (nametable_base_y.wrapping_sub(nametable_base_y % 8)).wrapping_add((a / 34) * 8);

            let current_nametable = match ((nametable_y%480) >= 240, (nametable_x%512) >= 256) {
                (false, false) => 0,
                (false, true) => 1,
                (true, false) => 2,
                (true, true) => 3,
            };
            let tile = self.cartridge.ppu_read(0x2000 + current_nametable * 0x400 + ((nametable_x % 256) / 8) + 32 * ((nametable_y % 240) / 8));

            let pallete_index = 8 * ((nametable_y % 240) / 32) + ((nametable_x & 0xff) / 32);
            let pallete_bit_select = (2 * (((nametable_y % 240) / 16) % 2) + (((nametable_x & 0xff) / 16) % 2)) * 2;
            let current_pallete = (self.cartridge.ppu_read(0x2000 + current_nametable * 0x400 + 0x3c0 + pallete_index) >> pallete_bit_select) & 3;

            self.draw_tile_section(
                nametable_x as i16 - nametable_base_x as i16 - 8, 
                nametable_y as i16 - nametable_base_y as i16, 
                current_pallete, 
                (self.ppu.ppuctrl & 0x10) >> 4,
                tile, 
                false, 
                false, 
                (self.ppu.ppumask & PPUMASK_SHOW_BACKGROUND_LEFT) == 0,
                true,
                false,
                output, w, h,
                0
            );
        }
    }

    fn draw_sprites(&self, output: &mut [u8], w: u16, h: u16, scanline: u8) -> Option<(u16, u16)> {
        let mut detected_sprite0: Option<(u16, u16)> = None;

        if self.ppu.ppumask & PPUMASK_SHOW_SPRITE != 0 {
            let sprite_size = self.ppu.ppuctrl & (1 << 5);
            for i in (0..64).rev() {
                let pos_y = self.ppu.oam[i * 4];
                let tile  = self.ppu.oam[i * 4 + 1];
                let byte3 = self.ppu.oam[i * 4 + 2];
                let pos_x = self.ppu.oam[i * 4 + 3];
                if sprite_size == 0 {
                    //8x8 sprite
                    let s0 = self.draw_tile_section(
                        pos_x as i16, 
                        pos_y as i16 - scanline as i16, 
                        (byte3 & 0x3) + 4, 
                        (self.ppu.ppuctrl & 0x8) >> 3,
                        tile, 
                        (byte3 & 0x40) != 0, 
                        (byte3 & 0x80) != 0, 
                        (self.ppu.ppumask & PPUMASK_SHOW_SPRITE_LEFT) == 0,
                        false, 
                        i == 0,
                        output, w, h,
                        scanline,
                    );

                    if let None = detected_sprite0 {
                        detected_sprite0 = s0;
                    }
                } else {
                    let flipy = (byte3 & 0x80) != 0;
                    //8x16 sprite
                    let s0 = self.draw_tile_section(
                        pos_x as i16, 
                        pos_y as i16 - scanline as i16, 
                        (byte3 & 0x3) + 4, 
                        tile & 0x1,
                        if !flipy {tile & !0x1} else {tile | 0x1}, 
                        (byte3 & 0x40) != 0, 
                        flipy, 
                        (self.ppu.ppumask & PPUMASK_SHOW_SPRITE_LEFT) == 0,
                        false, 
                        i == 0,
                        output, w, h,
                        scanline,
                    );
                    if let None = detected_sprite0 {
                        detected_sprite0 = s0;
                    }
                    let s0 = self.draw_tile_section(
                        pos_x as i16, 
                        pos_y as i16 + 8 - scanline as i16, 
                        (byte3 & 0x3) + 4, 
                        tile & 0x1,
                        if flipy {tile & !0x1} else {tile | 0x1},  
                        (byte3 & 0x40) != 0, 
                        flipy, 
                        (self.ppu.ppumask & PPUMASK_SHOW_SPRITE_LEFT) == 0,
                        false, 
                        i == 0,
                        output, w, h,
                        scanline,
                    );
                    if let None = detected_sprite0 {
                        detected_sprite0 = s0;
                    }
                }
                
            }
        }
        
        detected_sprite0
    }

    fn draw_tile_section(&self, 
        x: i16,
        y: i16, 
        pallete: u8, 
        pattern_t: u8, 
        tile: u8, 
        flipx: bool, 
        flipy: bool, 
        mask_left: bool, 
        background: bool, 
        sprite0: bool, 
        framebuffer: &mut [u8],
        framebuffer_w: u16,
        framebuffer_h: u16,
        scanline: u8,
    ) -> Option<(u16, u16)>{
        let mut detected_sprite0: Option<(u16, u16)> = None;

        for i in 0..8 {
            let byte1 = self.cartridge.ppu_read(i + 16*(tile as u16) + 0x1000 * (pattern_t as u16));
            let byte2 = self.cartridge.ppu_read(i + 16*(tile as u16) + 0x1000 * (pattern_t as u16) + 8);
            for a in 0..8 {
                let mask = 0x80 >> a;

                let mut color_index = 0u16;
                if byte1 & mask != 0 {
                    color_index |= 1;
                }
                if byte2 & mask != 0 {
                    color_index |= 2;
                }

                let screen_pos_x = if flipx { (x as i32) + 7 - (a as i32)} else {(x as i32) + (a as i32)};
                let screen_pos_y = if flipy { (y as i32) + 7 - (i as i32)} else {(y as i32) + (i as i32)};

                if screen_pos_x < 0 || screen_pos_y < 0 || screen_pos_x >= framebuffer_w as i32 || screen_pos_y >= framebuffer_h as i32 {
                    continue;
                }

                if mask_left && screen_pos_x < 8 {
                    continue;
                }

                if color_index == 0 {
                    continue;
                }

                let color = self.ppu.pallete[(4 * pallete as u16 + color_index) as usize];

                if sprite0 {
                    if let None = detected_sprite0 {
                        if framebuffer[(256 * screen_pos_y + screen_pos_x) as usize] & 0x80 != 0 {
                            detected_sprite0 = Some((screen_pos_x as u16, scanline as u16 + screen_pos_y as u16));
                        }
                    }
                }

                framebuffer[(256 * screen_pos_y + screen_pos_x) as usize] = color;
                if background {
                    //uses the top bit to mark this as an opaque background pixel
                    framebuffer[(256 * screen_pos_y + screen_pos_x) as usize] |= 0x80;
                }
                
            }
        }

        detected_sprite0
    }

}

impl PPUState {
    fn read_ppustatus(&mut self) -> u8 {
        let ppustatus_copy = self.ppustatus;

        self.last_read_byte = 0;
        self.next_write_latch = Latch::Low;

        self.ppustatus = self.ppustatus & (!PPUSTATUS_VBLANK); //clear vblank flag

        return ppustatus_copy;
    }

    fn write_oam_byte(&mut self, v: u8) {
        self.oam[self.oamaddr as usize] = v;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    fn get_addr(&self) -> u16 {
        //self.temp_addr
        PPURegister{
            nametable: self.ppuctrl & 0x3,
            x_pos: self.ppuscroll.x,
            y_pos: self.ppuscroll.y,
        }.addr()
    }

    fn set_addr(&mut self, v: u16) {
        let data = parse_addr(v);
        self.ppuscroll.x = data.x_pos;
        self.ppuctrl = (self.ppuctrl & !0x3) | (data.nametable);
        self.ppuscroll.y = data.y_pos;
        
        self.temp_addr = data.addr();
    }

    fn write_scroll(&mut self, v: u8) {
        match self.next_write_latch {
            Latch::Low => {
                //self.ppuscroll.x = v;

                let mut a = parse_addr(self.temp_addr);
                a.x_pos = v;
                self.temp_addr = a.addr();
                self.ppuscroll.x = (self.ppuscroll.x & (!0x7)) | (v & 0x7);

                self.next_write_latch = Latch::High;
            },
            Latch::High => {
                //self.ppuscroll.y = v;
                
                let mut a = parse_addr(self.temp_addr);
                a.y_pos = v;
                self.temp_addr = a.addr();

                self.ppuscroll.x = (a.x_pos & (!0x7)) | (self.ppuscroll.x & 0x7);
                self.next_write_latch = Latch::Low;
            }
        }
    }
    fn write_addr(&mut self, v: u8) {
        match self.next_write_latch {
            Latch::Low => {
                self.temp_addr = (self.temp_addr & 0x00ff) | (((v as u16) & 0x3f) << 8);
    
                self.next_write_latch = Latch::High;
            },
            Latch::High => {
                self.temp_addr &= 0xff00;
                self.temp_addr |= v as u16;

                let data = parse_addr(self.temp_addr);
        
                self.ppuscroll.x = data.x_pos | (self.ppuscroll.x & 0x7);
                self.ppuctrl = (self.ppuctrl & !0x3) | (data.nametable);
                self.ppuscroll.y = data.y_pos;

                self.next_write_latch = Latch::Low;
            },
            
        }
    }

}

struct PPURegister {
    x_pos: u8,
    y_pos: u8,
    nametable: u8
}

impl PPURegister {
    fn addr(&self) -> u16 {
        let mut number = 0u16;
        number |= self.x_pos as u16 >> 3;
        number |= (self.y_pos as u16 >> 3) << 5;
        number |= (self.nametable as u16 & 0x3) << 10;
        number |= (self.y_pos as u16 & 0x7) << 12;

        number
    }
}

fn parse_addr(v: u16) -> PPURegister {
    PPURegister {
         x_pos: ((v & 0x1f) << 3) as u8, 
         y_pos: (((v & 0x3e0) >> 2) | ((v & 0x7000) >> 12)) as u8, 
         nametable: (((v >> 10) & 0x3) as u8) 
    }
}
