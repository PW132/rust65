use crate::bus;
use crate::bus::Segment;
use crate::cpu::CpuStatus;

pub fn adc(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    let result: u8;

    match reg.decimal_flag()
    {
        true =>
        {
            if reg.debug_text {println!("Attempted to ADC in decimal mode! This is not implemented!")};
            result = reg.a;
        }
        false =>
        {
            result = reg.a.wrapping_add(byte.wrapping_add(reg.carry_flag() as u8));
        }
    }

    reg.set_carry(result < reg.a);
    reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (result & 0x80 != byte & 0x80));
    reg.set_zero(result == 0);
    reg.set_negative(result > 0x7f);

    reg.a = result;

    reg.cycles_used += cycles
}

pub fn and(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.a &= byte;

    reg.set_negative(reg.a > 0x7f);
    reg.set_zero(reg.a == 0);

    reg.cycles_used += cycles
}

pub fn asl(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) 
{
    let mut byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a,
    };

    reg.set_carry(0 != byte & 0b10000000);

    byte <<= 1;
    reg.set_negative(byte > 0x7f);
    reg.set_zero(byte == 0);

    match i_addr {
        Some(v) => bus::write(memory, v, byte),
        None => reg.a = byte,
    };

    reg.cycles_used += cycles
}

pub fn bit(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.set_negative(0 != byte & 0b10000000);
    reg.set_overflow(0 != byte & 0b1000000);
    reg.set_zero(0 == byte & reg.a);

    reg.cycles_used += cycles
}

pub fn branch(memory: &mut [Segment], reg: &mut CpuStatus, flag: bool)
//basis for all branch instructions
{
    reg.cycles_used += 2; //use two cycles no matter what

    if flag
    //if the flag we tested is true and we should branch:
    {
        reg.cycles_used += 1; //use another cycle

        let old_pc: u16 = reg.pc; //store the old program counter to compare against later
        let offset: u8 = bus::read(memory, reg.pc); //read the next byte to get the offset
        reg.pc += 1;

        if offset < 127 //if the byte is positive, move PC forward that many bytes
        {
            reg.pc += offset as u16;
        } 
        else //if the byte is negative, invert all the bits of the offset to convert it to positive again and then subtract from the PC
        {
            reg.pc -= !offset as u16 + 1;
        }

        if old_pc & 0x100 != reg.pc & 0x100
        //use another cycle if we crossed a page boundary
        {
            reg.cycles_used += 1;
        }

        if reg.debug_text {
            println!(
                "Branching from address {:#06x} to {:#06x}...",
                old_pc, reg.pc
            )
        }
    } else
    //if the flag is false then just increment the program counter and do nothing else
    {
        reg.pc += 1;

        if reg.debug_text {
            println!("Branch condition evaluated but not taken.")
        }
    }
}

pub fn cmp(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.set_carry(reg.a >= byte);
    reg.set_zero(reg.a == byte);
    reg.set_negative(reg.a.wrapping_sub(byte) > 0x7f);

    reg.cycles_used += cycles
}

pub fn cpx(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.set_carry(reg.x >= byte);
    reg.set_zero(reg.x == byte);
    reg.set_negative(reg.x.wrapping_sub(byte) > 0x7f);

    reg.cycles_used += cycles
}

pub fn cpy(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.set_carry(reg.y >= byte);
    reg.set_zero(reg.y == byte);
    reg.set_negative(reg.y.wrapping_sub(byte) > 0x7f);

    reg.cycles_used += cycles
}

pub fn dec(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let mut byte: u8 = bus::read(memory, i_addr);

    byte = byte.wrapping_sub(1);

    reg.set_negative(byte > 0x7f);
    reg.set_zero(byte == 0);

    bus::write(memory, i_addr, byte);

    reg.cycles_used += cycles
}

pub fn eor(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.a ^= byte;

    reg.set_negative(reg.a > 0x7f);
    reg.set_zero(reg.a == 0);

    reg.cycles_used += cycles
}

pub fn inc(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let mut byte: u8 = bus::read(memory, i_addr);

    byte = byte.wrapping_add(1);

    reg.set_negative(byte > 0x7f);
    reg.set_zero(byte == 0);

    bus::write(memory, i_addr, byte);

    reg.cycles_used += cycles
}

pub fn jmp(reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    reg.pc = i_addr;

    reg.cycles_used += cycles;

    if reg.debug_text {
        println!("JMP to new address {:#06x}...", reg.pc)
    }
}

