use crate::bus;
use crate::bus::Segment;
use crate::op;

use text_io::{try_scan, read};
use std::io::{Read, Error, ErrorKind, Write, stdout};
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
    pub clock_time: u64,
    pub running: bool
}


impl CpuStatus
{
    pub fn new(speed: u64) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, last_op: 0, cycles_used: 0, reset: true, debug_text: false, clock_time: (1000000000 / speed), running: true}
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

    pub fn status_report(&mut self)
    {
        println!("Current CPU status:");
        println!("Last Opcode: {:#04x} X: {:#04x} Y: {:#04x} A: {:#04x} SP: {:#04x} SR: {:#010b} PC: {:#06x}", self.last_op, self.x, self.y, self.a, self.sp, self.sr, self.pc)
    }

    pub fn execute<'a>(&mut self, memory: &mut [Segment]) -> Result<u8, String> //runs a single CPU instruction, returns errors if there are any
    {
        self.cycles_used = 0;
        let addr: u16;
        let flag: bool;

        if self.reset                                                    //do we need to reset the CPU?
        {
            self.pc = 0xfffc;

            let lo_byte : u8 = bus::read(memory,self.pc); //retrieve reset vector from ROM
            self.pc += 1;
            let hi_byte : u8 = bus::read(memory,self.pc);

            self.pc = lo_byte as u16 + ((hi_byte as u16) << 8);           //set new program counter at reset routine
            
            self.cycles_used += 7;
            self.reset = false;

            if self.debug_text { println!("Starting program execution at {:#06x}", self.pc) }
        }

        let opcode: u8 = bus::read(memory, self.pc);        //get the current opcode
        self.last_op = opcode;

        self.pc += 1; 

        match opcode            //which instruction is it?
        {
            //Add With Carry
            0x69 => {op::adc(memory, self, 2, self.pc); self.pc += 1}, //ADC Immediate
            0x65 => {addr = bus::zp(memory, self); op::adc(memory, self, 2, addr)}, //ADC ZP
            0x75 => {addr = bus::zp_x(memory, self); op::adc(memory, self, 2, addr)}, //ADC ZP,X
            0x6d => {addr = bus::absolute(memory, self); op::adc(memory, self, 2, addr)}, //ADC Absolute
            0x7d => {addr = bus::absolute_x(memory, self, true); op::adc(memory, self, 2, addr)}, //ADC Absolute,X
            0x79 => {addr = bus::absolute_y(memory, self, true); op::adc(memory, self, 2, addr)}, //ADC Absolute,Y
            0x61 => {addr = bus::indirect_x(memory, self); op::adc(memory, self, 2, addr)}, //ADC Indirect,X
            0x71 => {addr = bus::indirect_y(memory, self, true); op::adc(memory, self, 2, addr)}, //ADC Indirect,Y

            //And Bitwise with Accumulator
            0x29 => {op::and(memory, self, 2, self.pc); self.pc += 1}, //AND Immediate
            0x25 => {addr = bus::zp(memory, self); op::and(memory, self, 3, addr)}, //AND ZP
            0x35 => {addr = bus::zp_x(memory, self); op::and(memory, self, 4, addr)}, //AND ZP,X
            0x2d => {addr = bus::absolute(memory, self); op::and(memory, self, 4, addr)}, //AND Absolute
            0x3d => {addr = bus::absolute_x(memory, self, true); op::and(memory, self, 4, addr)}, //AND Absolute,X
            0x39 => {addr = bus::absolute_y(memory, self, true); op::and(memory, self, 4, addr)}, //AND Absolute,Y
            0x21 => {addr = bus::indirect_x(memory, self); op::and(memory, self, 6, addr)}, //AND Indirect,X
            0x31 => {addr = bus::indirect_y(memory, self, true); op::and(memory, self, 5, addr)}, //AND Indirect,Y

            //Arithmetic Shift Left
            0x0a => {op::asl(memory, self, 2, None)}, //ASL A
            0x06 => {addr = bus::zp(memory, self); op::asl(memory, self, 5, Some(addr))}, //ASL ZP
            0x16 => {addr = bus::zp_x(memory, self); op::asl(memory, self, 6, Some(addr))}, //ASL ZP,X
            0x0e => {addr = bus::absolute(memory, self); op::asl(memory, self, 6, Some(addr))}, //ASL Absolute
            0x1e => {addr = bus::absolute_x(memory, self, true); op::asl(memory, self, 7, Some(addr))}, //ASL Absolute,X

            //Bit Test
            0x24 => {addr = bus::zp(memory, self); op::bit(memory, self, 3, addr)}, // BIT ZP
            0x2c => {addr = bus::absolute(memory, self); op::bit(memory, self, 4, addr)}, // BIT Absolute

            //Branch Instructions
            0x10 => {flag = !self.negative_flag(); op::branch(memory, self, flag)}, //BPL Branch on PLus
            0x30 => {flag = self.negative_flag(); op::branch(memory, self, flag)}, //BMI Branch on MInus
            0x50 => {flag = !self.overflow_flag(); op::branch(memory, self, flag)}, //BVC Branch on oVerflow Clear
            0x70 => {flag = self.overflow_flag(); op::branch(memory, self, flag)}, //BVS Branch on oVerflow Set
            0x90 => {flag = !self.carry_flag(); op::branch(memory, self, flag)}, //BCC Branch on Carry Clear
            0xb0 => {flag = self.carry_flag(); op::branch(memory, self, flag)}, //BCS Branch on Carry Set
            0xd0 => {flag = !self.zero_flag(); op::branch(memory, self, flag)}, //BNE Branch on Not Equal
            0xf0 => {flag = self.zero_flag(); op::branch(memory, self, flag)}, //BEQ Branch on EQual

            //Break

            //Clear Flag Instructions
            0x18 => {self.cycles_used += 2; self.set_carry(false)}, //CLC
            0xd8 => {self.cycles_used += 2; self.set_decimal(false)}, //CLD
            0x58 => {self.cycles_used += 2; self.set_interrupt(false)}, //CLI
            0xb8 => {self.cycles_used += 2; self.set_overflow(false)}, //CLV


            //Compare with Accumulator
            0xc9 => {op::cmp(memory, self, 2, self.pc); self.pc += 1}, //CMP Immediate
            0xc5 => {addr = bus::zp(memory, self); op::cmp(memory, self, 3, addr)}, //CMP ZP
            0xd5 => {addr = bus::zp_x(memory, self); op::cmp(memory, self, 4, addr)}, //CMP ZP,X
            0xcd => {addr = bus::absolute(memory, self); op::cmp(memory, self, 4, addr)}, //CMP Absolute
            0xdd => {addr = bus::absolute_x(memory, self, true); op::cmp(memory, self, 4, addr)}, //CMP Absolute,X
            0xd9 => {addr = bus::absolute_y(memory, self, true); op::cmp(memory, self, 4, addr)}, //CMP Absolute,Y
            0xc1 => {addr = bus::indirect_x(memory, self); op::cmp(memory, self, 6, addr)}, //CMP Indirect,X
            0xd1 => {addr = bus::indirect_y(memory, self, true); op::cmp(memory, self, 5, addr)}, //CMP Indirect,Y


            //Compare with X
            0xe0 => {op::cpx(memory, self, 2, self.pc); self.pc += 1}, //CPX Immediate
            0xe4 => {addr = bus::zp(memory, self); op::cpx(memory, self, 3, addr)}, //CPX ZP
            0xec => {addr = bus::absolute(memory, self); op::cpx(memory, self, 4, addr)}, //CPX Absolute


            //Compare with Y
            0xc0 => {op::cpy(memory, self, 2, self.pc); self.pc += 1}, //CPY Immediate
            0xc4 => {addr = bus::zp(memory, self); op::cpy(memory, self, 3, addr)}, //CPY ZP
            0xcc => {addr = bus::absolute(memory, self); op::cpy(memory, self, 4, addr)}, //CPY Absolute


            //Decrement Memory
            0xc6 => {addr = bus::zp(memory, self); op::dec(memory, self, 5, addr)}, //DEC ZP
            0xd6 => {addr = bus::zp_x(memory, self); op::dec(memory, self, 6, addr)}, //DEC ZP,X
            0xce => {addr = bus::absolute(memory, self); op::dec(memory, self, 6, addr)}, //DEC Absolute
            0xde => {addr = bus::absolute_x(memory, self, true); op::dec(memory, self, 7, addr)}, //DEC Absolute,X


            //Decrement X
            0xca => {self.x = self.x.wrapping_sub(1); self.set_zero(self.x == 0); self.set_negative(self.x > 0x7f); self.cycles_used += 2}, //DEX


            //Decrement Y
            0x88 => {self.y = self.y.wrapping_sub(1); self.set_zero(self.y == 0); self.set_negative(self.y > 0x7f);  self.cycles_used += 2}, //DEY


            //Exclusive OR
            0x49 => {op::eor(memory, self, 2, self.pc); self.pc += 1}, //EOR Immediate
            0x45 => {addr = bus::zp(memory, self); op::eor(memory, self, 3, addr)}, //EOR ZP
            0x55 => {addr = bus::zp_x(memory, self); op::eor(memory, self, 4, addr)}, //EOR ZP,X
            0x4d => {addr = bus::absolute(memory, self); op::eor(memory, self, 4, addr)}, //EOR Absolute
            0x5d => {addr = bus::absolute_x(memory, self, true); op::eor(memory, self, 4, addr)}, //EOR Absolute,X
            0x59 => {addr = bus::absolute_y(memory, self, true); op::eor(memory, self, 4, addr)}, //EOR Absolute,Y
            0x41 => {addr = bus::indirect_x(memory, self); op::eor(memory, self, 4, addr)}, //EOR Indirect,X
            0x51 => {addr = bus::indirect_y(memory, self, true); op::eor(memory, self, 4, addr)}, //EOR Indirect,Y


            //Increment Memory
            0xe6 => {addr = bus::zp(memory, self); op::inc(memory, self, 5, addr)}, //INC ZP
            0xf6 => {addr = bus::zp_x(memory, self); op::inc(memory, self, 6, addr)}, //INC ZP,X
            0xee => {addr = bus::absolute(memory, self); op::inc(memory, self, 6, addr)}, //INC Absolute
            0xfe => {addr = bus::absolute_x(memory, self, true); op::inc(memory, self, 7, addr)}, //INC Absolute,X


            //Increment X
            0xe8 => {self.x = self.x.wrapping_add(1); self.set_zero(self.x == 0); self.set_negative(self.x > 0x7f);  self.cycles_used += 2}, //INX


            //Increment Y
            0xc8 => {self.y = self.y.wrapping_add(1); self.set_zero(self.y == 0); self.set_negative(self.y > 0x7f);  self.cycles_used += 2}, //INY


            //Jump
            0x4c => {addr = bus::absolute(memory, self); op::jmp(self, 3, addr)}, //JMP Absolute
            0x6c => {addr = bus::indirect(memory, self); op::jmp(self, 5, addr)}, //JMP Indirect


            //Jump to Subroutine
            0x20 => {addr = bus::absolute(memory, self); op::jsr(memory, self, 6, addr)}, //JSR Absolute


            //Load A
            0xa9 => {op::lda(memory, self, 2, self.pc); self.pc += 1}, //LDA Immediate
            0xa5 => {addr = bus::zp(memory, self); op::lda(memory, self, 3, addr)}, //LDA ZP
            0xb5 => {addr = bus::zp_x(memory, self); op::lda(memory, self, 4, addr)}, //LDA ZP,X
            0xad => {addr = bus::absolute(memory, self); op::lda(memory, self, 4, addr)}, //LDA Absolute
            0xbd => {addr = bus::absolute_x(memory, self, true); op::lda(memory, self, 4, addr)}, //LDA Absolute,X
            0xb9 => {addr = bus::absolute_y(memory, self, true); op::lda(memory, self, 4, addr)}, //LDA Absolute,Y
            0xa1 => {addr = bus::indirect_x(memory, self); op::lda(memory, self, 6, addr)}, //LDA Indirect,X
            0xb1 => {addr = bus::indirect_y(memory, self, true); op::lda(memory, self, 5, addr)}, //LDA Indirect,Y


            //Load X
            0xa2 => {op::ldx(memory, self, 2, self.pc); self.pc += 1}, //LDX Immediate
            0xa6 => {addr = bus::zp(memory, self); op::ldx(memory, self, 3, addr)}, //LDX ZP
            0xb6 => {addr = bus::zp_y(memory, self); op::ldx(memory, self, 4, addr)}, //LDX ZP,Y
            0xae => {addr = bus::absolute(memory, self); op::ldx(memory, self, 4, addr)}, //LDX Absolute
            0xbe => {addr = bus::absolute_y(memory, self, true); op::ldx(memory, self, 4, addr)}, //LDX Absolute,Y


            //Load Y
            0xa0 => {op::ldy(memory, self, 2, self.pc); self.pc += 1}, //LDY Immediate
            0xa4 => {addr = bus::zp(memory, self); op::ldy(memory, self, 3, addr)}, //LDY ZP
            0xb4 => {addr = bus::zp_x(memory, self); op::ldy(memory, self, 4, addr)}, //LDY ZP,X
            0xac => {addr = bus::absolute(memory, self); op::ldy(memory, self, 4, addr)}, //LDY Absolute
            0xbc => {addr = bus::absolute_x(memory, self, true); op::ldy(memory, self, 4, addr)}, //LDY Absolute,X


            //Logical Shift Right
            0x4a => {op::lsr(memory, self, 2, None)}, //LSR A
            0x46 => {addr = bus::zp(memory, self); op::lsr(memory, self, 5, Some(addr))}, //LSR ZP
            0x56 => {addr = bus::zp_x(memory, self); op::lsr(memory, self, 6, Some(addr))}, //LSR ZP,X
            0x4e => {addr = bus::absolute(memory, self); op::lsr(memory, self, 6, Some(addr))}, //LSR Absolute
            0x5e => {addr = bus::absolute_x(memory, self, false); op::lsr(memory, self, 7, Some(addr))}, //LSR Absolute,X


            //OR with Accumulator
            0x09 => {op::ora(memory, self, 2, self.pc); self.pc += 1}, //ORA Immediate
            0x05 => {addr = bus::zp(memory, self); op::ora(memory, self, 3, addr)}, //ORA ZP
            0x15 => {addr = bus::zp_x(memory, self); op::ora(memory, self, 4, addr)}, //ORA ZP,X
            0x0d => {addr = bus::absolute(memory, self); op::ora(memory, self, 4, addr)}, //ORA Absolute
            0x1d => {addr = bus::absolute_x(memory, self, true); op::ora(memory, self, 4, addr)}, //ORA Absolute,X
            0x19 => {addr = bus::absolute_y(memory, self, true); op::ora(memory, self, 4, addr)}, //ORA Absolute,Y
            0x01 => {addr = bus::indirect_x(memory, self); op::ora(memory, self, 4, addr)}, //ORA Indirect,X
            0x11 => {addr = bus::indirect_y(memory, self, true); op::ora(memory, self, 4, addr)}, //ORA Indirect,Y


            //No Operation
            0xea => {self.cycles_used += 2}, //NOP


            //Rotate Left
            0x2a => {op::rol(memory, self, 2, None)}, //ROL A
            0x26 => {addr = bus::zp(memory, self); op::rol(memory, self, 5, Some(addr))}, //ROL ZP
            0x36 => {addr = bus::zp_x(memory, self); op::rol(memory, self, 6, Some(addr))}, //ROL ZP,X
            0x2e => {addr = bus::absolute(memory, self); op::rol(memory, self, 6, Some(addr))}, //ROL Absolute
            0x3e => {addr = bus::absolute_x(memory, self, true); op::rol(memory, self, 7, Some(addr))}, //ROL Absolute,X


            //Rotate Right
            0x6a => {op::ror(memory, self, 2, None)}, //ROR A
            0x66 => {addr = bus::zp(memory, self); op::ror(memory, self, 5, Some(addr))}, //ROR ZP
            0x76 => {addr = bus::zp_x(memory, self); op::ror(memory, self, 6, Some(addr))}, //ROR ZP,X
            0x6e => {addr = bus::absolute(memory, self); op::ror(memory, self, 6, Some(addr))}, //ROR Absolute
            0x7e => {addr = bus::absolute_x(memory, self, true); op::ror(memory, self, 7, Some(addr))}, //ROR Absolute,X


            //Return from Interrupt


            //Return from Subroutine
            0x60 => {op::rts(memory, self, 6)},


            //Subtract with Carry
            0xe9 => {op::sbc(memory, self, 2, self.pc); self.pc += 1}, //SBC Immediate
            0xe5 => {addr = bus::zp(memory, self); op::sbc(memory, self, 2, addr)}, //SBC ZP
            0xf5 => {addr = bus::zp_x(memory, self); op::sbc(memory, self, 2, addr)}, //SBC ZP,X
            0xed => {addr = bus::absolute(memory, self); op::sbc(memory, self, 2, addr)}, //SBC Absolute
            0xfd => {addr = bus::absolute_x(memory, self, true); op::sbc(memory, self, 2, addr)}, //SBC Absolute,X
            0xf9 => {addr = bus::absolute_y(memory, self, true); op::sbc(memory, self, 2, addr)}, //SBC Absolute,Y
            0xe1 => {addr = bus::indirect_x(memory, self); op::sbc(memory, self, 2, addr)}, //SBC Indirect,X
            0xf1 => {addr = bus::indirect_y(memory, self, true); op::sbc(memory, self, 2, addr)}, //SBC Indirect,Y


            //Stack Instructions
            0x9a => {op::transfer(self, 'x', 's')}, //TXS
            0xba => {op::transfer(self, 's', 'x')}, //TSX
            0x48 => {self.cycles_used += 3; bus::push_stack(memory, self, self.a)}, //PHA
            0x68 => {self.cycles_used += 4; self.a = bus::pull_stack(memory, self)},     //PLA
            0x08 => {self.cycles_used += 3; bus::push_stack(memory, self, self.sr)},//PHP
            0x28 => {self.cycles_used += 4; self.sr = bus::pull_stack(memory, self)},    //PLP


            //Set Flag Instructions
            0x38 => {self.cycles_used += 2; self.set_carry(true)}, //SEC
            0xf8 => {self.cycles_used += 2; self.set_decimal(true)}, //SED
            0x78 => {self.cycles_used += 2; self.set_interrupt(true)}, //SEI


            //Store A
            0x85 => {addr = bus::zp(memory, self); op::sta(memory, self, 3, addr)}, //STA ZP
            0x95 => {addr = bus::zp_x(memory, self); op::sta(memory, self, 4, addr)}, //STA ZP,X
            0x8d => {addr = bus::absolute(memory, self); op::sta(memory, self, 4, addr)}, //STA Absolute
            0x9d => {addr = bus::absolute_x(memory, self, true); op::sta(memory, self, 5, addr)}, //STA Absolute,X
            0x99 => {addr = bus::absolute_y(memory, self, true); op::sta(memory, self, 5, addr)}, //STA Absolute,Y
            0x81 => {addr = bus::indirect_x(memory, self); op::sta(memory, self, 6, addr)}, //STA Indirect,X
            0x91 => {addr = bus::indirect_y(memory, self, true); op::sta(memory, self, 6, addr)}, //STA Indirect,Y


            //Store X
            0x86 => {addr = bus::zp(memory, self); op::stx(memory, self, 3, addr)}, //STA ZP
            0x96 => {addr = bus::zp_x(memory, self); op::stx(memory, self, 4, addr)}, //STA ZP,X
            0x8e => {addr = bus::absolute(memory, self); op::stx(memory, self, 4, addr)}, //STA Absolute

            
            //Store Y
            0x84 => {addr = bus::zp(memory, self); op::sty(memory, self, 3, addr)}, //STA ZP
            0x94 => {addr = bus::zp_x(memory, self); op::sty(memory, self, 4, addr)}, //STA ZP,X
            0x8c => {addr = bus::absolute(memory, self); op::sty(memory, self, 4, addr)}, //STA Absolute


            //Transfer Register Value
            0xaa => {op::transfer(self, 'a', 'x')}, //TAX
            0xa8 => {op::transfer(self, 'a', 'y')}, //TAY
            0x8a => {op::transfer(self, 'x', 'a')}, //TXA
            0x98 => {op::transfer(self, 'y', 'a')}, //TYA


            other => return Err(format!("Unrecognized opcode {:#04x}! Halting execution...", other)) //whoops! invalid opcode
        }

        Ok(self.cycles_used)
    }

    
   pub fn debug_mode(&mut self, memory: &mut [Segment]) -> bool
   {
        let last_cmd: String = read!("{}\n");       //get text input and store it whole

        let poke = CpuStatus::parse_poke(&last_cmd);
        let peek = CpuStatus::parse_peek(&last_cmd);

        if poke.is_ok()
        {
            let poke_t = poke.unwrap();
            bus::write(memory, poke_t.0, poke_t.1);
            println!("Wrote {:#04x} to address {:#06x}", poke_t.1, poke_t.0);
        }
        else if peek.is_ok()
        {
            let peek_a = peek.unwrap();
            let peek_b = bus::read(memory,peek_a);
            println!("Read {:#04x} from address {:#06x}", peek_b, peek_a);
        }
        else
        {
            match last_cmd.trim()           //check for single-word commands with no arguments
            {
                "verbose" => self.debug_text = !self.debug_text, //enable or disable debug commentary
                "run" => self.running = true,                     //run command: start running code
                "reset" => self.reset = true,                    //reset command: reset the CPU
                "status" => self.status_report(),      //status command: get status of registers
    
                "step" =>                                        //step command: run a single operation and display results
                {   let check: Result<u8, String> = self.execute(memory);
                    if check.is_err()
                    {
                        println!("{}",check.unwrap_err());
                    }
                    else 
                    {
                        let cycles_taken: u8 = check.unwrap();
                        if self.debug_text {println!("Instruction used {} cycles...", cycles_taken)};
                    }
    
                    self.status_report(); 
                },
    
                "exit" => return false,                                //exit command: close emulator
                _ => println!("What?")
            }
        }
        
        print!(">");
        stdout().flush();
        return true;
   }

   fn parse_peek(cmd: &String) -> Result<u16, Box<dyn std::error::Error>>
   {
        let addr: u16;

        try_scan!(cmd.trim().bytes() => "{}", addr);

        Ok(addr)
   }

   fn parse_poke(cmd: &String) -> Result<(u16, u8), Box<dyn std::error::Error>>
   {
        let addr: u16;
        let byte: u8;

        try_scan!(cmd.trim().bytes() => "{}:{}", addr, byte);

        Ok((addr, byte))
   } 
}
