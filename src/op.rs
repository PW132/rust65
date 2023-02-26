use crate::bus;
use crate::bus::Segment;
use crate::cpu::CpuStatus;

pub fn adc(memory: &mut [Segment], reg: &mut CpuStatus, cycles: u8, i_addr: u16) 
{
    let byte: u8 = bus::read(memory, i_addr);

    let mut result: u16;

    match reg.decimal_flag() //is this a BCD operation?
    {
        true => //BCD add (this implementation is based upon Py6502's version)
        {
            let mut half_carry: bool = false;
            let mut hi_adjust: u8 = 0;
            let mut lo_adjust: u8 = 0;

            let mut lo_nibble: u8 = (reg.a & 0xf) + (byte & 0xf) + (reg.carry_flag() as u8); //low bits of A + low bits of byte + Carry flag
            
            if lo_nibble > 9
            {
                lo_adjust = 6;
                half_carry = true;
            }

            let mut hi_nibble: u8 = ( (reg.a >> 4) & 0xf ) + ( (byte>> 4) & 0xf ) + (half_carry as u8); //high bits of A + high bits of byte + Carry from low bits result

            reg.set_carry(hi_nibble > 9);
            if reg.carry_flag()
            {
                hi_adjust = 6;
            }

            //ALU result without decimal adjustments
            lo_nibble &= 0xf;
            hi_nibble &= 0xf;
            let alu_result: u8 = (hi_nibble << 4) + lo_nibble;

            reg.set_zero(alu_result == 0);
            reg.set_negative(alu_result > 0x7f);
            reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (alu_result & 0x80 != byte & 0x80));

            //Final A result with adjustment
            lo_nibble = (lo_nibble + lo_adjust) & 0xf;
            hi_nibble = (hi_nibble + hi_adjust) & 0xf;
            result = u16::from((hi_nibble << 4) + lo_nibble);
        }
        false => //Normal binary add
        {
            result = reg.a as u16 + byte as u16 + reg.carry_flag() as u16; // A + Byte + Carry

            reg.set_carry(result > 0xff);

            if reg.carry_flag()
            {
                result &= 0xff;
            }

            reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (result as u8 & 0x80 != byte & 0x80));
            reg.set_zero(result == 0);
            reg.set_negative(result > 0x7f);
        }
    }

    reg.a = result as u8;

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
    let byte: u8 = bus::read(memory, i_addr); //the only difference between add and subtract is using the inverse of the byte to be added!
    let c_byte = !byte;

    let mut result: u16;

    match reg.decimal_flag() 
    {
        true => //BCD
        {
            let mut half_carry: bool = false;
            let mut hi_adjust: u8 = 0;
            let mut lo_adjust: u8 = 0;

            let mut lo_nibble: u8 = (reg.a & 0xf) + (c_byte & 0xf) + (reg.carry_flag() as u8); 
            
            if lo_nibble > 9
            {
                lo_adjust = 6;
                half_carry = true;
            }

            let mut hi_nibble: u8 = ( (reg.a >> 4) & 0xf ) + ( (c_byte>> 4) & 0xf ) + (half_carry as u8); 

            reg.set_carry(hi_nibble > 9);
            if reg.carry_flag()
            {
                hi_adjust = 6;
            }

            //ALU result without decimal adjustments
            lo_nibble &= 0xf;
            hi_nibble &= 0xf;
            let alu_result: u8 = (hi_nibble << 4) + lo_nibble;

            reg.set_zero(alu_result == 0);
            reg.set_negative(alu_result > 0x7f);
            reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (alu_result & 0x80 != byte & 0x80));

            //Final A result with adjustment
            lo_nibble = (lo_nibble + lo_adjust) & 0xf;
            hi_nibble = (hi_nibble + hi_adjust) & 0xf;
            result = u16::from((hi_nibble << 4) + lo_nibble);
        }
        false => //Normal binary
        {
            result = reg.a as u16 + c_byte as u16 + reg.carry_flag() as u16; // A + Byte + Carry

            reg.set_carry(result > 0xff);

            if reg.carry_flag()
            {
                result &= 0xff;
            }

            reg.set_overflow((byte & 0x80 == reg.a & 0x80) && (result as u8 & 0x80 != byte & 0x80));
            reg.set_zero(result == 0);
            reg.set_negative(result > 0x7f);
        }
    }

    reg.a = result as u8;

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
