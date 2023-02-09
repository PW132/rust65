/* Rust65: an example 6502 system emulator in Rust
Written by Peter Worthington, 2023 */

mod bus;
mod cpu;
mod op;
mod terminal;

extern crate sdl2;
extern crate spin_sleep;

use crate::bus::Segment;
use crate::cpu::CpuStatus;

use std::io::{Read, Error, ErrorKind, Write, stdout};
use std::fs::File;
use std::path::Path;
use std::{panic, time};
use std::collections::VecDeque;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::event::{Event, WindowEvent};


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


    let mut dram_array: [u8; 0x7fff] = [0; 0x7fff]; //reserve 32KB of memory address space
    let dram: &mut[u8] = &mut dram_array[..];


    let mut pia_in_array: [u8; 3] = [0; 3];         //set up Peripheral Interface Adapter registers
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

    let mut cycle_total: i32 = 0;

    let mut terminal_buf: VecDeque<u8> = VecDeque::new();
    let mut i_char: Option<u8> = None;

    //Begin initializing SDL2 window...

    let sdl_context = sdl2::init().unwrap();                        //initialize all relevant SDL2 systems
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_subsystem = sdl2::ttf::init().unwrap();

    let font_path = Path::new("PrintChar21.ttf");                           //Get the Apple font
    let font = ttf_subsystem.load_font(font_path, 16).unwrap();
                                                                                    //create a window and canvas
    let window = video_subsystem.window("TV Terminal (rust65 Apple I)", 560, 384)
        .position_centered()
        .build()
        .unwrap();

    let mut screen = window.into_canvas()
        .present_vsync()
        .build()
        .unwrap();

    let texture_creator = screen.texture_creator();

    screen.set_draw_color(Color::RGB(0,0,0));
    screen.clear();
    screen.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    //Everything started up OK

    println!("Startup complete!");

    loop                //Main execution loop
    {

        for event in event_pump.poll_iter() //handle SDL events (typing in monitor window, close, etc)
        {
            match event
            {
                Event::Quit {..} => return,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => 
                {
                    nm65.running = false; 
                    print!("Emulation paused, dropping into monitor \n>");
                    stdout().flush();
                },
                Event::Window { win_event: WindowEvent::FocusGained, .. } => video_subsystem.text_input().start(),
                Event::Window { win_event: WindowEvent::FocusLost, .. } => video_subsystem.text_input().stop(),
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => i_char = Some(0xd),
                Event::TextInput { text: t, .. } => i_char = Some(t.as_bytes()[0]),
                _ => ()
            }
        }

        if nm65.running //if true, let's run 6502 code
        {
            let now = time::Instant::now();
            let check: Result<u8, String> = nm65.execute(memory); //execute an instruction, check for errors

            if check.is_err()
            {
                println!("{}",check.unwrap_err());
                nm65.status_report();
                nm65.running = false;                                        //stop running if something goes wrong

                print!(">");
                stdout().flush();
            }
            else                                                            //if the instruction executed OK...
            {
                let cycles_just_used: u8 = check.unwrap();                                          //count cycles used by the completed
                if nm65.debug_text {println!("Instruction used {} cycles...", cycles_just_used)};   //instruction, add them to a running total
                cycle_total += i32::from(cycles_just_used);

                if cycle_total > (1000000/1200)                                                     //should we update peripherals this frame?
                {
                    cycle_total = 0;                                                                //reset count
                    terminal::pia(memory, &mut terminal_buf, &mut i_char);                                       //update the peripherals (keyboard, display)
                    terminal::render_screen(&mut screen, &texture_creator, &mut terminal_buf, &font);
                }

                //sleep for the amount of time dictated by cycles taken and the CPU speed

                let mut wait_time = time::Duration::from_nanos(cycles_just_used as u64 * nm65.clock_time);
                let spent_time = now.elapsed();
                
                if wait_time > spent_time
                {
                    wait_time -= spent_time; 
                    spin_sleep::sleep(wait_time); 
                }
            }
        }

        else        //CPU is paused, drop into interactive monitor
        {   
            let continue_loop: bool = nm65.debug_mode(memory);
            if !continue_loop { return }

            terminal::pia(memory, &mut terminal_buf, &mut i_char);
            terminal::render_screen(&mut screen, &texture_creator, &mut terminal_buf, &font);
        }
    }
}