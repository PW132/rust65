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
    pub reset: bool,
    pub debug_text: bool,
    pub clock_time: u64
}


impl CpuStatus
{
    pub fn new(speed: u64) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, last_op: 0, cycles_used: 0, reset: true, debug_text: false, clock_time: (1000000000 / speed)}
    }


    #[inline]
    pub fn carry_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1)
    }
    #[inline]
    pub fn set_carry(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1 } else { self.sr &= !0b1 }
    }


    #[inline]
    pub fn zero_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10)
    }
    #[inline]
    pub fn set_zero(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10 } else { self.sr &= !0b10 }
    }

    #[inline]
    pub fn interrupt_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b100)
    }
    #[inline]
    pub fn set_interrupt(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b100 } else { self.sr &= !0b100 }
    }

    #[inline]
    pub fn decimal_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1000)
    }
    #[inline]
    pub fn set_decimal(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000 } else { self.sr &= !0b1000 }
    }

    #[inline]
    pub fn break_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10000)
    }
    #[inline]
    pub fn set_break(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10000 } else { self.sr &= !0b10000 }
    }

    #[inline]
    pub fn overflow_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1000000)
    }
    #[inline]
    pub fn set_overflow(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000000 } else { self.sr &= !0b1000000 }
    }

    #[inline]
    pub fn negative_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10000000)
    }
    #[inline]
    pub fn set_negative(&mut self, flag: bool)
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
    let addr: u16;
    let flag: bool;

    if reg.reset                                                    //do we need to reset the CPU?
    {
        reg.pc = 0xfffc;

        let lo_byte : u8 = bus::read(memory,reg.pc); //retrieve reset vector from ROM
        reg.pc += 1;
        let hi_byte : u8 = bus::read(memory,reg.pc);

        reg.pc = lo_byte as u16 + ((hi_byte as u16) << 8);           //set new program counter at reset routine
        
        reg.cycles_used += 7;
        reg.reset = false;

        if reg.debug_text { println!("Starting program execution at {:#06x}", reg.pc) }
    }

    let opcode: u8 = bus::read(memory, reg.pc);        //get the current opcode
    reg.last_op = opcode;

    reg.pc += 1; 

    match opcode            //which instruction is it?
    {
        //Add With Carry

        //Arithmetic Shift Left

        //Bit Test

        //Branch Instructions
        0x10 => {flag = !reg.negative_flag(); op::branch(memory, reg, flag)}, //BPL Branch on PLus
        0x30 => {flag = reg.negative_flag(); op::branch(memory, reg, flag)}, //BMI Branch on MInus
        0x50 => {flag = !reg.overflow_flag(); op::branch(memory, reg, flag)}, //BVC Branch on oVerflow Clear
        0x70 => {flag = reg.overflow_flag(); op::branch(memory, reg, flag)}, //BVS Branch on oVerflow Set
        0x90 => {flag = !reg.carry_flag(); op::branch(memory, reg, flag)}, //BCC Branch on Carry Clear
        0xb0 => {flag = reg.carry_flag(); op::branch(memory, reg, flag)}, //BCS Branch on Carry Set
        0xd0 => {flag = !reg.zero_flag(); op::branch(memory, reg, flag)}, //BNE Branch on Not Equal
        0xf0 => {flag = reg.zero_flag(); op::branch(memory, reg, flag)}, //BEQ Branch on EQual

        //Break

        //Clear Flag Instructions
        0x18 => {reg.cycles_used += 2; reg.set_carry(false)}, //CLC
        0xd8 => {reg.cycles_used += 2; reg.set_decimal(false)}, //CLD
        0x58 => {reg.cycles_used += 2; reg.set_interrupt(false)}, //CLI
        0xb8 => {reg.cycles_used += 2; reg.set_overflow(false)}, //CLV


        //Compare with Accumulator
        0xc9 => {op::cmp(memory, reg, 2, reg.pc); reg.pc += 1}, //CMP Immediate
        0xc5 => {addr = bus::zp(memory, reg); op::cmp(memory, reg, 3, addr);}, //CMP ZP
        0xd5 => {addr = bus::zp_x(memory, reg); op::cmp(memory, reg, 4, addr);}, //CMP ZP,X
        0xcd => {addr = bus::absolute(memory, reg); op::cmp(memory, reg, 4, addr)}, //CMP Absolute
        0xdd => {addr = bus::absolute_x(memory, reg); op::cmp(memory, reg, 4, addr)}, //CMP Absolute,X
        0xd9 => {addr = bus::absolute_y(memory, reg); op::cmp(memory, reg, 4, addr)}, //CMP Absolute,Y
        //CMP Indirect,X
        //CMP Indirect,Y


        //Compare with X
        0xe0 => {op::cpx(memory, reg, 2, reg.pc); reg.pc += 1}, //CPX Immediate
        0xe4 => {addr = bus::zp(memory, reg); op::cpx(memory, reg, 3, addr);}, //CPX ZP
        0xec => {addr = bus::absolute(memory, reg); op::cpx(memory, reg, 4, addr);}, //CPX Absolute


        //Compare with Y
        0xc0 => {op::cpy(memory, reg, 2, reg.pc); reg.pc += 1}, //CPY Immediate
        0xc4 => {addr = bus::zp(memory, reg); op::cpy(memory, reg, 3, addr);}, //CPY ZP
        0xcc => {addr = bus::absolute(memory, reg); op::cpy(memory, reg, 4, addr);}, //CPY Absolute


        //Decrement Memory
        0xc6 => {addr = bus::zp(memory, reg); op::dec(memory, reg, 5, addr)}, //DEC ZP
        0xd6 => {addr = bus::zp_x(memory, reg); op::dec(memory, reg, 6, addr)}, //DEC ZP,X
        0xce => {addr = bus::absolute(memory, reg); op::dec(memory, reg, 6, addr)}, //DEC Absolute
        0xde => {addr = bus::absolute_x(memory, reg); op::dec(memory, reg, 7, addr)}, //DEC Absolute,X


        //Decrement X
        0xca => {reg.x = reg.x.wrapping_sub(1); reg.set_zero(reg.x == 0); reg.set_negative(reg.x > 0x7f); reg.cycles_used += 2;}, //DEX


        //Decrement Y
        0x88 => {reg.y = reg.y.wrapping_sub(1); reg.set_zero(reg.y == 0); reg.set_negative(reg.y > 0x7f);  reg.cycles_used += 2;}, //DEY


        //Exclusive OR


        //Increment Memory
        0xe6 => {addr = bus::zp(memory, reg); op::inc(memory, reg, 5, addr)}, //INC ZP
        0xf6 => {addr = bus::zp_x(memory, reg); op::inc(memory, reg, 6, addr)}, //INC ZP,X
        0xee => {addr = bus::absolute(memory, reg); op::inc(memory, reg, 6, addr)}, //INC Absolute
        0xfe => {addr = bus::absolute_x(memory, reg); op::inc(memory, reg, 7, addr)}, //INC Absolute,X


        //Increment X
        0xe8 => {reg.x = reg.x.wrapping_add(1); reg.set_zero(reg.x == 0); reg.set_negative(reg.x > 0x7f);  reg.cycles_used += 2;}, //INX


        //Increment Y
        0xc8 => {reg.x = reg.x.wrapping_add(1); reg.set_zero(reg.y == 0); reg.set_negative(reg.y > 0x7f);  reg.cycles_used += 2;}, //INY


        //Jump
        0x4c => {addr = bus::absolute(memory, reg); op::jmp(reg, 3, addr)}, //JMP Absolute
        0x6c => {addr = bus::indirect(memory, reg); op::jmp(reg, 5, addr)}, //JMP Indirect


        //Jump to Subroutine


        //Load A
        0xa9 => {op::lda(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDA Immediate
        0xa5 => {addr = bus::zp(memory, reg); op::lda(memory, reg, 3, addr)}, //LDA ZP
        0xb5 => {addr = bus::zp_x(memory, reg); op::lda(memory, reg, 4, addr)}, //LDA ZP,X
        0xad => {addr = bus::absolute(memory, reg); op::lda(memory, reg, 4, addr)}, //LDA Absolute
        0xbd => {addr = bus::absolute_x(memory, reg); op::lda(memory, reg, 4, addr)}, //LDA Absolute,X
        0xb9 => {addr = bus::absolute_y(memory, reg); op::lda(memory, reg, 4, addr)}, //LDA Absolute,Y
        //LDA Indirect,X
        //LDA Indirect,Y


        //Load X
        0xa2 => {op::ldx(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDX Immediate
        0xa6 => {addr = bus::zp(memory, reg); op::ldx(memory, reg, 3, addr)}, //LDX ZP
        0xb6 => {addr = bus::zp_y(memory, reg); op::ldx(memory, reg, 4, addr)}, //LDX ZP,Y
        0xae => {addr = bus::absolute(memory, reg); op::ldx(memory, reg, 4, addr)}, //LDX Absolute
        0xbe => {addr = bus::absolute_y(memory, reg); op::ldx(memory, reg, 4, addr)}, //LDX Absolute,Y


        //Load Y
        0xa0 => {op::ldy(memory, reg, 2, reg.pc); reg.pc += 1;}, //LDY Immediate
        0xa4 => {addr = bus::zp(memory, reg); op::ldy(memory, reg, 3, addr)}, //LDY ZP
        0xb4 => {addr = bus::zp_x(memory, reg); op::ldy(memory, reg, 4, addr)}, //LDY ZP,X
        0xac => {addr = bus::absolute(memory, reg); op::ldy(memory, reg, 4, addr)}, //LDY Absolute
        0xbc => {addr = bus::absolute_x(memory, reg); op::ldy(memory, reg, 4, addr)}, //LDY Absolute,X


        //Logical Shift Right
        0x4a => {op::lsr(memory, reg, 2, None); reg.pc += 1;}, //LSR A
        0x46 => {addr = bus::zp(memory, reg); op::lsr(memory, reg, 5, Some(addr))}, //LSR ZP
        0x56 => {addr = bus::zp_x(memory, reg); op::lsr(memory, reg, 6, Some(addr))}, //LSR ZP,X
        0x4e => {addr = bus::absolute(memory, reg); op::lsr(memory, reg, 6, Some(addr))}, //LSR Absolute
        0x5e => {addr = bus::absolute_x(memory, reg); op::lsr(memory, reg, 7, Some(addr))}, //LSR Absolute,X


        //No Operation
        0xea => {reg.cycles_used += 2} //NOP


        //Rotate Left


        //Rotate Right


        //Return from Interrupt


        //Return from Subroutine


        //Subtract with Carry


        //Stack Instructions
        0x9a => {reg.cycles_used += 2; reg.sp = reg.x}, //TXS
        0xba => {reg.cycles_used += 2; reg.x = reg.sp}, //TSX
        0x48 => {reg.cycles_used += 3; bus::push_stack(memory, reg, reg.a)}, //PHA
        0x68 => {reg.cycles_used += 4; reg.a = bus::pull_stack(memory, reg)},     //PLA
        0x08 => {reg.cycles_used += 3; bus::push_stack(memory, reg, reg.sr)},//PHP
        0x28 => {reg.cycles_used += 4; reg.sr = bus::pull_stack(memory, reg)},    //PLP


        //Set Flag Instructions
        0x38 => {reg.cycles_used += 2; reg.set_carry(true)}, //SEC
        0xf8 => {reg.cycles_used += 2; reg.set_decimal(true)}, //SED
        0x78 => {reg.cycles_used += 2; reg.set_interrupt(true)}, //SEI


        //Store A
        0x85 => {addr = bus::zp(memory, reg); op::sta(memory, reg, 3, addr)}, //STA ZP
        0x95 => {addr = bus::zp_x(memory, reg); op::sta(memory, reg, 4, addr)}, //STA ZP,X
        0x8d => {addr = bus::absolute(memory, reg); op::sta(memory, reg, 4, addr)}, //STA Absolute
        0x9d => {addr = bus::absolute_x(memory, reg); op::sta(memory, reg, 5, addr)}, //STA Absolute,X
        0x99 => {addr = bus::absolute_y(memory, reg); op::sta(memory, reg, 5, addr)}, //STA Absolute,Y
        //STA Indirect,X
        //STA Indirect,Y


        //Store X
        0x86 => {addr = bus::zp(memory, reg); op::stx(memory, reg, 3, addr)}, //STA ZP
        0x96 => {addr = bus::zp_x(memory, reg); op::stx(memory, reg, 4, addr)}, //STA ZP,X
        0x8e => {addr = bus::absolute(memory, reg); op::stx(memory, reg, 4, addr)}, //STA Absolute

        
        //Store Y
        0x84 => {addr = bus::zp(memory, reg); op::sty(memory, reg, 3, addr)}, //STA ZP
        0x94 => {addr = bus::zp_x(memory, reg); op::sty(memory, reg, 4, addr)}, //STA ZP,X
        0x8c => {addr = bus::absolute(memory, reg); op::sty(memory, reg, 4, addr)}, //STA Absolute


        other => return Err(format!("Unrecognized opcode {:#04x}! Halting execution...", other)) //whoops! invalid opcode
    }

    Ok(reg.cycles_used)
}