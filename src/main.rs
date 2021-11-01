mod bus;

use std::io::{Error, ErrorKind};
use std::panic;
use text_io::try_scan;
use text_io::read;
struct CpuStatus
{
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sr: u8,
    sp: u8
}

fn execute<'a>(memory: &'a mut[u8; 0xffff], reg: &'a mut CpuStatus) -> Result<bool, String> //runs a single CPU instruction, returns errors if there are any
{
    if reg.pc == 0xfffc
    {
        let lo_byte : u8 = bus::read(&memory,0xfffc); //retrieve reset vector from ROM
        let hi_byte : u8 = bus::read(&memory,0xfffd);

        reg.pc = lo_byte as u16 + (hi_byte as u16 * 256); //set new program counter at reset routine

        println!("Starting program execution at {}", reg.pc);
    }

    let opcode: u8 = bus::read(&memory, reg.pc); //get the opcode

    match opcode //which instruction is it?
    {
        1 => println!("eughh"),
        other => return Err(format!("Unrecognized opcode {}! Halting execution...", other)) //whoops! invalid opcode
    }

    Ok(true)
}

fn main() {
    println!("Starting emulator...");

    let mut memory: [u8; 0xffff] = [0; 0xffff]; //reserve 64KB of memory address space

    let mut reg = CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0}; //create and initialize registers

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer

    //println!("{0}, {1}, {2}", reg.a, reg.x, reg.y);
    //bus::write(&mut memory, 0, 0b01010101);
    //println!("{}", bus::read(&memory, 0));

    loop
    {
        if cpu_running //if true, let's run code
        {
            let check: Result<bool, String> = execute(&mut memory, &mut reg); //execute an instruction, check for errors
            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                cpu_running = false;
            }
            else 
            {
                if !check.unwrap_or(true)
                {
                    println!("Pausing program execution...");
                    cpu_running = false;
                }
            }
        }
        else //CPU is paused, drop into interactive monitor
        {   
            last_cmd = read!("{}\n"); //get text input and store it whole
            
            if last_cmd.trim() == "run"
            {
                cpu_running = true;
                continue;
            }
            else if last_cmd.trim() == "exit"
            {
                break;
            }
            println!("What?");
        }
    }

    return;
}
