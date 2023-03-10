use crate::cpu::CpuStatus;

pub struct Segment<'a> {
    pub data: &'a mut [u8],
    pub start_addr: u16,
    pub end_addr: u16,
    pub write_enabled: bool,
    pub read_enabled: bool,
}

impl Segment<'_> {
    pub fn new(
        i_data: &mut [u8],
        i_start_addr: u16,
        i_write_enabled: bool,
        i_read_enabled: bool,
    ) -> Segment {
        let length = i_data.len();
        Segment {
            data: i_data,
            start_addr: i_start_addr,
            end_addr: i_start_addr + length as u16,
            write_enabled: i_write_enabled,
            read_enabled: i_read_enabled,
        }
    }
}

pub fn absolute(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //Absolute
{
    let lo_byte: u8;
    let hi_byte: u8;
    let o_addr: u16;

    lo_byte = read(memspace, reg.pc);
    reg.pc += 1;
    hi_byte = read(memspace, reg.pc);
    reg.pc += 1;

    o_addr = ((hi_byte as u16) << 8) + lo_byte as u16;

    return o_addr;
}

pub fn absolute_x(memspace: &mut [Segment], reg: &mut CpuStatus, wrap_check: bool) -> u16 //Absolute + X
{
    let lo_byte: u8;
    let hi_byte: u8;
    let addr: u16;
    let o_addr: u16;

    lo_byte = read(memspace, reg.pc);
    reg.pc += 1;
    hi_byte = read(memspace, reg.pc);
    reg.pc += 1;

    addr = ((hi_byte as u16) << 8) + lo_byte as u16;

    o_addr = addr.wrapping_add(reg.x as u16);

    if addr & 0x100 != o_addr & 0x100 && wrap_check {
        reg.cycles_used += 1;
    }

    return o_addr;
}

pub fn absolute_y(memspace: &mut [Segment], reg: &mut CpuStatus, wrap_check: bool) -> u16 //Absolute + Y
{
    let lo_byte: u8;
    let hi_byte: u8;
    let addr: u16;
    let o_addr: u16;

    lo_byte = read(memspace, reg.pc);
    reg.pc += 1;
    hi_byte = read(memspace, reg.pc);
    reg.pc += 1;

    addr = ((hi_byte as u16) << 8) + lo_byte as u16;

    o_addr = addr.wrapping_add(reg.y as u16);

    if addr & 0x100 != o_addr & 0x100 && wrap_check {
        reg.cycles_used += 1;
    }

    return o_addr;
}

pub fn zp(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //Zero Page
{
    let o_addr: u16;

    o_addr = read(memspace, reg.pc) as u16;
    reg.pc += 1;

    return o_addr;
}

pub fn zp_x(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //Zero Page + X
{
    let o_addr: u8;

    o_addr = read(memspace, reg.pc).wrapping_add(reg.x);
    reg.pc += 1;

    return o_addr as u16;
}

pub fn zp_y(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //Zero Page + Y
{
    let o_addr: u8;

    o_addr = read(memspace, reg.pc).wrapping_add(reg.y);
    reg.pc += 1;

    return o_addr as u16;
}

pub fn indirect(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //indirect addressing, only used by JMP. Kinda jank to implement.
{
    let lo_byte: u8;
    let hi_byte: u8;

    let mut i_addr: u16;
    let mut i_addr2: u16;
    let mut o_addr: u16;

    lo_byte = read(memspace, reg.pc);
    reg.pc += 1;
    hi_byte = read(memspace, reg.pc);
    reg.pc += 1;

    i_addr = (hi_byte as u16) << 8;
    i_addr2 = i_addr;
    i_addr += lo_byte as u16;
    i_addr2 += lo_byte.wrapping_add(1) as u16; //We use wrapping_add here to mimic the NMOS 6502 bug where indirect jumps don't work right at page boundaries

    o_addr = read(memspace, i_addr) as u16;
    o_addr += (read(memspace, i_addr2) as u16) << 8;

    return o_addr;
}

pub fn indirect_x(memspace: &mut [Segment], reg: &mut CpuStatus) -> u16 //Indirect + X.
{
    let zp_addr: u8;
    let lo_byte: u8;
    let hi_byte: u8;

    let o_addr: u16;

    zp_addr = read(memspace, reg.pc).wrapping_add(reg.x);
    reg.pc += 1;
    lo_byte = read(memspace, zp_addr as u16);
    hi_byte = read(memspace, (zp_addr as u16).wrapping_add(1));

    o_addr = ((hi_byte as u16) << 8) + lo_byte as u16;

    return o_addr;
}

pub fn indirect_y(memspace: &mut [Segment], reg: &mut CpuStatus, wrap_check: bool) -> u16 //Indirect + Y. Significantly different to Indirect + X in operation.
{
    let zp_addr: u8;
    let lo_byte: u8;
    let hi_byte: u8;

    let mut i_addr: u16 = 0;
    let o_addr: u16;

    zp_addr = read(memspace, reg.pc);
    reg.pc += 1;
    lo_byte = read(memspace, zp_addr as u16);
    hi_byte = read(memspace, (zp_addr as u16).wrapping_add(1));

    i_addr += ((hi_byte as u16) << 8) + lo_byte as u16;

    o_addr = i_addr.wrapping_add(reg.y as u16);

    if i_addr & 0x100 != o_addr & 0x100 && wrap_check {
        reg.cycles_used += 1;
    }

    return o_addr;
}

pub fn read(memspace: &mut [Segment], addr: u16) -> u8 //bus arbitration for reading bytes
{
    match addr //put special effects that happen upon a read from a certain address here
    {
        0xd010 => memspace[2].data[1] &= !0b10000000, //when reading PIA port A input register, clear bit 7 of the output register
        _ => ()
    }

    for bank in memspace.iter() {
        if addr >= bank.start_addr && addr < bank.end_addr {
            if bank.read_enabled {
                return bank.data[(addr - bank.start_addr) as usize];
            }
        }
    }

    println!("Attempt to read from unmapped address {:#06x}!", addr);
    return 0xAA;
}

pub fn write(memspace: &mut [Segment], addr: u16, data: u8) //bus arbitration for writing bytes
{
    for bank in memspace.iter_mut() {
        if addr >= bank.start_addr && addr < bank.end_addr {
            if bank.write_enabled {
                bank.data[(addr - bank.start_addr) as usize] = data;
                break;
            }
        }
    }

    match addr //put special effects that happen upon a write to a certain address here
    {
        0xd012 => memspace[2].data[2] |= 0b10000000, //when writing to PIA port B output register, set bit 7 of the input register
        _ => ()
    }

    return;
}

pub fn push_stack(memory: &mut [Segment], reg: &mut CpuStatus, data: u8)
//push a byte onto the stack and update the pointer
{
    if reg.debug_text {
        print!("Pushing {:#04x} onto stack... ", data)
    }

    reg.sp = reg.sp.wrapping_add(1);

    if reg.debug_text {
        if reg.sp == 0 {
            println!("stack overflow!")
        } else {
            println!("push succeeded")
        }
    }

    write(memory, reg.sp as u16 + 0x101, data)
}

pub fn pull_stack(memory: &mut [Segment], reg: &mut CpuStatus) -> u8 //pull a byte from the stack and update the pointer
{
    let pulled: u8 = read(memory, reg.sp as u16 + 0x101);

    if reg.debug_text {
        print!("Pulling {:#04x} from stack... ", pulled)
    }

    reg.sp = reg.sp.wrapping_sub(1);

    if reg.debug_text {
        if reg.sp == 0xff {
            println!("stack underflow!")
        } else {
            println!("pull succeeded")
        }
    }

    return pulled;
}
