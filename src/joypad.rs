use crate::memory_controller::MemoryPtr;

#[derive(Clone, Copy)]
pub enum Button {
    A=0,
    B=1,
    SELECT=2,
    START=3,
    UP=4,
    DOWN=5,
    LEFT=6,
    RIGHT=7
}

pub struct Joypad {
    strobe: bool,
    state: [bool; 8],
    current_button: usize
}


impl Joypad {
    pub fn new() -> Joypad {
        Joypad { 
            state: [false; 8],
            strobe: false,
            current_button: 0,
        }
    }

    pub fn set_state(&mut self, b :Button, pressed: bool) {
        self.state[b as usize] = pressed;
    }
    

    pub fn read(&mut self, _: MemoryPtr) -> u8 {
        if self.strobe {
            return if self.state[0] {1} else {0};
        }
        let result = if self.state[self.current_button] {1} else {0};
        self.current_button += 1;
        return result;
    }
    pub fn write(&mut self, _: MemoryPtr, value: u8){
        if value & 0x1 != 0 {
            self.strobe = true;
            self.current_button = 0;
        } else {
            self.strobe = false;
        }
    }
}

