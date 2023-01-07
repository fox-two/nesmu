mod addressing_modes;
mod instructions;
use core::fmt;

use log::debug;

use super::memory_controller::MemoryPtr;

pub trait CpuMemory {
    fn read(&mut self, addr: MemoryPtr, cpu: &mut Cpu) -> u8;
    fn write(&mut self, addr: MemoryPtr, value: u8, cpu: &mut Cpu);
}

pub trait CpuMemoryRead {
    fn read(&mut self, addr: MemoryPtr, cpu: &mut Cpu) -> u8;
}

pub trait CpuMemoryWrite {
    fn write(&mut self, addr: MemoryPtr, value: u8, cpu: &mut Cpu);
}

type CpuFlags = u8;

#[derive(Clone, Copy)]
enum Flags {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal = 1 << 3,
    Break = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Sign = 1 << 7,
}

trait BitField<K> {
    fn set(&mut self, bit: K, value: bool);
    fn get(&self, bit: Flags) -> bool;
}

impl BitField<Flags> for CpuFlags {
    fn set(&mut self, bit: Flags, value: bool) {
        if value {
            *self = *self | (bit as u8);
        } else {
            *self = *self & (!(bit as u8));
        }
    }
    fn get(&self, bit: Flags) -> bool {
        (self & (bit as u8)) != 0
    }
}

pub struct Cpu {
    pub accumulator: u8,
    pub x: u8,
    pub y: u8,
    pub flags: u8,
    pub stack_pointer: u8,
    pub program_counter: MemoryPtr,

    pub irq_requested: bool,
    pub cycle_count: u64,
    pub last_instruction: u8
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            accumulator: 0,
            x: 0,
            y: 0,
            flags: 0x24,
            stack_pointer: 0xfd,
            program_counter: MemoryPtr(0),
            cycle_count: 7,
            last_instruction: 0,
            irq_requested: false,
        }
    }

    pub fn context<T: CpuMemory>(&mut self, memory: T) -> CpuContext<T> {
        CpuContext {
            state: self,
            memory,
        }
    }

    #[allow(dead_code)]
    pub fn context_borrowed<'a, T: CpuMemory>(&'a mut self, memory: &'a mut T) -> CpuContext<BorrowedMemory<T>>  {
        CpuContext { 
            state: self, 
            memory: BorrowedMemory { mem: memory },
        }
    }
}

pub struct BorrowedMemory<'a, T: CpuMemory> {
    mem: &'a mut T
}

impl<'a, T: CpuMemory> CpuMemory for BorrowedMemory<'a, T> {
    fn read(&mut self, addr: MemoryPtr, cpu: &mut Cpu) -> u8 {
        self.mem.read(addr, cpu)
    }
    fn write(&mut self, addr: MemoryPtr, value: u8, cpu: &mut Cpu) {
        self.mem.write(addr, value, cpu)
    }
}

impl fmt::Display for Cpu {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "{:04X}\tA:{:02X}\tX:{:02X}\tY:{:02X}\tP:{:02X}\tSP:{:02X}\tLast instruction: {:02X}",
            self.program_counter.0,
            self.accumulator,
            self.x,
            self.y,
            self.flags,
            self.stack_pointer,
            self.last_instruction,
        )
    }
}

trait ReadInterface<'a, T: CpuMemory> {
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8;
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8);
}

trait ReadInterfaceBytesRead {
    fn bytes_read(&self) -> u16;
}



struct AccumulatorOperand;

impl<'a, T: CpuMemory> ReadInterface<'a, T> for AccumulatorOperand {
    fn get(&self, context: &mut CpuContext<'a, T>) -> u8 {
        context.state.accumulator
    }
    fn set(&self, context: &mut CpuContext<'a, T>, v: u8) {
        context.state.accumulator = v;
    }
}

impl ReadInterfaceBytesRead for AccumulatorOperand {
    fn bytes_read(&self) -> u16 {
        0
    }
}

pub struct CpuContext<'a, T: CpuMemory> {
    state: &'a mut Cpu,
    memory: T,
}

