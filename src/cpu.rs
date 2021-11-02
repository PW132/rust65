use crate::bus;
pub struct CpuStatus //contains the registers of the CPU, the clock speed, and other settings.
{
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sr: u8,
    pub sp: u8,
    pub debug_text: bool,
    pub clock_speed: u32
}

pub fn status_report(reg: &CpuStatus)
{
    println!("Current CPU status:");
    println!("X: {:#04x} Y: {:#04x} A: {:#04x} SP: {:#04x} SR: {:#010b} PC: {:#06x}", reg.x, reg.y, reg.a, reg.sp, reg.sr, reg.pc)
}

pub fn execute<'a>(memory: &'a mut[u8; 0xffff], reg: &'a mut CpuStatus) -> Result<bool, String> //runs a single CPU instruction, returns errors if there are any
{
    if reg.pc == 0xfffc //do we need to reset the CPU?
    {
        let lo_byte : u8 = bus::read(&memory,0xfffc); //retrieve reset vector from ROM
        let hi_byte : u8 = bus::read(&memory,0xfffd);

        reg.pc = lo_byte as u16 + (hi_byte as u16 * 256); //set new program counter at reset routine
        
        if reg.debug_text { println!("Starting program execution at {:#06x}", reg.pc) }
    }

    let opcode: u8 = bus::read(&memory, reg.pc); //get the current opcode

    match opcode //which instruction is it?
    {
        1 => println!("eughh"),
        other => return Err(format!("Unrecognized opcode {:#04x}! Halting execution...", other)) //whoops! invalid opcode
    }

    Ok(true)
}

pub fn push_stack(memory: &mut[u8; 0xffff], reg: &mut CpuStatus, data: u8) //push a byte onto the stack and update the pointer
{
    if reg.debug_text { print!("Pushing {:#04x} onto stack... ", data) }

    reg.sp -= 1;

    if reg.debug_text 
    {
        if reg.sp == 0 { println!("stack overflow!") }
        else { println!("push succeeded") }
    }

    bus::write(memory, reg.sp as u16 + 0x101, data)
}

pub fn pull_stack(memory: &mut[u8; 0xffff], reg: &mut CpuStatus) -> u8  //pull a byte from the stack and update the pointer
{
    let pulled: u8 = bus::read(memory, reg.sp as u16 + 0x101);

    if reg.debug_text { print!("Pulling {:#04x} from stack... ", pulled) }

    reg.sp += 1;

    if reg.debug_text 
    {
        if reg.sp == 0 { println!("stack underflow!") }
        else { println!("pull succeeded") }
    }

    return pulled;
}
