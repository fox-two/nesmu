use crate::memory_controller::MemoryPtr;

use super::{CpuMemory, CpuContext};


pub trait AddrMode<'a, T: CpuMemory> {
    type Tp;
    fn new(context: &mut CpuContext<'a, T>) -> Self;
    fn get(&self, context: &mut CpuContext<'a, T>) -> Self::Tp;
    fn bytes_read() -> u16;
    fn cycles() -> u64;
}

pub trait AddrModeWrite<'a, T: CpuMemory> {
    fn set(&self,  context: &mut CpuContext<'a, T>, v: u8);
}


pub struct Immediate;

impl <'a, T: CpuMemory> AddrMode<'a, T> for Immediate {
    type Tp = u8;
    fn new(_: &mut CpuContext<'a, T>) -> Immediate {
        return Immediate{}
    }
    
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(context.state.program_counter + 1, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        0
    }
}


pub struct ZeroPage {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for ZeroPage {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> ZeroPage {
        ZeroPage { 
            addr: MemoryPtr(context.memory.read(context.state.program_counter + 1, context.state) as u16)
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        1
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for ZeroPage {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}


pub struct ZeroPageX {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for ZeroPageX {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> ZeroPageX {
        ZeroPageX { 
            addr: MemoryPtr(context.memory.read(context.state.program_counter + 1, context.state).wrapping_add(context.state.x) as u16),
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        2
    }
}

impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for ZeroPageX {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}


pub struct ZeroPageY {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for ZeroPageY {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> ZeroPageY {
        ZeroPageY { 
            addr: MemoryPtr(context.memory.read(context.state.program_counter + 1, context.state).wrapping_add(context.state.y) as u16),
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        2
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for ZeroPageY {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}


pub struct Absolute {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for Absolute {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> Absolute {
        let low = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        let high = context.memory.read(context.state.program_counter + 2, context.state) as u16;
        Absolute { 
            addr: MemoryPtr(high << 8 | low)
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        2
    }
    fn cycles() -> u64 {
        2
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for Absolute {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}


pub struct AbsoluteX {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for AbsoluteX {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> AbsoluteX {
        let low = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        let high = context.memory.read(context.state.program_counter + 2, context.state) as u16;
        //TODO: check if page boundary was crossed and add extra cycle
        AbsoluteX { 
            addr: MemoryPtr((high << 8 | low) + context.state.x as u16)
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        2
    }
    fn cycles() -> u64 {
        2
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for AbsoluteX {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}

pub struct AbsoluteY {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for AbsoluteY {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> AbsoluteY {
        let low = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        let high = context.memory.read(context.state.program_counter + 2, context.state) as u16;
        //TODO: check if page boundary was crossed
        AbsoluteY { 
            addr: MemoryPtr((high << 8 | low).wrapping_add(context.state.y as u16))
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        2
    }
    fn cycles() -> u64 {
        2
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for AbsoluteY {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}


pub struct IndirectX {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for IndirectX {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> IndirectX {
        let base = (context.memory.read(context.state.program_counter + 1, context.state) as u16 + context.state.x as u16) & 0xff;
        IndirectX { 
            addr: MemoryPtr((context.memory.read(MemoryPtr((base+1) & 0xff), context.state) as u16) << 8 | context.memory.read(MemoryPtr(base), context.state) as u16) 
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        4
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for IndirectX {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}

pub struct IndirectY {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for IndirectY {
    type Tp = u8;
    fn new(context: &mut CpuContext<'a, T>) -> IndirectY {
        let base = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        IndirectY { 
            addr: MemoryPtr((context.memory.read(MemoryPtr((base+1) & 0xff), context.state) as u16) << 8 | context.memory.read(MemoryPtr(base), context.state) as u16) + context.state.y.into() 
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.memory.read(self.addr, context.state)
    }
    fn bytes_read() -> u16 {
        1
    }
    fn cycles() -> u64 {
        3
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for IndirectY {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.memory.write(self.addr, v, context.state);
    }
}

pub struct Accumulator;

impl <'a, T: CpuMemory> AddrMode<'a, T> for Accumulator {
    type Tp = u8;
    fn new(_: &mut CpuContext<'a, T>) -> Accumulator {
        Accumulator
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.state.accumulator
    }
    fn bytes_read() -> u16 {
        0
    }
    fn cycles() -> u64 {
        0
    }
}


impl <'a, T: CpuMemory> AddrModeWrite<'a, T> for Accumulator {
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.state.accumulator = v
    }
}


pub struct Implied;

impl <'a, T: CpuMemory> AddrMode<'a, T> for Implied {
    type Tp = u8;
    fn new(_: &mut CpuContext<'a, T>) -> Implied {
        Implied
    }
    fn get(&self, _: &mut CpuContext<'a, T>) -> u8 {
        unreachable!("cannot read with implied addressing mode")
    }
    fn bytes_read() -> u16 {
        0
    }
    fn cycles() -> u64 {
        0
    }
}



pub struct ImmediateU16 {
    operand: u16
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for ImmediateU16 {
    type Tp = u16;
    fn new(context: &mut CpuContext<'a, T>) -> ImmediateU16 {
        let low = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        let high = context.memory.read(context.state.program_counter + 2, context.state) as u16;
        ImmediateU16 { 
            operand: (high << 8 | low)
        }
    }
    fn get(&self, _: &mut CpuContext<'a, T>) -> u16 {
        return self.operand;
    }
    fn bytes_read() -> u16 {
        2
    }
    fn cycles() -> u64 {
        2
    }
}

pub struct AbsoluteU16 {
    addr: MemoryPtr
}

impl <'a, T: CpuMemory> AddrMode<'a, T> for AbsoluteU16 {
    type Tp = u16;
    fn new(context: &mut CpuContext<'a, T>) -> AbsoluteU16 {
        let low = context.memory.read(context.state.program_counter + 1, context.state) as u16;
        let high = context.memory.read(context.state.program_counter + 2, context.state) as u16;
        AbsoluteU16 { 
            addr: MemoryPtr(high << 8 | low)
        }
    }
    fn get(&self, context: &mut CpuContext<'a, T>) -> u16 {
        let low = context.memory.read(self.addr, context.state) as u16;
        let high = context.memory.read(MemoryPtr((self.addr.0 & 0xff00) | (self.addr.0.wrapping_add(1) &0xff)), context.state )as u16;

        high << 8 | low
    }
    fn bytes_read() -> u16 {
        2
    }
    fn cycles() -> u64 {
        2
    }
}
