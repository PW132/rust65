/* Rust65: an example 6502 system emulator in Rust
   Written by Peter Worthington, 2021 */

mod bus;
mod cpu;
mod op;
mod terminal;

use crate::bus::Segment;
use crate::cpu::CpuStatus;
use std::io::{Read, Error, ErrorKind, Write, stdout};
use std::fs::File;
use std::path::Path;
use std::{panic, time, thread};
use text_io::{try_scan, read};

fn main() {
    println!("Starting emulator...");


    let rom_path = Path::new("applesoft-lite-0.4.bin"); //read ROM file and keep resident in an array
    let mut rom_file = match File::open(rom_path) 
    {
        Err(why) => panic!("couldn't open {}: {}", rom_path.display(), why),
        Ok(file) => file
    };
    let mut rom_array: [u8; 0x1fff] = [0; 0x1fff];
    let _s = rom_file.read(&mut rom_array);
    let rom: &mut[u8] = &mut rom_array[..];


    let mut dram_array: [u8; 0x7fff] = [0; 0x7fff]; //reserve 32KB of memory address space
    let dram: &mut[u8] = &mut dram_array[..];


    let mut pia_in_array: [u8; 3] = [0; 3];
    let mut pia_out_array: [u8; 3] = [0; 3];
    let pia_in: &mut[u8] = &mut pia_in_array[..];
    let pia_out: &mut[u8] = &mut pia_out_array[..];


    let memory: &mut[Segment] = //define memory map
    &mut[
        Segment::new(dram, 0, true, true),
        Segment::new(rom, 0xe000, false, true),
        Segment::new(pia_in, 0xd010, false, true),
        Segment::new(pia_out, 0xd010, true, false)
    ];


    let mut nm65 = CpuStatus::new(1000000); //create and initialize registers and other cpu state

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer


    print!("Startup complete! \n>");
    stdout().flush().unwrap();

    
    loop                //Main execution loop
    {
        if cpu_running //if true, let's run 6502 code
        {
            let now = time::Instant::now();
            let check: Result<u8, String> = cpu::execute(memory, &mut nm65); //execute an instruction, check for errors

            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                cpu::status_report(&nm65);
                cpu_running = false;                                        //stop running if something goes wrong

                print!(">");
                std::io::stdout().flush().unwrap();
            }
            else 
            {
                let cycles_taken: u8 = check.unwrap();
                if nm65.debug_text {println!("Instruction used {} cycles...", cycles_taken)};

                //if the instruction executed successfully, sleep for the amount of time dictacted by cycles taken and the CPU speed

                let mut wait_time = time::Duration::from_nanos(cycles_taken as u64 * nm65.clock_time);
                let spent_time = now.elapsed();
                
                if wait_time > spent_time
                {
                    wait_time -= spent_time; 
                    thread::sleep(wait_time); 
                }
                else
                {
                    println!("slow!")
                }
            }
        }

        else        //CPU is paused, drop into interactive monitor
        {   
            last_cmd = read!("{}\n");       //get text input and store it whole
            
            match last_cmd.trim() //check for single-word commands with no arguments
            {
                "verbose" => nm65.debug_text = !nm65.debug_text,
                "run" => cpu_running = true, //run command: start running code
                "reset" => nm65.reset = true,
                "status" => cpu::status_report(&nm65), //status command: get status of registers

                "step" =>                   //step command: run a single operation and display results
                {   let check: Result<u8, String> = cpu::execute(memory, &mut nm65);
                    if check.is_err()
                    {
                        println!("{}",check.unwrap_err());
                    }
                    else 
                    {
                        let cycles_taken: u8 = check.unwrap();
                        if nm65.debug_text {println!("Instruction used {} cycles...", cycles_taken)};
                    }
    
                    cpu::status_report(&nm65); 
                },

                "exit" => break,            //exit command: close emulator
                _ => println!("What?")
            }
            print!(">");
            stdout().flush().unwrap();
        }
    }

    return;
}