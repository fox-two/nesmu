use std::marker::PhantomData;
use std::num::Wrapping;

use crate::memory_controller::MemoryPtr;

use super::addressing_modes::AddrMode;
use super::addressing_modes::AddrModeWrite;
use super::BitField;
use super::CpuContext;
use super::CpuMemory;
use super::Flags;

pub(super) trait Operation<'a, K: AddrMode<'a, T>, T: CpuMemory> {
    fn exec(state: &mut CpuContext<'a, T>) {
        match Self::operation(&K::new(state), state) {
            Some(x) => {
                state.state.program_counter = x;
            }
            None => state.state.program_counter += 1 + K::bytes_read(),
        }
        state.state.cycle_count += Self::get_cycles() + K::cycles();
    }

    fn operation(input: &K, state: &mut CpuContext<'a, T>) -> Option<MemoryPtr>;

    fn get_cycles() -> u64;
}

pub(super) struct AdcOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for AdcOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let src = input.get(cpu);
        let tmp = u16::from(src)
            + u16::from(cpu.state.accumulator)
            + (if cpu.state.flags.get(Flags::Carry) {
                1
            } else {
                0
            });

        cpu.state.flags.set(Flags::Zero, (tmp & 0xff) == 0);
        cpu.state.flags.set(Flags::Sign, (tmp & (1 << 7)) != 0);
        cpu.state.flags.set(
            Flags::Overflow,
            (((cpu.state.accumulator ^ src) & 0x80) == 0)
                && ((u16::from(cpu.state.accumulator) ^ tmp) & 0x80) != 0,
        );
        cpu.state.flags.set(Flags::Carry, tmp > 0xff);

