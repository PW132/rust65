use crate::bus;
use crate::bus::Segment;
use crate::op;
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

impl CpuStatus
{
    pub fn new(speed: u32) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, debug_text: false, clock_speed: speed}
    }


    pub fn setCarry(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b1;
        }
        else 
        {
            self.sr &= !0b1;
        }
    }


    pub fn setZero(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b10;
        }
        else 
        {
            self.sr &= !0b10;
        }
    }


    pub fn setInterrupt(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b100;
        }
        else 
        {
            self.sr &= !0b100;
        }
    }


    pub fn setDecimal(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b1000;
        }
        else 
        {
            self.sr &= !0b1000;
        }
    }


    pub fn setOverflow(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b1000000;
        }
        else 
        {
            self.sr &= !0b1000000;
        }
    }


    pub fn setNegative(&mut self, flag: bool)
    {
        if flag
        {
            self.sr |= 0b10000000;
        }
        else 
        {
            self.sr &= !0b10000000;
        }
    }
}

pub fn status_report(reg: &CpuStatus)
{
    println!("Current CPU status:");
    println!("X: {:#04x} Y: {:#04x} A: {:#04x} SP: {:#04x} SR: {:#010b} PC: {:#06x}", reg.x, reg.y, reg.a, reg.sp, reg.sr, reg.pc)
}


pub fn execute<'a>(memory: &[Segment], reg: &'a mut CpuStatus) -> Result<u8, String> //runs a single CPU instruction, returns errors if there are any
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

    Ok(0)
}