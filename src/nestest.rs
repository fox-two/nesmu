use std::{
    fs::File,
    io::{self, BufRead},
};

use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    cpu::{Cpu},
    ines_rom_file,
    memory_controller::{MemoryPtr}, Nes,
};

#[test]
fn nestest() {
    let mut builder = Builder::new();
    builder.target(Target::Stdout);
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();

    let x = ines_rom_file::Rom::new("nestest.nes").unwrap();

    let k = x.get_cpu_mapper().unwrap();

    let mut console = Nes::new(k);
    
    console.cpu = Cpu {
        accumulator: 0,
        x: 0,
        y: 0,
        flags: 0x24,
        stack_pointer: 0xfd,
        program_counter: MemoryPtr(0xc000),
        cycle_count: 7,
        last_instruction: 0,
        irq_requested: false
    };

    for (line, expected_state) in reference_log_iter().enumerate() {
        if line == 5002 {
            //from this position unimplemented opcodes are tested
            break;
        }
        if console.cpu != expected_state {
            panic!(
                "states differ at line {}!\nGot: {}\t\t Expected: {}",
                line + 1,
                console.cpu,
                expected_state
            );
        }

        console.cpu_context().execute_next_instruction().unwrap();
    }
}

impl PartialEq for Cpu {
    fn eq(&self, other: &Self) -> bool {
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

fn reference_log_iter() -> impl Iterator<Item = Cpu> {
    io::BufReader::new(File::open("nestest.log").unwrap())
        .lines()
        .filter_map(|x| {
            lazy_static! {
                static ref RE: Regex =
                    Regex::new("(.{4}).*A:(.{2}) X:(.{2}) Y:(.{2}) P:(.{2}) SP:(.{2})").unwrap();
            }

            let k = x.unwrap();
            let captures = RE.captures(&k).unwrap();

            let value = Cpu {
                program_counter: MemoryPtr(
                    u16::from_str_radix(captures.get(1).unwrap().as_str(), 16).unwrap(),
                ),
                accumulator: u8::from_str_radix(captures.get(2).unwrap().as_str(), 16).unwrap(),
                x: u8::from_str_radix(captures.get(3).unwrap().as_str(), 16).unwrap(),
                y: u8::from_str_radix(captures.get(4).unwrap().as_str(), 16).unwrap(),
                flags: u8::from_str_radix(captures.get(5).unwrap().as_str(), 16).unwrap(),
                stack_pointer: u8::from_str_radix(captures.get(6).unwrap().as_str(), 16).unwrap(),
                cycle_count: 0,
                last_instruction: 0,
                irq_requested: false,
            };
            Some(value)
        })
}
