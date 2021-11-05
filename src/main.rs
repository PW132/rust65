/* Rust65: an example 6502 system emulator in Rust
   Written by Peter Worthington, 2021 */

mod bus;
mod cpu;
mod op;

use crate::bus::Segment;
use crate::cpu::CpuStatus;
use std::io::{Read, Error, ErrorKind, Write, stdout};
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


    let mut dram_array: [u8; 0xdfff] = [0; 0xdfff]; //reserve 56KB of memory address space
    let dram: &mut[u8] = &mut dram_array[..];


    let memory: &mut[Segment] = //define memory map
    &mut[
        Segment {data: dram, start_addr: 0, write_enabled: true, read_enabled: true},
        Segment {data: rom, start_addr: 0xe000, write_enabled: false, read_enabled: true}
    ];

    let mut reg = CpuStatus::new(1000000); //create and initialize registers and other cpu state

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer


    print!("Startup complete! \n>");
    stdout().flush().unwrap();

    
    loop
    {
        if cpu_running //if true, let's run code
        {
            let check: Result<u8, String> = cpu::execute(memory, &mut reg); //execute an instruction, check for errors

            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                cpu::status_report(&reg);
                cpu_running = false;

                print!(">");
                std::io::stdout().flush().unwrap();
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
                {   let check: Result<u8, String> = cpu::execute(memory, &mut reg);
                    if check.is_err()
                    {
                        println!("{}",check.unwrap_err());
                    }
    
                    cpu::status_report(&reg); },

                "exit" => break, //exit command: close emulator
                _ => println!("What?")
            }
            print!(">");
            stdout().flush().unwrap();
        }
    }

    return;
}