impl<'a, T: CpuMemory> CpuContext<'a, T> {
    pub fn reset(&mut self) {
        let entry_point = ((self.memory.read(MemoryPtr(0xfffd), self.state) as u16) << 8) | (self.memory.read(MemoryPtr(0xfffc), self.state) as u16);
        self.state.program_counter = MemoryPtr(entry_point);
    }

    pub fn execute_next_instruction(&mut self) -> Result<(), ()> {
        if self.state.irq_requested {
            self.irq();
        }
        
        let opcode = self.memory.read(self.state.program_counter, &mut self.state);

        use addressing_modes::*;
        use instructions::*;
        match opcode {
            0x69 => AdcOp::<Immediate>::exec(self),
            0x65 => AdcOp::<ZeroPage>::exec(self),
            0x75 => AdcOp::<ZeroPageX>::exec(self),
            0x6d => AdcOp::<Absolute>::exec(self),
            0x7d => AdcOp::<AbsoluteX>::exec(self),
            0x79 => AdcOp::<AbsoluteY>::exec(self),
            0x61 => AdcOp::<IndirectX>::exec(self),
            0x71 => AdcOp::<IndirectY>::exec(self),

            0x29 => AndOp::<Immediate>::exec(self),
            0x25 => AndOp::<ZeroPage>::exec(self),
            0x35 => AndOp::<ZeroPageX>::exec(self),
            0x2d => AndOp::<Absolute>::exec(self),
            0x3d => AndOp::<AbsoluteX>::exec(self),
            0x39 => AndOp::<AbsoluteY>::exec(self),
            0x21 => AndOp::<IndirectX>::exec(self),
            0x31 => AndOp::<IndirectY>::exec(self),

            0x0a => AslOp::<Accumulator>::exec(self),
            0x06 => AslOp::<ZeroPage>::exec(self),
            0x16 => AslOp::<ZeroPageX>::exec(self),
            0x0e => AslOp::<Absolute>::exec(self),
            0x1e => AslOp::<AbsoluteX>::exec(self),

            0x90 => BccOp::<Immediate>::exec(self),

            0xb0 => BcsOp::<Immediate>::exec(self),

            0xf0 => BeqOp::<Immediate>::exec(self),

            0x24 => BitOp::<ZeroPage>::exec(self),
            0x2c => BitOp::<Absolute>::exec(self),

            0x30 => BmiOp::<Immediate>::exec(self),

            0xd0 => BneOp::<Immediate>::exec(self),

            0x10 => BplOp::<Immediate>::exec(self),

            0x50 => BvcOp::<Immediate>::exec(self),

            0x70 => BvsOp::<Immediate>::exec(self),

            0x18 => ClcOp::<Implied>::exec(self),

            0xd8 => CldOp::<Implied>::exec(self),

            0x58 => CliOp::<Implied>::exec(self),

            0xb8 => ClvOp::<Implied>::exec(self),

            0xc9 => CmpOp::<Immediate>::exec(self),
            0xc5 => CmpOp::<ZeroPage>::exec(self),
            0xd5 => CmpOp::<ZeroPageX>::exec(self),
            0xcd => CmpOp::<Absolute>::exec(self),
            0xdd => CmpOp::<AbsoluteX>::exec(self),
            0xd9 => CmpOp::<AbsoluteY>::exec(self),
            0xc1 => CmpOp::<IndirectX>::exec(self),
            0xd1 => CmpOp::<IndirectY>::exec(self),

            0xe0 => CpxOp::<Immediate>::exec(self),
            0xe4 => CpxOp::<ZeroPage>::exec(self),
            0xec => CpxOp::<Absolute>::exec(self),

            0xc0 => CpyOp::<Immediate>::exec(self),
            0xc4 => CpyOp::<ZeroPage>::exec(self),
            0xcc => CpyOp::<Absolute>::exec(self),

            0xc6 => DecOp::<ZeroPage>::exec(self),
            0xd6 => DecOp::<ZeroPageX>::exec(self),
            0xce => DecOp::<Absolute>::exec(self),
            0xde => DecOp::<AbsoluteX>::exec(self),

            0xca => DexOp::<Implied>::exec(self),
            0x88 => DeyOp::<Implied>::exec(self),

            0x49 => EorOp::<Immediate>::exec(self),
            0x45 => EorOp::<ZeroPage>::exec(self),
            0x55 => EorOp::<ZeroPageX>::exec(self),
            0x4d => EorOp::<Absolute>::exec(self),
            0x5d => EorOp::<AbsoluteX>::exec(self),
            0x59 => EorOp::<AbsoluteY>::exec(self),
            0x41 => EorOp::<IndirectX>::exec(self),
            0x51 => EorOp::<IndirectY>::exec(self),

            0xe6 => IncOp::<ZeroPage>::exec(self),
            0xf6 => IncOp::<ZeroPageX>::exec(self),
            0xee => IncOp::<Absolute>::exec(self),
            0xfe => IncOp::<AbsoluteX>::exec(self),

            0xe8 => InxOp::<Implied>::exec(self),
            0xc8 => InyOp::<Implied>::exec(self),

            0x4c => JmpOp::<ImmediateU16>::exec(self),
            0x6c => JmpOp::<AbsoluteU16>::exec(self),

            0x20 => JsrOp::<ImmediateU16>::exec(self),

            0xa9 => LdaOp::<Immediate>::exec(self),
            0xa5 => LdaOp::<ZeroPage>::exec(self),
            0xb5 => LdaOp::<ZeroPageX>::exec(self),
            0xad => LdaOp::<Absolute>::exec(self),
            0xbd => LdaOp::<AbsoluteX>::exec(self),
            0xb9 => LdaOp::<AbsoluteY>::exec(self),
            0xa1 => LdaOp::<IndirectX>::exec(self),
            0xb1 => LdaOp::<IndirectY>::exec(self),

            0xa2 => LdxOp::<Immediate>::exec(self),
            0xa6 => LdxOp::<ZeroPage>::exec(self),
            0xb6 => LdxOp::<ZeroPageY>::exec(self),
            0xae => LdxOp::<Absolute>::exec(self),
            0xbe => LdxOp::<AbsoluteY>::exec(self),

            0xa0 => LdyOp::<Immediate>::exec(self),
            0xa4 => LdyOp::<ZeroPage>::exec(self),
            0xb4 => LdyOp::<ZeroPageX>::exec(self),
            0xac => LdyOp::<Absolute>::exec(self),
            0xbc => LdyOp::<AbsoluteX>::exec(self),

            0x4a => LsrOp::<Accumulator>::exec(self),
            0x46 => LsrOp::<ZeroPage>::exec(self),
            0x56 => LsrOp::<ZeroPageX>::exec(self),
            0x4e => LsrOp::<Absolute>::exec(self),
            0x5e => LsrOp::<AbsoluteX>::exec(self),

            0xea => NopOp::<Implied>::exec(self),

            0x09 => OraOp::<Immediate>::exec(self),
            0x05 => OraOp::<ZeroPage>::exec(self),
            0x15 => OraOp::<ZeroPageX>::exec(self),
            0x0d => OraOp::<Absolute>::exec(self),
            0x1d => OraOp::<AbsoluteX>::exec(self),
            0x19 => OraOp::<AbsoluteY>::exec(self),
            0x01 => OraOp::<IndirectX>::exec(self),
            0x11 => OraOp::<IndirectY>::exec(self),

            0x48 => PhaOp::<Implied>::exec(self),

            0x08 => PhpOp::<Implied>::exec(self),

            0x68 => PlaOp::<Implied>::exec(self),

            0x28 => PlpOp::<Implied>::exec(self),

            0x2a => RolOp::<Accumulator>::exec(self),
            0x26 => RolOp::<ZeroPage>::exec(self),
            0x36 => RolOp::<ZeroPageX>::exec(self),
            0x2e => RolOp::<Absolute>::exec(self),
            0x3e => RolOp::<AbsoluteX>::exec(self),

            0x6a => RorOp::<Accumulator>::exec(self),
            0x66 => RorOp::<ZeroPage>::exec(self),
            0x76 => RorOp::<ZeroPageX>::exec(self),
            0x6e => RorOp::<Absolute>::exec(self),
            0x7e => RorOp::<AbsoluteX>::exec(self),

            0x40 => RtiOp::<Implied>::exec(self),
            0x60 => RtsOp::<Implied>::exec(self),

            0xe9 => SbcOp::<Immediate>::exec(self),
            0xe5 => SbcOp::<ZeroPage>::exec(self),
            0xf5 => SbcOp::<ZeroPageX>::exec(self),
            0xed => SbcOp::<Absolute>::exec(self),
            0xfd => SbcOp::<AbsoluteX>::exec(self),
            0xf9 => SbcOp::<AbsoluteY>::exec(self),
            0xe1 => SbcOp::<IndirectX>::exec(self),
            0xf1 => SbcOp::<IndirectY>::exec(self),

            0x38 => SecOp::<Implied>::exec(self),

            0xf8 => SedOp::<Implied>::exec(self),

            0x78 => SeiOp::<Implied>::exec(self),

            0x85 => StaOp::<ZeroPage>::exec(self),
            0x95 => StaOp::<ZeroPageX>::exec(self),
            0x8d => StaOp::<Absolute>::exec(self),
            0x9d => StaOp::<AbsoluteX>::exec(self),
            0x99 => StaOp::<AbsoluteY>::exec(self),
            0x81 => StaOp::<IndirectX>::exec(self),
            0x91 => StaOp::<IndirectY>::exec(self),

            0x86 => StxOp::<ZeroPage>::exec(self),
            0x96 => StxOp::<ZeroPageY>::exec(self),
            0x8e => StxOp::<Absolute>::exec(self),

            0x84 => StyOp::<ZeroPage>::exec(self),
            0x94 => StyOp::<ZeroPageX>::exec(self),
            0x8c => StyOp::<Absolute>::exec(self),

            0xaa => TaxOp::<Accumulator>::exec(self),

            0xa8 => TayOp::<Accumulator>::exec(self),
            0xba => TsxOp::<Accumulator>::exec(self),
            0x8a => TxaOp::<Accumulator>::exec(self),
            0x9a => TxsOp::<Accumulator>::exec(self),
            0x98 => TyaOp::<Accumulator>::exec(self),

            _ => {
                debug!(
                    "unknown opcode {:X} at {:X}",
                    opcode, self.state.program_counter.0
                );
                return Err(());
            }
        };

        self.state.last_instruction = opcode;
        Ok(())
    }

