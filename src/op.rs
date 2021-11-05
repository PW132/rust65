use crate::cpu::CpuStatus;
use crate::bus;
use crate::bus::Segment;

pub fn lsr(memory: &[Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) -> u8
{
    let byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a
    };

    let mut extra_bit: bool = false;
    // do logical shift right here
    reg.setCarry(extra_bit);

    return cycles;
}