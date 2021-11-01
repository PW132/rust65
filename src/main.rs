mod mem;

use std::panic;
use text_io::scan;
use text_io::read;
struct CpuStatus
{
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sr: u8
}

fn main() {
    println!("Starting emulator...");

    let mut dram: [u8; 0xffff] = [0; 0xffff]; //reserve 64KB of memory address space

    let mut reg = CpuStatus {a:0, x:0, y:0, pc:0xfffc, sr:0b00100100};

    let mut last_cmd: String;

    //println!("{0}, {1}, {2}", reg.a, reg.x, reg.y);
    //mem::write(&mut dram, 0, 0b01010101);
    //println!("{}", mem::read(&dram, 0));

    loop
    {
        last_cmd = read!("{}\n");
        
        if last_cmd.trim() != "exit"
        {
            println!("{}", last_cmd);
            continue;
        }
        break;
    }

    return;
}
