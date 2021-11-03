/* Rust65: an example 6502 system emulator in Rust
   Written by Peter Worthington, 2021 */

mod bus;
mod cpu;

use crate::bus::Segment;
use crate::cpu::CpuStatus;
use crate::cpu::execute;
use std::io::Read;
use std::io::{Error, ErrorKind, Write};
use std::fs::File;
use std::path::Path;
use std::panic;
use text_io::try_scan;
use text_io::read;

fn main() {
    println!("Starting emulator...");

    let rom_path = Path::new("applesoft-lite-0.4.bin"); //read ROM file and keep resident in an array
    let mut rom_file = match File::open(rom_path) 
    {
        Err(why) => panic!("couldn't open {}: {}", rom_path.display(), why),
        Ok(file) => file
    };
    let mut rom_array: [u8; 0x1fff] = [0; 0x1fff];
    rom_file.read(&mut rom_array);
    let rom: &mut[u8] = &mut rom_array[..];

    let mut dram_array: [u8; 0xdfff] = [0; 0xdfff]; //reserve 64KB of memory address space
    let dram: &mut[u8] = &mut dram_array[..];

    let memory: &[Segment] = //define memory map
    &[
        Segment {data: dram, start_addr: 0, write_enabled: true, read_enabled: true},
        Segment {data: rom, start_addr: 0xe000, write_enabled: false, read_enabled: true}
    ];

    let mut reg = CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, debug_text: true, clock_speed: 0}; //create and initialize registers

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer

    print!("Startup complete! \n>");
    std::io::stdout().flush().unwrap();

    loop
    {
        if cpu_running //if true, let's run code
        {
            let check: Result<bool, String> = execute(memory, &mut reg); //execute an instruction, check for errors
            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                cpu::status_report(&reg);
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
            
            match last_cmd.trim()
            {
                "run" => cpu_running = true, //run command: start running code
                "status" => cpu::status_report(&reg), //status command: get status of registers
                "step" => //step command: run a single operation
                {   let check: Result<bool, String> = execute(memory, &mut reg);
                    if check.is_err()
                    {
                        println!("{}",check.unwrap_err());
                    }
    
                    cpu::status_report(&reg);},
                "exit" => break, //exit command: close emulator
                _ => println!("What?")
            }
            print!(">");
            std::io::stdout().flush().unwrap();
        }
    }

    return;
}