pub fn jsr(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let return_addr: u16 = reg.pc - 1;
    let return_byte_lo: u8 = (return_addr & 0xff) as u8;
    let return_byte_hi: u8 = ((return_addr & 0xff00) >> 8) as u8;

    bus::push_stack(memory, reg, return_byte_hi);
    bus::push_stack(memory, reg, return_byte_lo);

    reg.pc = i_addr;

    reg.cycles_used += cycles;

    if reg.debug_text {
        println!("JSR to new address {:#06x}...", reg.pc)
    }
}

pub fn lda(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.a = byte;

    reg.set_negative(reg.a > 0x7f);
    reg.set_zero(reg.a == 0);

    reg.cycles_used += cycles
}

pub fn ldx(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.x = byte;

    reg.set_negative(reg.x > 0x7f);
    reg.set_zero(reg.x == 0);

    reg.cycles_used += cycles
}

pub fn ldy(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8;
    byte = bus::read(memory, i_addr);

    reg.y = byte;

    reg.set_negative(reg.y > 0x7f);
    reg.set_zero(reg.y == 0);

    reg.cycles_used += cycles
}

pub fn lsr(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) 
{
    let mut byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a,
    };

    reg.set_carry(0 != byte & 0b1);

    byte >>= 1;
    reg.set_negative(false);
    reg.set_zero(byte == 0);

    match i_addr {
        Some(v) => bus::write(memory, v, byte),
        None => reg.a = byte,
    };

    reg.cycles_used += cycles
}

pub fn ora(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    reg.a |= byte;

    reg.set_negative(reg.a > 0x7f);
    reg.set_zero(reg.a == 0);

    reg.cycles_used += cycles
}

pub fn rol(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) 
{
    let mut byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a,
    };

    let new_carry = 0 != byte & 0b10000000;

    byte <<= 1;

    if reg.carry_flag() { byte |= 0b1 }

    reg.set_carry(new_carry);
    reg.set_negative(byte > 0x7f);
    reg.set_zero(byte == 0);

    match i_addr {
        Some(v) => bus::write(memory, v, byte),
        None => reg.a = byte,
    };

    reg.cycles_used += cycles
}

pub fn ror(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: Option<u16>) 
{
    let mut byte: u8;

    match i_addr {
        Some(v) => byte = bus::read(memory, v),
        None => byte = reg.a,
    };

    let new_carry = 0 != byte & 0b1;

    byte >>= 1;

    if reg.carry_flag() { byte |= 0b10000000 }

    reg.set_carry(new_carry);
    reg.set_negative(byte > 0x7f);
    reg.set_zero(byte == 0);

    match i_addr {
        Some(v) => bus::write(memory, v, byte),
        None => reg.a = byte,
    };

    reg.cycles_used += cycles
}

pub fn rts(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8) 
{
    let return_byte_lo: u8 = bus::pull_stack(memory, reg);
    let return_byte_hi: u8 = bus::pull_stack(memory, reg);

    reg.pc = (((return_byte_hi as u16) << 8) + return_byte_lo as u16) + 1;

    reg.cycles_used += cycles
}

pub fn sbc(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = !bus::read(memory, i_addr);

    let result: u8;

    match reg.decimal_flag()
    {
        true =>
        {
            if reg.debug_text {println!("Attempted to SBC in decimal mode! This is not implemented!")};
            result = reg.a;
        }
        false =>
        {
            result = reg.a.wrapping_add(byte.wrapping_add(reg.carry_flag() as u8));
        }
    }

    reg.set_carry(result < reg.a);
    reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (result & 0x80 != byte & 0x80));
    reg.set_zero(result == 0);
    reg.set_negative(result > 0x7f);

    reg.a = result;

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

pub fn transfer(reg: &mut CpuStatus, origin: char, destination: char) 
{
    let val: u8;

    match origin {
        'a' => val = reg.a,
        'x' => val = reg.x,
        'y' => val = reg.y,
        's' => val = reg.sp,
        _ => panic!("Invalid origin argument to op::transfer \n")
    };

    match destination {
        'a' => reg.a = val,
        'x' => reg.x = val,
        'y' => reg.y = val,
        's' => reg.sp = val,
        _ => panic!("Invalid destination argument to op::transfer \n")
    };

    reg.cycles_used += 2;

    if destination != 's'
    {
        reg.set_negative(val > 0x7f);
        reg.set_zero(0 == val);
    }
}
