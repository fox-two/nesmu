use super::*;
use crate::memory_controller::Ram;

#[test]
fn test_adc_instruction() {
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x69, 0x50]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x50,
            flags: 0,
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x69, 0x50]),
            instructions_to_execute: 1,
        },
    );
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x69, 0x50]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x50,
            flags: 0,
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x69, 0x50]),
            instructions_to_execute: 1,
        },
    );

    //test zero page adc
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(4),
            ram: pad_ram(&[0x0, 0x1a, 0x0, 0x0, 0x65, 0x01]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x1a,
            flags: 0,
            program_counter: MemoryPtr(6),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0, 0x1a, 0x0, 0x0, 0x65, 0x01]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_and_instruction() {
    process_testcase(
        RelevantState {
            accumulator: 0xF0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x29, 0x11]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x10,
            flags: 0,
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x29, 0x11]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_asl_instruction() {
    process_testcase(
        RelevantState {
            accumulator: 0x01,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x02,
            flags: 0,
            program_counter: MemoryPtr(1),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
    );

    process_testcase(
        RelevantState {
            accumulator: 0x80,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x00,
            flags: new_flags(&[Flags::Carry, Flags::Zero]),
            program_counter: MemoryPtr(1),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
    );

    process_testcase(
        RelevantState {
            accumulator: 0x40,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x80,
            flags: new_flags(&[Flags::Sign]),
            program_counter: MemoryPtr(1),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0a]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_bcc_instruction() {
    //no branch taken
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Carry]),
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(3),
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, ((-3) as i8) as u8]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Carry]),
            program_counter: MemoryPtr(5),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, ((-3) as i8) as u8]),
            instructions_to_execute: 1,
        },
    );

    //branch taken backwards
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(3),
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, ((-5) as i8) as u8]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            program_counter: MemoryPtr(0),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, ((-5) as i8) as u8]),
            instructions_to_execute: 1,
        },
    );

    //branch taken forwards
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(3),
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, 3]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            program_counter: MemoryPtr(8),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x0, 0x0, 0x0, 0x90, 3]),
            instructions_to_execute: 1,
        },
    );

    //branch taken backwards, crossing page
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x90, (-3 as i8) as u8, 0x0, 0x0, 0x0]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            program_counter: MemoryPtr(0xFFFF),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x90, (-3 as i8) as u8, 0x0, 0x0, 0x0]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_bcs_instruction() {
    //branch taken
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Carry]),
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xb0, 0x70]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Carry]),
            program_counter: MemoryPtr(0x72),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xb0, 0x70]),
            instructions_to_execute: 1,
        },
    );

    //branch not taken
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xb0, 0x70]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xb0, 0x70]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_beq_instruction() {
    //branch taken
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Zero]),
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xf0, 0x70]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: new_flags(&[Flags::Zero]),
            program_counter: MemoryPtr(0x72),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xf0, 0x70]),
            instructions_to_execute: 1,
        },
    );

    //branch not taken
    process_testcase(
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xf0, 0x70]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0x0,
            flags: 0,
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xf0, 0x70]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_cmp_instruction() {
    //equal result
    process_testcase(
        RelevantState {
            accumulator: 4,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xc9, 4]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 4,
            flags: new_flags(&[Flags::Zero, Flags::Carry]),
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xc9, 4]),
            instructions_to_execute: 1,
        },
    );

    //negative result
    process_testcase(
        RelevantState {
            accumulator: 4,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xc9, 8]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 4,
            flags: new_flags(&[Flags::Sign]),
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xc9, 8]),
            instructions_to_execute: 1,
        },
    );

    //positive result
    process_testcase(
        RelevantState {
            accumulator: 40,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xc9, 10]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 40,
            flags: new_flags(&[Flags::Carry]),
            program_counter: MemoryPtr(2),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xc9, 10]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_dec_instruction() {
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xce, 6, 0, 0xce, 6, 0, 21]),
            instructions_to_execute: 2,
        },
        RelevantState {
            accumulator: 0,
            flags: 0,
            program_counter: MemoryPtr(6),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xce, 6, 0, 0xce, 6, 0, 19]),
            instructions_to_execute: 2,
        },
    );
}

