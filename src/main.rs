/* Rust65: an example 6502 system emulator in Rust
   Written by Peter Worthington, 2021 */

mod bus;
mod cpu;

use crate::cpu::CpuStatus;
use crate::cpu::execute;
use std::io::{Error, ErrorKind};
use std::panic;
use text_io::try_scan;
use text_io::read;

fn main() {
    println!("Starting emulator...");

    let mut memory: [u8; 0xffff] = [0; 0xffff]; //reserve 64KB of memory address space

    let mut reg = CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100, sp:0, debug_text: true, clock_speed: 0}; //create and initialize registers

    let mut cpu_running: bool = false;
    let mut last_cmd: String; //the command line buffer

    loop
    {
        if cpu_running //if true, let's run code
        {
            let check: Result<bool, String> = execute(&mut memory, &mut reg); //execute an instruction, check for errors
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
            
            if last_cmd.trim() == "run"
            {
                cpu_running = true;
                continue;
            }
            else if last_cmd.trim() == "step"
            {
                let check: Result<bool, String> = execute(&mut memory, &mut reg); //execute an instruction, check for errors
                if check.is_err()
                {
                    println!("{}",check.unwrap_err());
                }

                cpu::status_report(&reg);
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
