use crate::bus;
use crate::bus::Segment;

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
    pub running: bool,
    external_irq: bool,
    external_nmi: bool
}


impl CpuStatus
{
    pub fn new(speed: u64) -> CpuStatus
    {
        CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, last_op: 0, cycles_used: 0, reset: true, debug_text: false, clock_time: (1000000000 / speed), running: true, external_irq: false, external_nmi: false}
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
            self.pc = bus::absolute(memory, self);           //set new program counter at reset routine
            
            self.cycles_used += 7;
            self.reset = false;

            if self.debug_text { println!("Starting program execution at {:#06x}", self.pc) }
        }

        if self.external_nmi                                           //was there a non-maskable interrupt request?
        {
            bus::push_stack(memory, self, ((self.pc & 0xff00) >> 8) as u8); //handle NMI
            bus::push_stack(memory, self, (self.pc & 0x00ff) as u8);
            bus::push_stack(memory, self, self.sr);

            self.pc = 0xfffa;
            self.pc = bus::absolute(memory, self);

            self.set_interrupt(true);

            self.cycles_used += 7;
            self.external_nmi = false;
            self.external_irq = false;
        }
        else if self.external_irq                                            //was there an interrupt request?
        {
            bus::push_stack(memory, self, ((self.pc & 0xff00) >> 8) as u8); //handle IRQ
            bus::push_stack(memory, self, (self.pc & 0x00ff) as u8);
            bus::push_stack(memory, self, self.sr);

            self.pc = 0xfffe;
            self.pc = bus::absolute(memory, self);

            self.set_interrupt(true);

            self.cycles_used += 7;
            self.external_irq = false;
        }

        let opcode: u8 = bus::read(memory, self.pc);        //get the current opcode
        self.last_op = opcode;

        self.pc = self.pc.wrapping_add(1); 

        match opcode            //which instruction is it?
        {
            //Add With Carry
            0x69 => {self.adc(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //ADC Immediate
            0x65 => {addr = bus::zp(memory, self); self.adc(memory, 3, addr)}, //ADC ZP
            0x75 => {addr = bus::zp_x(memory, self); self.adc(memory, 4, addr)}, //ADC ZP,X
            0x6d => {addr = bus::absolute(memory, self); self.adc(memory, 4, addr)}, //ADC Absolute
            0x7d => {addr = bus::absolute_x(memory, self, true); self.adc(memory, 4, addr)}, //ADC Absolute,X
            0x79 => {addr = bus::absolute_y(memory, self, true); self.adc(memory, 4, addr)}, //ADC Absolute,Y
            0x61 => {addr = bus::indirect_x(memory, self); self.adc(memory, 6, addr)}, //ADC Indirect,X
            0x71 => {addr = bus::indirect_y(memory, self, true); self.adc(memory, 5, addr)}, //ADC Indirect,Y

            //And Bitwise with Accumulator
            0x29 => {self.and(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //AND Immediate
            0x25 => {addr = bus::zp(memory, self); self.and(memory, 3, addr)}, //AND ZP
            0x35 => {addr = bus::zp_x(memory, self); self.and(memory, 4, addr)}, //AND ZP,X
            0x2d => {addr = bus::absolute(memory, self); self.and(memory, 4, addr)}, //AND Absolute
            0x3d => {addr = bus::absolute_x(memory, self, true); self.and(memory, 4, addr)}, //AND Absolute,X
            0x39 => {addr = bus::absolute_y(memory, self, true); self.and(memory, 4, addr)}, //AND Absolute,Y
            0x21 => {addr = bus::indirect_x(memory, self); self.and(memory, 6, addr)}, //AND Indirect,X
            0x31 => {addr = bus::indirect_y(memory, self, true); self.and(memory, 5, addr)}, //AND Indirect,Y

            //Arithmetic Shift Left
            0x0a => {self.asl(memory, 2, None)}, //ASL A
            0x06 => {addr = bus::zp(memory, self); self.asl(memory, 5, Some(addr))}, //ASL ZP
            0x16 => {addr = bus::zp_x(memory, self); self.asl(memory, 6, Some(addr))}, //ASL ZP,X
            0x0e => {addr = bus::absolute(memory, self); self.asl(memory, 6, Some(addr))}, //ASL Absolute
            0x1e => {addr = bus::absolute_x(memory, self, true); self.asl(memory, 7, Some(addr))}, //ASL Absolute,X

            //Bit Test
            0x24 => {addr = bus::zp(memory, self); self.bit(memory, 3, addr)}, // BIT ZP
            0x2c => {addr = bus::absolute(memory, self); self.bit(memory, 4, addr)}, // BIT Absolute

            //Branch Instructions
            0x10 => {flag = !self.negative_flag(); self.branch(memory, flag)}, //BPL Branch on PLus
            0x30 => {flag = self.negative_flag(); self.branch(memory, flag)}, //BMI Branch on MInus
            0x50 => {flag = !self.overflow_flag(); self.branch(memory, flag)}, //BVC Branch on oVerflow Clear
            0x70 => {flag = self.overflow_flag(); self.branch(memory, flag)}, //BVS Branch on oVerflow Set
            0x90 => {flag = !self.carry_flag(); self.branch(memory, flag)}, //BCC Branch on Carry Clear
            0xb0 => {flag = self.carry_flag(); self.branch(memory, flag)}, //BCS Branch on Carry Set
            0xd0 => {flag = !self.zero_flag(); self.branch(memory, flag)}, //BNE Branch on Not Equal
            0xf0 => {flag = self.zero_flag(); self.branch(memory, flag)}, //BEQ Branch on EQual

            //Break
            0x00 => {self.brk(memory, 7)},

            //Clear Flag Instructions
            0x18 => {self.cycles_used += 2; self.set_carry(false)}, //CLC
            0xd8 => {self.cycles_used += 2; self.set_decimal(false)}, //CLD
            0x58 => {self.cycles_used += 2; self.set_interrupt(false)}, //CLI
            0xb8 => {self.cycles_used += 2; self.set_overflow(false)}, //CLV


            //Compare with Accumulator
            0xc9 => {self.cmp(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //CMP Immediate
            0xc5 => {addr = bus::zp(memory, self); self.cmp(memory, 3, addr)}, //CMP ZP
            0xd5 => {addr = bus::zp_x(memory, self); self.cmp(memory, 4, addr)}, //CMP ZP,X
            0xcd => {addr = bus::absolute(memory, self); self.cmp(memory, 4, addr)}, //CMP Absolute
            0xdd => {addr = bus::absolute_x(memory, self, true); self.cmp(memory, 4, addr)}, //CMP Absolute,X
            0xd9 => {addr = bus::absolute_y(memory, self, true); self.cmp(memory, 4, addr)}, //CMP Absolute,Y
            0xc1 => {addr = bus::indirect_x(memory, self); self.cmp(memory, 6, addr)}, //CMP Indirect,X
            0xd1 => {addr = bus::indirect_y(memory, self, true); self.cmp(memory, 5, addr)}, //CMP Indirect,Y


            //Compare with X
            0xe0 => {self.cpx(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //CPX Immediate
            0xe4 => {addr = bus::zp(memory, self); self.cpx(memory, 3, addr)}, //CPX ZP
            0xec => {addr = bus::absolute(memory, self); self.cpx(memory, 4, addr)}, //CPX Absolute


            //Compare with Y
            0xc0 => {self.cpy(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //CPY Immediate
            0xc4 => {addr = bus::zp(memory, self); self.cpy(memory, 3, addr)}, //CPY ZP
            0xcc => {addr = bus::absolute(memory, self); self.cpy(memory, 4, addr)}, //CPY Absolute


            //Decrement Memory
            0xc6 => {addr = bus::zp(memory, self); self.dec(memory, 5, addr)}, //DEC ZP
            0xd6 => {addr = bus::zp_x(memory, self); self.dec(memory, 6, addr)}, //DEC ZP,X
            0xce => {addr = bus::absolute(memory, self); self.dec(memory, 6, addr)}, //DEC Absolute
            0xde => {addr = bus::absolute_x(memory, self, true); self.dec(memory, 7, addr)}, //DEC Absolute,X


            //Decrement X
            0xca => {self.x = self.x.wrapping_sub(1); self.set_zero(self.x == 0); self.set_negative(self.x > 0x7f); self.cycles_used += 2}, //DEX


            //Decrement Y
            0x88 => {self.y = self.y.wrapping_sub(1); self.set_zero(self.y == 0); self.set_negative(self.y > 0x7f);  self.cycles_used += 2}, //DEY


            //Exclusive OR
            0x49 => {self.eor(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //EOR Immediate
            0x45 => {addr = bus::zp(memory, self); self.eor(memory, 3, addr)}, //EOR ZP
            0x55 => {addr = bus::zp_x(memory, self); self.eor(memory, 4, addr)}, //EOR ZP,X
            0x4d => {addr = bus::absolute(memory, self); self.eor(memory, 4, addr)}, //EOR Absolute
            0x5d => {addr = bus::absolute_x(memory, self, true); self.eor(memory, 4, addr)}, //EOR Absolute,X
            0x59 => {addr = bus::absolute_y(memory, self, true); self.eor(memory, 4, addr)}, //EOR Absolute,Y
            0x41 => {addr = bus::indirect_x(memory, self); self.eor(memory, 4, addr)}, //EOR Indirect,X
            0x51 => {addr = bus::indirect_y(memory, self, true); self.eor(memory, 4, addr)}, //EOR Indirect,Y

            //Halt (invalid opcodes that would freeze the CPU on a real NMOS 6502)
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2 | 0xf2 => return Err(format!("Illegal (JAM/HLT/KIL) opcode {:#04x}! Halting execution...", self.last_op)),

            //Increment Memory
            0xe6 => {addr = bus::zp(memory, self); self.inc(memory, 5, addr)}, //INC ZP
            0xf6 => {addr = bus::zp_x(memory, self); self.inc(memory, 6, addr)}, //INC ZP,X
            0xee => {addr = bus::absolute(memory, self); self.inc(memory, 6, addr)}, //INC Absolute
            0xfe => {addr = bus::absolute_x(memory, self, true); self.inc(memory, 7, addr)}, //INC Absolute,X


            //Increment X
            0xe8 => {self.x = self.x.wrapping_add(1); self.set_zero(self.x == 0); self.set_negative(self.x > 0x7f);  self.cycles_used += 2}, //INX


            //Increment Y
            0xc8 => {self.y = self.y.wrapping_add(1); self.set_zero(self.y == 0); self.set_negative(self.y > 0x7f);  self.cycles_used += 2}, //INY


            //Jump
            0x4c => {addr = bus::absolute(memory, self); self.jmp(3, addr)}, //JMP Absolute
            0x6c => {addr = bus::indirect(memory, self); self.jmp(5, addr)}, //JMP Indirect


            //Jump to Subroutine
            0x20 => {addr = bus::absolute(memory, self); self.jsr(memory, 6, addr)}, //JSR Absolute


            //Load A
            0xa9 => {self.lda(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //LDA Immediate
            0xa5 => {addr = bus::zp(memory, self); self.lda(memory, 3, addr)}, //LDA ZP
            0xb5 => {addr = bus::zp_x(memory, self); self.lda(memory, 4, addr)}, //LDA ZP,X
            0xad => {addr = bus::absolute(memory, self); self.lda(memory, 4, addr)}, //LDA Absolute
            0xbd => {addr = bus::absolute_x(memory, self, true); self.lda(memory, 4, addr)}, //LDA Absolute,X
            0xb9 => {addr = bus::absolute_y(memory, self, true); self.lda(memory, 4, addr)}, //LDA Absolute,Y
            0xa1 => {addr = bus::indirect_x(memory, self); self.lda(memory, 6, addr)}, //LDA Indirect,X
            0xb1 => {addr = bus::indirect_y(memory, self, true); self.lda(memory, 5, addr)}, //LDA Indirect,Y


            //Load X
            0xa2 => {self.ldx(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //LDX Immediate
            0xa6 => {addr = bus::zp(memory, self); self.ldx(memory, 3, addr)}, //LDX ZP
            0xb6 => {addr = bus::zp_y(memory, self); self.ldx(memory, 4, addr)}, //LDX ZP,Y
            0xae => {addr = bus::absolute(memory, self); self.ldx(memory, 4, addr)}, //LDX Absolute
            0xbe => {addr = bus::absolute_y(memory, self, true); self.ldx(memory, 4, addr)}, //LDX Absolute,Y


            //Load Y
            0xa0 => {self.ldy(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //LDY Immediate
            0xa4 => {addr = bus::zp(memory, self); self.ldy(memory, 3, addr)}, //LDY ZP
            0xb4 => {addr = bus::zp_x(memory, self); self.ldy(memory, 4, addr)}, //LDY ZP,X
            0xac => {addr = bus::absolute(memory, self); self.ldy(memory, 4, addr)}, //LDY Absolute
            0xbc => {addr = bus::absolute_x(memory, self, true); self.ldy(memory, 4, addr)}, //LDY Absolute,X


            //Logical Shift Right
            0x4a => {self.lsr(memory, 2, None)}, //LSR A
            0x46 => {addr = bus::zp(memory, self); self.lsr(memory, 5, Some(addr))}, //LSR ZP
            0x56 => {addr = bus::zp_x(memory, self); self.lsr(memory, 6, Some(addr))}, //LSR ZP,X
            0x4e => {addr = bus::absolute(memory, self); self.lsr(memory, 6, Some(addr))}, //LSR Absolute
            0x5e => {addr = bus::absolute_x(memory, self, false); self.lsr(memory, 7, Some(addr))}, //LSR Absolute,X


            //OR with Accumulator
            0x09 => {self.ora(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //ORA Immediate
            0x05 => {addr = bus::zp(memory, self); self.ora(memory, 3, addr)}, //ORA ZP
            0x15 => {addr = bus::zp_x(memory, self); self.ora(memory, 4, addr)}, //ORA ZP,X
            0x0d => {addr = bus::absolute(memory, self); self.ora(memory, 4, addr)}, //ORA Absolute
            0x1d => {addr = bus::absolute_x(memory, self, true); self.ora(memory, 4, addr)}, //ORA Absolute,X
            0x19 => {addr = bus::absolute_y(memory, self, true); self.ora(memory, 4, addr)}, //ORA Absolute,Y
            0x01 => {addr = bus::indirect_x(memory, self); self.ora(memory, 4, addr)}, //ORA Indirect,X
            0x11 => {addr = bus::indirect_y(memory, self, true); self.ora(memory, 4, addr)}, //ORA Indirect,Y


            //No Operation
            0xea => {self.cycles_used += 2}, //NOP


            //Rotate Left
            0x2a => {self.rol(memory, 2, None)}, //ROL A
            0x26 => {addr = bus::zp(memory, self); self.rol(memory, 5, Some(addr))}, //ROL ZP
            0x36 => {addr = bus::zp_x(memory, self); self.rol(memory, 6, Some(addr))}, //ROL ZP,X
            0x2e => {addr = bus::absolute(memory, self); self.rol(memory, 6, Some(addr))}, //ROL Absolute
            0x3e => {addr = bus::absolute_x(memory, self, true); self.rol(memory, 7, Some(addr))}, //ROL Absolute,X


            //Rotate Right
            0x6a => {self.ror(memory, 2, None)}, //ROR A
            0x66 => {addr = bus::zp(memory, self); self.ror(memory, 5, Some(addr))}, //ROR ZP
            0x76 => {addr = bus::zp_x(memory, self); self.ror(memory, 6, Some(addr))}, //ROR ZP,X
            0x6e => {addr = bus::absolute(memory, self); self.ror(memory, 6, Some(addr))}, //ROR Absolute
            0x7e => {addr = bus::absolute_x(memory, self, true); self.ror(memory, 7, Some(addr))}, //ROR Absolute,X


            //Return from Interrupt
            0x40 => {self.rti(memory, 6)},


            //Return from Subroutine
            0x60 => {self.rts(memory, 6)},


            //Subtract with Carry
            0xe9 => {self.sbc(memory, 2, self.pc); self.pc = self.pc.wrapping_add(1)}, //SBC Immediate
            0xe5 => {addr = bus::zp(memory, self); self.sbc(memory, 3, addr)}, //SBC ZP
            0xf5 => {addr = bus::zp_x(memory, self); self.sbc(memory, 4, addr)}, //SBC ZP,X
            0xed => {addr = bus::absolute(memory, self); self.sbc(memory, 4, addr)}, //SBC Absolute
            0xfd => {addr = bus::absolute_x(memory, self, true); self.sbc(memory, 4, addr)}, //SBC Absolute,X
            0xf9 => {addr = bus::absolute_y(memory, self, true); self.sbc(memory, 4, addr)}, //SBC Absolute,Y
            0xe1 => {addr = bus::indirect_x(memory, self); self.sbc(memory, 6, addr)}, //SBC Indirect,X
            0xf1 => {addr = bus::indirect_y(memory, self, true); self.sbc(memory, 5, addr)}, //SBC Indirect,Y


            //Stack Instructions
            0x9a => {self.cycles_used += 2; self.transfer('x', 's')}, //TXS
            0xba => {self.cycles_used += 2; self.transfer('s', 'x')}, //TSX
            0x48 => {self.cycles_used += 3; bus::push_stack(memory, self, self.a)}, //PHA
            0x68 => {self.cycles_used += 4; self.a = bus::pull_stack(memory, self); self.set_negative(self.a > 0x7f); self.set_zero(self.a == 0)},     //PLA
            0x08 => {self.cycles_used += 3; bus::push_stack(memory, self, self.sr | 0x30)},                   //PHP
            0x28 => {self.cycles_used += 4; self.sr = self.sr & 0x30 | (bus::pull_stack(memory, self) & 0xcf)},    //PLP


            //Set Flag Instructions
            0x38 => {self.cycles_used += 2; self.set_carry(true)}, //SEC
            0xf8 => {self.cycles_used += 2; self.set_decimal(true)}, //SED
            0x78 => {self.cycles_used += 2; self.set_interrupt(true)}, //SEI


            //Store A
            0x85 => {addr = bus::zp(memory, self); self.sta(memory, 3, addr)}, //STA ZP
            0x95 => {addr = bus::zp_x(memory, self); self.sta(memory, 4, addr)}, //STA ZP,X
            0x8d => {addr = bus::absolute(memory, self); self.sta(memory, 4, addr)}, //STA Absolute
            0x9d => {addr = bus::absolute_x(memory, self, true); self.sta(memory, 5, addr)}, //STA Absolute,X
            0x99 => {addr = bus::absolute_y(memory, self, true); self.sta(memory, 5, addr)}, //STA Absolute,Y
            0x81 => {addr = bus::indirect_x(memory, self); self.sta(memory, 6, addr)}, //STA Indirect,X
            0x91 => {addr = bus::indirect_y(memory, self, true); self.sta(memory, 6, addr)}, //STA Indirect,Y


            //Store X
            0x86 => {addr = bus::zp(memory, self); self.stx(memory, 3, addr)}, //STX ZP
            0x96 => {addr = bus::zp_y(memory, self); self.stx(memory, 4, addr)}, //STX ZP,Y
            0x8e => {addr = bus::absolute(memory, self); self.stx(memory, 4, addr)}, //STX Absolute

            
            //Store Y
            0x84 => {addr = bus::zp(memory, self); self.sty(memory, 3, addr)}, //STY ZP
            0x94 => {addr = bus::zp_x(memory, self); self.sty(memory, 4, addr)}, //STY ZP,X
            0x8c => {addr = bus::absolute(memory, self); self.sty(memory, 4, addr)}, //STY Absolute


            //Transfer Register Value
            0xaa => {self.cycles_used += 2; self.transfer('a', 'x')}, //TAX
            0xa8 => {self.cycles_used += 2; self.transfer('a', 'y')}, //TAY
            0x8a => {self.cycles_used += 2; self.transfer('x', 'a')}, //TXA
            0x98 => {self.cycles_used += 2; self.transfer('y', 'a')}, //TYA


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
                
                "irq" => self.irq(),
                "nmi" => self.nmi(),
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


   fn adc(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
   {
        let byte: u8 = bus::read(memory, i_addr);

        let mut result: u16;

        match self.decimal_flag() //is this a BCD operation?
        {
            true => //BCD add (this implementation is based upon Py6502's version)
            {
                let mut half_carry: bool = false;
                let mut hi_adjust: u8 = 0;
                let mut lo_adjust: u8 = 0;

                let mut lo_nibble: u8 = (self.a & 0xf) + (byte & 0xf) + (self.carry_flag() as u8); //low bits of A + low bits of byte + Carry flag
                
                if lo_nibble > 9
                {
                    lo_adjust = 6;
                    half_carry = true;
                }

                let mut hi_nibble: u8 = ( (self.a >> 4) & 0xf ) + ( (byte>> 4) & 0xf ) + (half_carry as u8); //high bits of A + high bits of byte + Carry from low bits result

                self.set_carry(hi_nibble > 9);
                if self.carry_flag()
                {
                    hi_adjust = 6;
                }

                //ALU result without decimal adjustments
                lo_nibble &= 0xf;
                hi_nibble &= 0xf;
                let alu_result: u8 = (hi_nibble << 4) + lo_nibble;

                self.set_zero(alu_result == 0);
                self.set_negative(alu_result > 0x7f);
                self.set_overflow((byte & 0x80 == self.a & 0x80) && (alu_result & 0x80 != byte & 0x80));

                //Final A result with adjustment
                lo_nibble = (lo_nibble + lo_adjust) & 0xf;
                hi_nibble = (hi_nibble + hi_adjust) & 0xf;
                result = u16::from((hi_nibble << 4) + lo_nibble);
            }
            false => //Normal binary add
            {
                result = self.a as u16 + byte as u16 + self.carry_flag() as u16; // A + Byte + Carry

                self.set_carry(result > 0xff);

                if self.carry_flag()
                {
                    result &= 0xff;
                }

                self.set_overflow((byte & 0x80 == self.a & 0x80) && (result as u8 & 0x80 != byte & 0x80));
                self.set_zero(result == 0);
                self.set_negative(result > 0x7f);
            }
        }

        self.a = result as u8;

        self.cycles_used += cycles
    }


    fn and(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.a &= byte;

        self.set_negative(self.a > 0x7f);
        self.set_zero(self.a == 0);

        self.cycles_used += cycles
    }


    fn asl(&mut self, memory: &mut [Segment], cycles: u8, i_addr: Option<u16>) 
    {
        let mut byte: u8;

        match i_addr {
            Some(v) => byte = bus::read(memory, v),
            None => byte = self.a,
        };

        self.set_carry(0 != byte & 0b10000000);

        byte <<= 1;
        self.set_negative(byte > 0x7f);
        self.set_zero(byte == 0);

        match i_addr {
            Some(v) => bus::write(memory, v, byte),
            None => self.a = byte,
        };

        self.cycles_used += cycles
    }


    fn bit(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.set_negative(0 != byte & 0b10000000);
        self.set_overflow(0 != byte & 0b1000000);
        self.set_zero(0 == byte & self.a);

        self.cycles_used += cycles
    }


    fn branch(&mut self, memory: &mut [Segment], flag: bool)
    //basis for all branch instructions
    {
        self.cycles_used += 2; //use two cycles no matter what

        if flag
        //if the flag we tested is true and we should branch:
        {
            self.cycles_used += 1; //use another cycle

            let old_pc: u16 = self.pc; //store the old program counter to compare against later
            let offset: u8 = bus::read(memory, self.pc); //read the next byte to get the offset
            self.pc += 1;

            if offset < 127 //if the byte is positive, move PC forward that many bytes
            {
                self.pc += offset as u16;
            } 
            else //if the byte is negative, invert all the bits of the offset to convert it to positive again and then subtract from the PC
            {
                self.pc -= !offset as u16 + 1;
            }

            if old_pc & 0x100 != self.pc & 0x100
            //use another cycle if we crossed a page boundary
            {
                self.cycles_used += 1;
            }

            if self.debug_text {
                println!(
                    "Branching from address {:#06x} to {:#06x}...",
                    old_pc, self.pc
                )
            }
        } else
        //if the flag is false then just increment the program counter and do nothing else
        {
            self.pc += 1;

            if self.debug_text {
                println!("Branch condition evaluated but not taken.")
            }
        }
    }


    fn brk(&mut self, memory: &mut [Segment], cycles: u8)
    {
        self.pc += 1;
        self.set_break(true);

        bus::push_stack(memory, self, ((self.pc & 0xff00) >> 8) as u8);
        bus::push_stack(memory, self, (self.pc & 0x00ff) as u8);
        bus::push_stack(memory, self, self.sr);

        self.pc = 0xfffe;
        self.pc = bus::absolute(memory, self);

        self.cycles_used += cycles;
    }


    fn cmp(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.set_carry(self.a >= byte);
        self.set_zero(self.a == byte);
        self.set_negative(self.a.wrapping_sub(byte) > 0x7f);

        self.cycles_used += cycles
    }


    fn cpx(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.set_carry(self.x >= byte);
        self.set_zero(self.x == byte);
        self.set_negative(self.x.wrapping_sub(byte) > 0x7f);

        self.cycles_used += cycles
    }


    fn cpy(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.set_carry(self.y >= byte);
        self.set_zero(self.y == byte);
        self.set_negative(self.y.wrapping_sub(byte) > 0x7f);

        self.cycles_used += cycles
    }


    fn dec(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let mut byte: u8 = bus::read(memory, i_addr);

        byte = byte.wrapping_sub(1);

        self.set_negative(byte > 0x7f);
        self.set_zero(byte == 0);

        bus::write(memory, i_addr, byte);

        self.cycles_used += cycles
    }


    fn eor(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.a ^= byte;

        self.set_negative(self.a > 0x7f);
        self.set_zero(self.a == 0);

        self.cycles_used += cycles
    }


    fn inc(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let mut byte: u8 = bus::read(memory, i_addr);

        byte = byte.wrapping_add(1);

        self.set_negative(byte > 0x7f);
        self.set_zero(byte == 0);

        bus::write(memory, i_addr, byte);

        self.cycles_used += cycles
    }


    fn jmp(&mut self, cycles: u8, i_addr: u16) 
    {
        self.pc = i_addr;

        self.cycles_used += cycles;

        if self.debug_text {
            println!("JMP to new address {:#06x}...", self.pc)
        }
    }


    fn jsr(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let return_addr: u16 = self.pc - 1;
        let return_byte_lo: u8 = (return_addr & 0xff) as u8;
        let return_byte_hi: u8 = ((return_addr & 0xff00) >> 8) as u8;

        bus::push_stack(memory, self, return_byte_hi);
        bus::push_stack(memory, self, return_byte_lo);

        self.pc = i_addr;

        self.cycles_used += cycles;

        if self.debug_text {
            println!("JSR to new address {:#06x}...", self.pc)
        }
    }


    fn lda(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8;
        byte = bus::read(memory, i_addr);

        self.a = byte;

        self.set_negative(self.a > 0x7f);
        self.set_zero(self.a == 0);

        self.cycles_used += cycles
    }


    fn ldx(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8;
        byte = bus::read(memory, i_addr);

        self.x = byte;

        self.set_negative(self.x > 0x7f);
        self.set_zero(self.x == 0);

        self.cycles_used += cycles
    }


    fn ldy(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8;
        byte = bus::read(memory, i_addr);

        self.y = byte;

        self.set_negative(self.y > 0x7f);
        self.set_zero(self.y == 0);

        self.cycles_used += cycles
    }


    fn lsr(&mut self, memory: &mut [Segment], cycles: u8, i_addr: Option<u16>) 
    {
        let mut byte: u8;

        match i_addr {
            Some(v) => byte = bus::read(memory, v),
            None => byte = self.a,
        };

        self.set_carry(0 != byte & 0b1);

        byte >>= 1;
        self.set_negative(false);
        self.set_zero(byte == 0);

        match i_addr {
            Some(v) => bus::write(memory, v, byte),
            None => self.a = byte,
        };

        self.cycles_used += cycles
    }


    fn ora(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr);

        self.a |= byte;

        self.set_negative(self.a > 0x7f);
        self.set_zero(self.a == 0);

        self.cycles_used += cycles
    }


    fn rol(&mut self, memory: &mut [Segment], cycles: u8, i_addr: Option<u16>) 
    {
        let mut byte: u8;

        match i_addr {
            Some(v) => byte = bus::read(memory, v),
            None => byte = self.a,
        };

        let new_carry = 0 != byte & 0b10000000;

        byte <<= 1;

        if self.carry_flag() { byte |= 0b1 }

        self.set_carry(new_carry);
        self.set_negative(byte > 0x7f);
        self.set_zero(byte == 0);

        match i_addr {
            Some(v) => bus::write(memory, v, byte),
            None => self.a = byte,
        };

        self.cycles_used += cycles
    }


    fn ror(&mut self, memory: &mut [Segment], cycles: u8, i_addr: Option<u16>) 
    {
        let mut byte: u8;

        match i_addr {
            Some(v) => byte = bus::read(memory, v),
            None => byte = self.a,
        };

        let new_carry = 0 != byte & 0b1;

        byte >>= 1;

        if self.carry_flag() { byte |= 0b10000000 }

        self.set_carry(new_carry);
        self.set_negative(byte > 0x7f);
        self.set_zero(byte == 0);

        match i_addr {
            Some(v) => bus::write(memory, v, byte),
            None => self.a = byte,
        };

        self.cycles_used += cycles
    }


    fn rti(&mut self, memory: &mut [Segment], cycles: u8) 
    {
        self.sr = self.sr & 0x30 | (bus::pull_stack(memory, self) & 0xcf);

        let return_byte_lo: u8 = bus::pull_stack(memory, self);
        let return_byte_hi: u8 = bus::pull_stack(memory, self);

        self.pc = ((return_byte_hi as u16) << 8) + return_byte_lo as u16;

        self.cycles_used += cycles
    }


    fn rts(&mut self, memory: &mut [Segment], cycles: u8) 
    {
        let return_byte_lo: u8 = bus::pull_stack(memory, self);
        let return_byte_hi: u8 = bus::pull_stack(memory, self);

        self.pc = (((return_byte_hi as u16) << 8) + return_byte_lo as u16) + 1;

        self.cycles_used += cycles
    }


    fn sbc(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        let byte: u8 = bus::read(memory, i_addr); //the only difference between add and subtract is using the inverse of the byte to be added!
        let c_byte = !byte;

        let mut result: u16;

        match self.decimal_flag() 
        {
            true => //BCD
            {
                let mut half_carry: bool = false;
                let mut hi_adjust: u8 = 0;
                let mut lo_adjust: u8 = 0;

                let mut lo_nibble: u8 = (self.a & 0xf) + (c_byte & 0xf) + (self.carry_flag() as u8); 
                
                if lo_nibble > 9
                {
                    lo_adjust = 6;
                    half_carry = true;
                }

                let mut hi_nibble: u8 = ( (self.a >> 4) & 0xf ) + ( (c_byte>> 4) & 0xf ) + (half_carry as u8); 

                self.set_carry(hi_nibble > 9);
                if self.carry_flag()
                {
                    hi_adjust = 6;
                }

                //ALU result without decimal adjustments
                lo_nibble &= 0xf;
                hi_nibble &= 0xf;
                let alu_result: u8 = (hi_nibble << 4) + lo_nibble;

                self.set_zero(alu_result == 0);
                self.set_negative(alu_result > 0x7f);
                self.set_overflow((c_byte & 0x80 == self.a & 0x80) && (alu_result & 0x80 != c_byte & 0x80));

                //Final A result with adjustment
                lo_nibble = (lo_nibble + lo_adjust) & 0xf;
                hi_nibble = (hi_nibble + hi_adjust) & 0xf;
                result = u16::from((hi_nibble << 4) + lo_nibble);
            }
            false => //Normal binary
            {
                result = self.a as u16 + c_byte as u16 + self.carry_flag() as u16; // A + Byte + Carry

                self.set_carry(result > 0xff);

                if self.carry_flag()
                {
                    result &= 0xff;
                }

                self.set_overflow((c_byte & 0x80 == self.a & 0x80) && (result as u8 & 0x80 != c_byte & 0x80));
                self.set_zero(result == 0);
                self.set_negative(result > 0x7f);
            }
        }

        self.a = result as u8;

        self.cycles_used += cycles
    }


    fn sta(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        bus::write(memory, i_addr, self.a);

        self.cycles_used += cycles
    }


    fn stx(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        bus::write(memory, i_addr, self.x);

        self.cycles_used += cycles
    }


    fn sty(&mut self, memory: &mut [Segment], cycles: u8, i_addr: u16) 
    {
        bus::write(memory, i_addr, self.y);

        self.cycles_used += cycles
    }


    fn transfer(&mut self, origin: char, destination: char) 
    {
        let val: u8;

        match origin {
            'a' => val = self.a,
            'x' => val = self.x,
            'y' => val = self.y,
            's' => val = self.sp,
            _ => panic!("Invalid origin argument to self.transfer \n")
        };

        match destination {
            'a' => self.a = val,
            'x' => self.x = val,
            'y' => self.y = val,
            's' => self.sp = val,
            _ => panic!("Invalid destination argument to self.transfer \n")
        };

        self.cycles_used += 2;

        if destination != 's'
        {
            self.set_negative(val > 0x7f);
            self.set_zero(0 == val);
        }
    }
    

    pub fn carry_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1)
    }

    pub fn set_carry(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1 } else { self.sr &= !0b1 }
    }


    pub fn zero_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10)
    }

    pub fn set_zero(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10 } else { self.sr &= !0b10 }
    }

    
    pub fn interrupt_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b100)
    }
    
    pub fn set_interrupt(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b100 } else { self.sr &= !0b100 }
    }

    
    pub fn decimal_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1000)
    }
    
    pub fn set_decimal(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000 } else { self.sr &= !0b1000 }
    }

    
    pub fn break_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10000)
    }
    
    pub fn set_break(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10000 } else { self.sr &= !0b10000 }
    }

    
    pub fn overflow_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b1000000)
    }
    
    pub fn set_overflow(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b1000000 } else { self.sr &= !0b1000000 }
    }

    
    pub fn negative_flag(&mut self) -> bool
    {
        return 0 != (self.sr & 0b10000000)
    }
    
    pub fn set_negative(&mut self, flag: bool)
    {
        if flag { self.sr |= 0b10000000 } else { self.sr &= !0b10000000 }
    }


    pub fn irq(&mut self)
    {
        if !self.interrupt_flag()
        {
            self.external_irq = true;
        }
    }

    pub fn nmi(&mut self)
    {
        self.external_nmi = true;
    }
}
