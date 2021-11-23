use crate::cpu;
use crate::cpu::CpuStatus;
use crate::bus;
use crate::bus::Segment;


pub fn jmp(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    reg.pc = i_addr;

    reg.cycles_used += cycles
}


pub fn lda(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.a = byte;

    reg.setNegative(byte > 0x7f);
    reg.setZero(byte == 0);

    reg.cycles_used += cycles
}


pub fn ldx(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.x = byte;

    reg.setNegative(byte > 0x7f);
    reg.setZero(byte == 0);

    reg.cycles_used += cycles
}


pub fn ldy(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.y = byte;

    reg.setNegative(byte > 0x7f);
    reg.setZero(byte == 0);

    reg.cycles_used += cycles
}


pub fn sta(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    bus::write(memory, i_addr, reg.a);

    reg.cycles_used += cycles
}


pub fn stx(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    bus::write(memory, i_addr, reg.x);

    reg.cycles_used += cycles
}


pub fn sty(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16)
{
    bus::write(memory, i_addr, reg.y);

    reg.cycles_used += cycles
}


pub fn lsr(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>)
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

    reg.cycles_used += cycles;
}