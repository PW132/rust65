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
    pub cycles_used: u8,
    pub debug_text: bool,
    pub clock_speed: u32
}

impl CpuStatus
{
    pub fn new(speed: u32) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, last_op: 0, cycles_used: 0, debug_text: false, clock_speed: speed}
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
    reg.cycles_used = 0;
    let mut addr: u16 = 0;

    if reg.pc == 0xfffc //do we need to reset the CPU?
    {
        let lo_byte : u8 = bus::read(&memory,0xfffc); //retrieve reset vector from ROM
        let hi_byte : u8 = bus::read(&memory,0xfffd);

        reg.pc = lo_byte as u16 + ((hi_byte as u16) << 8); //set new program counter at reset routine
        
        reg.cycles_used += 7;

        if reg.debug_text { println!("Starting program execution at {:#06x}", reg.pc) }
    }

    let opcode: u8 = bus::read(&memory, reg.pc); //get the current opcode
    reg.last_op = opcode;

    reg.pc += 1; 

    match opcode //which instruction is it?
    {
        //Clear Flag Instructions
        0x18 => {reg.cycles_used += 2; reg.setCarry(false)}, //CLC
        0xd8 => {reg.cycles_used += 2; reg.setDecimal(false)}, //CLD
        0x58 => {reg.cycles_used += 2; reg.setInterrupt(false)}, //CLI
        0xb8 => {reg.cycles_used += 2; reg.setOverflow(false)} //CLV


        //Jump
        0x4c => {addr = bus::absolute(memory, reg); reg.cycles_used += op::jmp(memory, reg, 3, addr)}, //JMP Absolute
        0x6c => {addr = bus::indirect(memory, reg); reg.cycles_used += op::jmp(memory, reg, 5, addr)}, //JMP Indirect


        //Load A
        0xa9 => {reg.cycles_used += op::lda(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDA Immediate
        0xa5 => {addr = bus::zp(memory, reg); reg.cycles_used += op::lda(memory, reg, 3, addr)}, //LDA ZP
        0xb5 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::lda(memory, reg, 4, addr)}, //LDA ZP,X
        0xad => {addr = bus::absolute(memory, reg); reg.cycles_used += op::lda(memory, reg, 4, addr)}, //LDA Absolute
        0xbd => {addr = bus::absolute_x(memory, reg); reg.cycles_used += op::lda(memory, reg, 4, addr)}, //LDA Absolute,X
        0xb9 => {addr = bus::absolute_y(memory, reg); reg.cycles_used += op::lda(memory, reg, 4, addr)}, //LDA Absolute,Y
        //LDA Indirect,X
        //LDA Indirect,Y


        //Load X
        0xa2 => {reg.cycles_used += op::ldx(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDX Immediate
        0xa6 => {addr = bus::zp(memory, reg); reg.cycles_used += op::ldx(memory, reg, 3, addr)}, //LDX ZP
        0xb6 => {addr = bus::zp_y(memory, reg); reg.cycles_used += op::ldx(memory, reg, 4, addr)}, //LDX ZP,Y
        0xae => {addr = bus::absolute(memory, reg); reg.cycles_used += op::ldx(memory, reg, 4, addr)}, //LDX Absolute
        0xbe => {addr = bus::absolute_y(memory, reg); reg.cycles_used += op::ldx(memory, reg, 4, addr)}, //LDX Absolute,Y


        //Load Y
        0xa0 => {reg.cycles_used += op::ldy(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDY Immediate
        0xa4 => {addr = bus::zp(memory, reg); reg.cycles_used += op::ldy(memory, reg, 3, addr)}, //LDY ZP
        0xb4 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::ldy(memory, reg, 4, addr)}, //LDY ZP,X
        0xac => {addr = bus::absolute(memory, reg); reg.cycles_used += op::ldy(memory, reg, 4, addr)}, //LDY Absolute
        0xbc => {addr = bus::absolute_x(memory, reg); reg.cycles_used += op::ldy(memory, reg, 4, addr)}, //LDY Absolute,X


        //Logical Shift Right
        0x4a => {reg.cycles_used += op::lsr(memory, reg, 2, None); reg.pc += 1;}, //LSR A
        0x46 => {addr = bus::zp(memory, reg); reg.cycles_used += op::lsr(memory, reg, 5, Some(addr))}, //LSR ZP
        0x56 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::lsr(memory, reg, 6, Some(addr))}, //LSR ZP,X
        0x4e => {addr = bus::absolute(memory, reg); reg.cycles_used += op::lsr(memory, reg, 6, Some(addr))}, //LSR Absolute
        0x5e => {addr = bus::absolute_x(memory, reg); reg.cycles_used += op::lsr(memory, reg, 7, Some(addr))}, //LSR Absolute,X


        //No Operation
        0xea => {reg.cycles_used += 2} //NOP


        //Set Flag Instructions
        0x38 => {reg.cycles_used += 2; reg.setCarry(true)}, //SEC
        0xf8 => {reg.cycles_used += 2; reg.setDecimal(true)}, //SED
        0x78 => {reg.cycles_used += 2; reg.setInterrupt(true)}, //SEI


        //Store A
        0x85 => {addr = bus::zp(memory, reg); reg.cycles_used += op::sta(memory, reg, 3, addr)}, //STA ZP
        0x95 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::sta(memory, reg, 4, addr)}, //STA ZP,X
        0x8d => {addr = bus::absolute(memory, reg); reg.cycles_used += op::sta(memory, reg, 4, addr)}, //STA Absolute
        0x9d => {addr = bus::absolute_x(memory, reg); reg.cycles_used += op::sta(memory, reg, 5, addr)}, //STA Absolute,X
        0x99 => {addr = bus::absolute_y(memory, reg); reg.cycles_used += op::sta(memory, reg, 5, addr)}, //STA Absolute,Y
        //STA Indirect,X
        //STA Indirect,Y


        //Store X
        0x86 => {addr = bus::zp(memory, reg); reg.cycles_used += op::stx(memory, reg, 3, addr)}, //STA ZP
        0x96 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::stx(memory, reg, 4, addr)}, //STA ZP,X
        0x8e => {addr = bus::absolute(memory, reg); reg.cycles_used += op::stx(memory, reg, 4, addr)}, //STA Absolute

        
        //Store Y
        0x84 => {addr = bus::zp(memory, reg); reg.cycles_used += op::sty(memory, reg, 3, addr)}, //STA ZP
        0x94 => {addr = bus::zp_x(memory, reg); reg.cycles_used += op::sty(memory, reg, 4, addr)}, //STA ZP,X
        0x8c => {addr = bus::absolute(memory, reg); reg.cycles_used += op::sty(memory, reg, 4, addr)}, //STA Absolute


        other => return Err(format!("Unrecognized opcode {:#04x}! Halting execution...", other)) //whoops! invalid opcode
    }

    Ok(reg.cycles_used)
}