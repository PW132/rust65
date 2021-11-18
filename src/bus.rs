use crate::cpu::CpuStatus;
pub struct Segment<'a>
{
    pub data: &'a mut [u8],
    pub start_addr: u16,
    pub write_enabled: bool,
    pub read_enabled: bool
}

pub fn absolute(memspace: &[Segment], reg: &mut CpuStatus) -> u16
{
    let lo_byte: u8;
    let hi_byte: u8;
    let mut o_addr: u16 = 0;

    lo_byte = read(memspace, reg.pc);
    reg.pc += 1;
    hi_byte = read(memspace, reg.pc);
    reg.pc += 1;

    o_addr += hi_byte as u16;
    o_addr <<= 8;
    o_addr += lo_byte as u16;

    return o_addr;
}

pub fn zp(memspace: &[Segment], reg: &mut CpuStatus) -> u16
{
    let o_addr: u16;

    o_addr = read(memspace, reg.pc) as u16;
    reg.pc += 1;

    return o_addr;
}

pub fn read(memspace: &[Segment], addr: u16) -> u8 //bus arbitration for reading bytes
{
    let mut read_byte: u8 = 0;
    for bank in memspace
    {
        if addr >= bank.start_addr && addr < (bank.data.len() as u16 + bank.start_addr)
        {
            if bank.read_enabled
            {
                read_byte = bank.data[(addr - bank.start_addr) as usize];
                break;
            }
        }   
    }
    return read_byte;
}


pub fn write(memspace: &mut[Segment], addr: u16, data: u8) //bus arbitration for writing bytes
{
    for bank in memspace
    {
        if addr >= bank.start_addr && addr < (bank.data.len() as u16 + bank.start_addr)
        {
            if bank.write_enabled
            {
                bank.data[(addr - bank.start_addr) as usize] = data;
                break;
            }
        }
    }

    return;
}


pub fn push_stack(memory: &mut[Segment], reg: &mut CpuStatus, data: u8) //push a byte onto the stack and update the pointer
{
    if reg.debug_text { print!("Pushing {:#04x} onto stack... ", data) }

    reg.sp = reg.sp.wrapping_sub(1);

    if reg.debug_text 
    {
        if reg.sp == 0 { println!("stack overflow!") }
        else { println!("push succeeded") }
    }

    write(memory, reg.sp as u16 + 0x101, data)
}


pub fn pull_stack(memory: &mut[Segment], reg: &mut CpuStatus) -> u8  //pull a byte from the stack and update the pointer
{
    let pulled: u8 = read(memory, reg.sp as u16 + 0x101);

    if reg.debug_text { print!("Pulling {:#04x} from stack... ", pulled) }

    reg.sp = reg.sp.wrapping_add(1);

    if reg.debug_text 
    {
        if reg.sp == 0 { println!("stack underflow!") }
        else { println!("pull succeeded") }
    }

    return pulled;
}