    fn stack_push(&mut self, v: u8) {
        self.memory
            .write(MemoryPtr(self.state.stack_pointer as u16 + 0x0100), v, self.state);
        self.state.stack_pointer = self.state.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.state.stack_pointer = self.state.stack_pointer.wrapping_add(1);

        self.memory
            .read(MemoryPtr(self.state.stack_pointer as u16 + 0x0100), self.state)
    }

    pub fn nmi(&mut self) {
        self.stack_push((self.state.program_counter.0  >> 8) as u8);
        self.stack_push((self.state.program_counter.0 & 0xff) as u8);
        self.stack_push(self.state.flags);

        self.state.program_counter = MemoryPtr(((self.memory.read(MemoryPtr(0xfffb), self.state) as u16) << 8) | (self.memory.read(MemoryPtr(0xfffa), self.state) as u16));

    }

    fn irq(&mut self) {
        if self.state.flags.get(Flags::InterruptDisable) {
            return;
        }
        self.stack_push((self.state.program_counter.0  >> 8) as u8);
        self.stack_push((self.state.program_counter.0 & 0xff) as u8);
        self.stack_push(self.state.flags);

        self.state.program_counter = MemoryPtr(((self.memory.read(MemoryPtr(0xffff), self.state) as u16) << 8) | (self.memory.read(MemoryPtr(0xfffe), self.state) as u16));
    }
}

#[cfg(test)]
mod tests;