        cpu.state.accumulator = tmp as u8;

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct AndOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for AndOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let src = input.get(cpu) & cpu.state.accumulator;
        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);
        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.accumulator = src;

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct AslOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for AslOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut src = input.get(cpu);
        cpu.state
            .flags
            .set(Flags::Carry, if src & 0x80 != 0 { true } else { false });
        src <<= 1;
        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);
        input.set(cpu, src);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BccOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BccOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if !cpu.state.flags.get(Flags::Carry) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BcsOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BcsOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if cpu.state.flags.get(Flags::Carry) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BeqOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BeqOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if cpu.state.flags.get(Flags::Zero) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BitOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BitOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let src = input.get(cpu);

        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);
        cpu.state.flags.set(Flags::Overflow, (0x40 & src) != 0);
        cpu.state
            .flags
            .set(Flags::Zero, (src & cpu.state.accumulator) == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BmiOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BmiOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if cpu.state.flags.get(Flags::Sign) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BneOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BneOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if !cpu.state.flags.get(Flags::Zero) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BplOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BplOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if !cpu.state.flags.get(Flags::Sign) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BvcOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BvcOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if !cpu.state.flags.get(Flags::Overflow) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct BvsOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for BvsOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        if cpu.state.flags.get(Flags::Overflow) {
            Some(branch_implementation(input, cpu))
        } else {
            None
        }
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct ClcOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for ClcOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::Carry, false);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct CldOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for CldOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::Decimal, false);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct CliOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for CliOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::InterruptDisable, false);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct ClvOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for ClvOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::Overflow, false);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct CmpOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for CmpOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let arg = input.get(cpu);

        let (arg, _) = (arg as u16).overflowing_neg();

        let aux = (Wrapping(cpu.state.accumulator as u16) + Wrapping(arg)).0;
        cpu.state.flags.set(Flags::Carry, aux < 0x100);
        cpu.state.flags.set(Flags::Sign, aux & 0x80 != 0);
        cpu.state.flags.set(Flags::Zero, (aux & 0xff) == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct CpxOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for CpxOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let arg = input.get(cpu);

        let (arg, _) = (arg as u16).overflowing_neg();

        let aux = (Wrapping(cpu.state.x as u16) + Wrapping(arg)).0;
        cpu.state.flags.set(Flags::Carry, aux < 0x100);
        cpu.state.flags.set(Flags::Sign, aux & 0x80 != 0);
        cpu.state.flags.set(Flags::Zero, (aux & 0xff) == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct CpyOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for CpyOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let arg = input.get(cpu);

        let (arg, _) = (arg as u16).overflowing_neg();

        let aux = (Wrapping(cpu.state.y as u16) + Wrapping(arg)).0;
        cpu.state.flags.set(Flags::Carry, aux < 0x100);
        cpu.state.flags.set(Flags::Sign, aux & 0x80 != 0);
        cpu.state.flags.set(Flags::Zero, (aux & 0xff) == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct DecOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for DecOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let result = input.get(cpu).wrapping_sub(1);
        cpu.state.flags.set(Flags::Sign, (result & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, result == 0);

        input.set(cpu, result);
        None
    }

    fn get_cycles() -> u64 {
        4
    }
}

pub(super) struct DexOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for DexOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.x = cpu.state.x.wrapping_sub(1);

        cpu.state.flags.set(Flags::Sign, (cpu.state.x & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, cpu.state.x == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct DeyOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for DeyOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.y = cpu.state.y.wrapping_sub(1);

        cpu.state.flags.set(Flags::Sign, (cpu.state.y & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, cpu.state.y == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct EorOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for EorOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.accumulator = cpu.state.accumulator ^ input.get(cpu);
        cpu.state
            .flags
            .set(Flags::Sign, (cpu.state.accumulator & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, cpu.state.accumulator == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct IncOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for IncOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let result = input.get(cpu).wrapping_add(1);
        cpu.state.flags.set(Flags::Sign, (result & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, result == 0);

        input.set(cpu, result);
        None
    }

    fn get_cycles() -> u64 {
        4
    }
}

pub(super) struct InxOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for InxOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.x = cpu.state.x.wrapping_add(1);

        cpu.state.flags.set(Flags::Sign, (cpu.state.x & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, cpu.state.x == 0);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct InyOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T>> Operation<'a, K, T> for InyOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.y = cpu.state.y.wrapping_add(1);

        cpu.state.flags.set(Flags::Sign, (cpu.state.y & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, cpu.state.y == 0);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct JmpOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u16>> Operation<'a, K, T> for JmpOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        Some(MemoryPtr(input.get(cpu)))
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct JsrOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u16>> Operation<'a, K, T> for JsrOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let return_point = (cpu.state.program_counter + K::bytes_read()).0;

        cpu.stack_push((return_point >> 8) as u8);
        cpu.stack_push((return_point & 0xff) as u8);

        Some(MemoryPtr(input.get(cpu)))
    }
    fn get_cycles() -> u64 {
        4
    }
}

pub(super) struct LdaOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for LdaOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let value = input.get(cpu);
        cpu.state.flags.set(Flags::Sign, (value & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, value == 0);

        cpu.state.accumulator = value;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct LdxOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for LdxOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let value = input.get(cpu);
        cpu.state.flags.set(Flags::Sign, (value & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, value == 0);

        cpu.state.x = value;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct LdyOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for LdyOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let value = input.get(cpu);
        cpu.state.flags.set(Flags::Sign, (value & 0x80) != 0);
        cpu.state.flags.set(Flags::Zero, value == 0);

        cpu.state.y = value;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct LsrOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for LsrOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut src = input.get(cpu);
        cpu.state
            .flags
            .set(Flags::Carry, if src & 0x01 != 0 { true } else { false });
        src >>= 1;
        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.flags.set(Flags::Sign, false);
        input.set(cpu, src);

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct NopOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for NopOp<K> {
    fn operation(_: &K, _: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct OraOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for OraOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let src = input.get(cpu) | cpu.state.accumulator;

        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);
        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.accumulator = src | cpu.state.accumulator;

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct PhaOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for PhaOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.stack_push(cpu.state.accumulator);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct PhpOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for PhpOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut flags = cpu.state.flags;
        flags.set(Flags::Break, true);
        cpu.stack_push(flags);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct PlaOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for PlaOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let value = cpu.stack_pop();

        cpu.state.flags.set(Flags::Sign, (value & (1 << 7)) != 0);
        cpu.state.flags.set(Flags::Zero, value == 0);

        cpu.state.accumulator = value;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct PlpOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for PlpOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut value = cpu.stack_pop();

        value.set(Flags::Unused, true);
        value.set(Flags::Break, false);

        cpu.state.flags = value;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct RolOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for RolOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut src = input.get(cpu) as i16;
        src <<= 1;

        if cpu.state.flags.get(Flags::Carry) {
            src |= 0x01;
        }
        cpu.state.flags.set(Flags::Carry, src > 0xff);

        src &= 0xff;

        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);

        input.set(cpu, src as u8);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct RorOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for RorOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut src = input.get(cpu) as u16;
        if cpu.state.flags.get(Flags::Carry) {
            src |= 0x100;
        }
        cpu.state.flags.set(Flags::Carry, (src & 0x01) != 0);
        src >>= 1;

        cpu.state.flags.set(Flags::Zero, src == 0);
        cpu.state.flags.set(Flags::Sign, (src & (1 << 7)) != 0);

        input.set(cpu, src as u8);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct RtiOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for RtiOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let mut flags = cpu.stack_pop();
        flags.set(Flags::Unused, true);
        cpu.state.flags = flags;
        
        let addr = (cpu.stack_pop() as u16) | ((cpu.stack_pop() as u16) << 8);

        Some(MemoryPtr(addr))
    }
    fn get_cycles() -> u64 {
        6
    }
}

pub(super) struct RtsOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for RtsOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let addr = ((cpu.stack_pop() as u16) | ((cpu.stack_pop() as u16) << 8)).wrapping_add(1);

        Some(MemoryPtr(addr))
    }
    fn get_cycles() -> u64 {
        6
    }
}

pub(super) struct SbcOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for SbcOp<K> {
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        let src = input.get(cpu);
        let mut tmp = u16::wrapping_add(
            u16::from(cpu.state.accumulator),
            u16::from(src).wrapping_neg(),
        );

        if !cpu.state.flags.get(Flags::Carry) {
            tmp = tmp.wrapping_add((1 as u16).wrapping_neg());
        }

        cpu.state.flags.set(Flags::Zero, tmp == 0);
        cpu.state.flags.set(Flags::Sign, (tmp & (1 << 7)) != 0);
        cpu.state.flags.set(
            Flags::Overflow,
            (((cpu.state.accumulator ^ src) & 0x80) != 0)
                && ((u16::from(cpu.state.accumulator) ^ tmp) & 0x80) != 0,
        );
        cpu.state.flags.set(Flags::Carry, tmp < 0x100);

        cpu.state.accumulator = (tmp & 0xff) as u8;

        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct SecOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for SecOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::Carry, true);
        None
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct SedOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for SedOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::Decimal, true);
        None
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct SeiOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for SeiOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.flags.set(Flags::InterruptDisable, true);
        None
    }
    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct StaOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for StaOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        input.set(cpu, cpu.state.accumulator);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct StxOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for StxOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        input.set(cpu, cpu.state.x);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

pub(super) struct StyOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8> + AddrModeWrite<'a, T>> Operation<'a, K, T>
    for StyOp<K>
{
    fn operation(input: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        input.set(cpu, cpu.state.y);
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

macro_rules! move_register_op {
    ($name:ident, $origin:ident, $destination:ident) => {
        pub(super) struct $name<K> {
            p: PhantomData<K>,
        }

        impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for $name<K> {
            fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
                let tmp = cpu.state.$origin;
                cpu.state.flags.set(Flags::Zero, tmp == 0);
                cpu.state.flags.set(Flags::Sign, (tmp & (1 << 7)) != 0);
                cpu.state.$destination = tmp;
                None
            }

            fn get_cycles() -> u64 {
                2
            }
        }
    };
}

move_register_op!(TaxOp, accumulator, x);
move_register_op!(TayOp, accumulator, y);
move_register_op!(TsxOp, stack_pointer, x);
move_register_op!(TxaOp, x, accumulator);
move_register_op!(TyaOp, y, accumulator);

pub(super) struct TxsOp<K> {
    p: PhantomData<K>,
}

impl<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>> Operation<'a, K, T> for TxsOp<K> {
    fn operation(_: &K, cpu: &mut CpuContext<'a, T>) -> Option<MemoryPtr> {
        cpu.state.stack_pointer = cpu.state.x;
        None
    }

    fn get_cycles() -> u64 {
        2
    }
}

///////////////////////////////////////////////////

fn branch_implementation<'a, T: CpuMemory, K: AddrMode<'a, T, Tp = u8>>(
    input: &K,
    cpu: &mut CpuContext<'a, T>,
) -> MemoryPtr {
    let destination = MemoryPtr(sum_u16_with_signed_u8(
        cpu.state.program_counter.0 + 1 + K::bytes_read(),
        input.get(cpu),
    ));

    cpu.state.cycle_count += 1;

    if (cpu.state.program_counter.0 & 0xff00) != (destination.0 & 0xff00) {
        cpu.state.cycle_count += 1;
    }

    destination
}

fn sum_u16_with_signed_u8(a: u16, v: u8) -> u16 {
    (a as i32 + ((v as i8) as i32)) as u16
}