#[test]
fn test_dex_instruction() {
    //underflow
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0xca]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0,
            flags: new_flags(&[Flags::Sign]),
            program_counter: MemoryPtr(1),
            x: 0xff,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0xca]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_jmp_instruction() {
    //direct jump
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x4c, 0xfa, 0xfc]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0,
            flags: 0,
            program_counter: MemoryPtr(0xfcfa),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x4c, 0xfa, 0xfc]),
            instructions_to_execute: 1,
        },
    );

    //indirect jump
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0,
            program_counter: MemoryPtr(0),
            ram: pad_ram(&[0x6c, 0x03, 0x00, 0xad, 0xde]),
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0,
            flags: 0,
            program_counter: MemoryPtr(0xdead),
            x: 0,
            y: 0,
            stack_pointer: 0,
            ram: pad_ram(&[0x6c, 0x03, 0x00, 0xad, 0xde]),
            instructions_to_execute: 1,
        },
    );
}

#[test]
fn test_jsr_instruction() {
    let testram = pad_ram(&[0, 0x20, 0xad, 0xde]);

    let mut expectedram = pad_ram(&[0, 0x20, 0xad, 0xde]);

    expectedram[0x01ff] = 0;
    expectedram[0x01fe] = 3;

    //direct jump
    process_testcase(
        RelevantState {
            accumulator: 0,
            flags: 0,
            x: 0,
            y: 0,
            stack_pointer: 0xff,
            program_counter: MemoryPtr(1),
            ram: testram,
            instructions_to_execute: 1,
        },
        RelevantState {
            accumulator: 0,
            flags: 0,
            program_counter: MemoryPtr(0xdead),
            x: 0,
            y: 0,
            stack_pointer: 0xfd,
            ram: expectedram,
            instructions_to_execute: 1,
        },
    );

}


fn pad_ram(data: &[u8]) -> [u8; 2048] {
    let mut ram_state: [u8; 2048] = [0; 2048];
    ram_state[..data.len()].copy_from_slice(&data);
    ram_state
}

#[derive(Debug)]
struct RelevantState {
    accumulator: u8,
    flags: u8,
    program_counter: MemoryPtr,
    x: u8,
    y: u8,
    ram: [u8; 2048],
    instructions_to_execute: u32,
    stack_pointer: u8,
}
impl PartialEq for RelevantState {
    fn eq(&self, other: &Self) -> bool {
        if self.ram != other.ram[..self.ram.len()] {
            return false;
        }

        if self.accumulator != other.accumulator {
            return false;
        }
        if self.flags != other.flags {
            return false;
        }
        if self.program_counter != other.program_counter {
            return false;
        }
        if self.x != other.x {
            return false;
        }
        if self.y != other.y {
            return false;
        }
        if self.stack_pointer != other.stack_pointer {
            return false;
        }
        return true;
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
fn process_testcase(initial: RelevantState, expected: RelevantState) {
    let mut ram = Ram::new();
    ram.set_ram_state(initial.ram);

    let mut cpu = Cpu{
        accumulator:  initial.accumulator,
        flags: initial.flags,
        program_counter: initial.program_counter,
        x: initial.x,
        y: initial.y,
        stack_pointer: initial.stack_pointer,
        cycle_count: 0,
        last_instruction: 0,
        irq_requested: false,
    };

    for _ in 0..initial.instructions_to_execute {
        match cpu.context_borrowed(&mut ram).execute_next_instruction() {
            Err(()) => panic!("error executing instruction"),
            _ => (),
        }
    }

    let got = RelevantState {
        accumulator: cpu.accumulator,
        flags: cpu.flags as u8,
        x: cpu.x,
        y: cpu.y,
        stack_pointer: cpu.stack_pointer,
        instructions_to_execute: initial.instructions_to_execute,
        program_counter: cpu.program_counter,
        ram: ram.dump_ram(),
    };
    assert_eq!(expected, got);
}

fn new_flags(flags: &[Flags]) -> CpuFlags {
    let mut v: CpuFlags = 0;

    for k in flags {
        v.set(*k, true);
    }
    v
}
