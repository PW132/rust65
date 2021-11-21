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
    pub last_op: u8,
    pub debug_text: bool,
    pub clock_speed: u32
}

impl CpuStatus
{
    pub fn new(speed: u32) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, last_op: 0, debug_text: false, clock_speed: speed}
    }


    pub fn setCarry(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1 } else { self.sr &= !0b1 }
    }

    pub fn setZero(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10 } else { self.sr &= !0b10 }
    }

    pub fn setInterrupt(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b100 } else { self.sr &= !0b100 }
    }

    pub fn setDecimal(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000 } else { self.sr &= !0b1000 }
    }

    pub fn setBreak(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10000 } else { self.sr &= !0b10000 }
    }

    pub fn setOverflow(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000000 } else { self.sr &= !0b1000000 }
    }

    pub fn setNegative(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10000000 } else { self.sr &= !0b10000000 }
    }
}

pub fn status_report(reg: &CpuStatus)
{
    println!("Current CPU status:");
    println!("Last Opcode: {:#04x} X: {:#04x} Y: {:#04x} A: {:#04x} SP: {:#04x} SR: {:#010b} PC: {:#06x}", reg.last_op, reg.x, reg.y, reg.a, reg.sp, reg.sr, reg.pc)
}


pub fn execute<'a>(memory: &mut [Segment], reg: &'a mut CpuStatus) -> Result<u8, String> //runs a single CPU instruction, returns errors if there are any
{
    let mut cycles: u8 = 0;
    let mut addr: u16 = 0;

    if reg.pc == 0xfffc //do we need to reset the CPU?
    {
        let lo_byte : u8 = bus::read(&memory,0xfffc); //retrieve reset vector from ROM
        let hi_byte : u8 = bus::read(&memory,0xfffd);

        reg.pc = lo_byte as u16 + (hi_byte as u16 * 256); //set new program counter at reset routine
        
        cycles += 7;

        if reg.debug_text { println!("Starting program execution at {:#06x}", reg.pc) }
    }

    let opcode: u8 = bus::read(&memory, reg.pc); //get the current opcode
    reg.last_op = opcode;

    reg.pc += 1; 

    match opcode //which instruction is it?
    {
        //Clear Flag Instructions
        0x18 => {cycles += 2; reg.setCarry(false)}, //CLC
        0xd8 => {cycles += 2; reg.setDecimal(false)}, //CLD
        0x58 => {cycles += 2; reg.setInterrupt(false)}, //CLI
        0xb8 => {cycles += 2; reg.setOverflow(false)} //CLV


        //Jump
        0x4c => {addr = bus::absolute(memory, reg); cycles += op::jmp(memory, reg, 3, addr)}, //JMP Absolute
        0x6c => {addr = bus::indirect(memory, reg); cycles += op::jmp(memory, reg, 5, addr)}, //JMP Indirect


        //Load A
        0xa9 => {cycles += op::lda(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDA Immediate
        0xa5 => {addr = bus::zp(memory, reg); cycles += op::lda(memory, reg, 3, addr)}, //LDA ZP
        0xb5 => {addr = bus::zp_x(memory, reg); cycles += op::lda(memory, reg, 4, addr)}, //LDA ZP,X
        0xad => {addr = bus::absolute(memory, reg); cycles += op::lda(memory, reg, 4, addr)}, //LDA Absolute
        //LDA Absolute,X
        //LDA Absolute,Y
        //LDA Indirect,X
        //LDA Indirect,Y


        //Load X
        0xa2 => {cycles += op::ldx(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDX Immediate
        0xa6 => {addr = bus::zp(memory, reg); cycles += op::ldx(memory, reg, 3, addr)}, //LDX ZP
        0xb6 => {addr = bus::zp_y(memory, reg); cycles += op::ldx(memory, reg, 4, addr)}, //LDX ZP,Y
        0xae => {addr = bus::absolute(memory, reg); cycles += op::ldx(memory, reg, 4, addr)}, //LDX Absolute


        //Load Y
        0xa0 => {cycles += op::ldy(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDY Immediate
        0xa4 => {addr = bus::zp(memory, reg); cycles += op::ldy(memory, reg, 3, addr)}, //LDY ZP
        0xb4 => {addr = bus::zp_x(memory, reg); cycles += op::ldy(memory, reg, 4, addr)}, //LDY ZP,X
        0xac => {addr = bus::absolute(memory, reg); cycles += op::ldy(memory, reg, 4, addr)}, //LDY Absolute


        //Logical Shift Right
        0x4a => {cycles += op::lsr(memory, reg, 2, None); reg.pc += 1;}, //LSR A
        0x46 => {addr = bus::zp(memory, reg); cycles += op::lsr(memory, reg, 5, Some(addr));}, //LSR ZP
        0x56 => {addr = bus::zp_x(memory, reg); cycles += op::lsr(memory, reg, 6, Some(addr));}, //LSR ZP,X
        0x4e => {addr = bus::absolute(memory, reg); cycles += op::lsr(memory, reg, 6, Some(addr));}, //LSR Absolute
        //LSR Absolute,X


        //No Operation
        0xea => {cycles += 2} //NOP


        //Set Flag Instructions
        0x38 => {cycles += 2; reg.setCarry(true)}, //SEC
        0xf8 => {cycles += 2; reg.setDecimal(true)}, //SED
        0x78 => {cycles += 2; reg.setInterrupt(true)}, //SEI


        //Store A
        0x85 => {addr = bus::zp(memory, reg); cycles += op::sta(memory, reg, 3, addr)}, //STA ZP
        0x95 => {addr = bus::zp_x(memory, reg); cycles += op::sta(memory, reg, 4, addr)}, //STA ZP,X
        0x8d => {addr = bus::absolute(memory, reg); cycles += op::sta(memory, reg, 4, addr)}, //STA Absolute
        //STA Absolute,X
        //STA Absolute,Y
        //STA Indirect,X
        //STA Indirect,Y


        //Store X
        0x86 => {addr = bus::zp(memory, reg); cycles += op::stx(memory, reg, 3, addr)}, //STA ZP
        0x96 => {addr = bus::zp_x(memory, reg); cycles += op::stx(memory, reg, 4, addr)}, //STA ZP,X
        0x8e => {addr = bus::absolute(memory, reg); cycles += op::stx(memory, reg, 4, addr)}, //STA Absolute

        
        //Store Y
        0x84 => {addr = bus::zp(memory, reg); cycles += op::sty(memory, reg, 3, addr)}, //STA ZP
        0x94 => {addr = bus::zp_x(memory, reg); cycles += op::sty(memory, reg, 4, addr)}, //STA ZP,X
        0x8c => {addr = bus::absolute(memory, reg); cycles += op::sty(memory, reg, 4, addr)}, //STA Absolute


        other => return Err(format!("Unrecognized opcode {:#04x}! Halting execution...", other)) //whoops! invalid opcode
    }

    Ok(cycles)
}