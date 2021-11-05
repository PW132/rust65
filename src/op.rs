use crate::cpu;
use crate::cpu::CpuStatus;
use crate::bus;
use crate::bus::Segment;

pub fn lsr(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) -> u8
{
    let mut byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a
    };

    reg.setCarry(0b1 & byte != 0);

    byte >>= 1;
    reg.setNegative(byte > 0x7f);
    reg.setZero(byte == 0);

    match i_addr {
        Some(v) => bus::write(memory, v, byte),
        None => reg.a = byte
    };

    return cycles;
}