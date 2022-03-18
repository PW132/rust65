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
        Segment {data: dram, start_addr: 0, write_enabled: true, read_enabled: true},
        Segment {data: pia_in, start_addr: 0xd010, write_enabled: false, read_enabled: true},
        Segment {data: pia_out, start_addr: 0xd010, write_enabled: true, read_enabled: false},
        Segment {data: rom, start_addr: 0xe000, write_enabled: false, read_enabled: true}
    ];


    let mut reg = CpuStatus::new(1000000); //create and initialize registers and other cpu state
    //reg.debug_text = true;

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer


    print!("Startup complete! \n>");
    stdout().flush().unwrap();

    
    loop                //Main execution loop
    {
        if cpu_running //if true, let's run 6502 code
        {
            let check: Result<u8, String> = cpu::execute(memory, &mut reg); //execute an instruction, check for errors

            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                cpu::status_report(&reg);
                cpu_running = false;                                        //stop running if something goes wrong

                print!(">");
                std::io::stdout().flush().unwrap();
            }
            else 
            {
                let cycles_taken: u8 = check.unwrap();
                if reg.debug_text {println!("Instruction used {} cycles...", cycles_taken)};

                let wait_time = time::Duration::from_millis(cycles_taken as u64 * (1000/reg.clock_speed as u64));
                let now = time::Instant::now();

                thread::sleep(wait_time);
                assert!(now.elapsed() >= wait_time);
            }
        }
        else        //CPU is paused, drop into interactive monitor
        {   
            last_cmd = read!("{}\n");       //get text input and store it whole
            
            match last_cmd.trim()
            {
                "verbose" => reg.debug_text = !reg.debug_text,
                "run" => cpu_running = true, //run command: start running code
                "reset" => reg.pc = 0xfffc,
                "status" => cpu::status_report(&reg), //status command: get status of registers

                "step" =>                   //step command: run a single operation
                {   let check: Result<u8, String> = cpu::execute(memory, &mut reg);
                    if check.is_err()
                    {
                        println!("{}",check.unwrap_err());
                    }
    
                    cpu::status_report(&reg); 
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