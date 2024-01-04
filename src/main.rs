/* Rust65: an example 6502 system emulator in Rust
Written by Peter Worthington, 2023 */

mod bus;
mod cpu;
mod terminal;

extern crate sdl2;
extern crate spin_sleep;

use config::Config;

use crate::bus::Segment;
use crate::cpu::CpuStatus;

use std::io::{Read, Error, ErrorKind, Write, stdout};
use std::fs::File;
use std::path::Path;
use std::str::Chars;
use std::{panic, time};
use std::collections::VecDeque;
use std::collections::HashMap;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::event::{Event, WindowEvent};


fn main() {
    println!("Starting emulator...");

    let settings = Config::builder().add_source(config::File::with_name("Settings")).build().unwrap();
    let unpacked_settings = settings.try_deserialize::<HashMap<String, String>>().unwrap();

    let rom_path = Path::new(unpacked_settings.get("rom_filename").unwrap()); //read ROM file and keep resident in an array
    let mut rom_file = match File::open(rom_path) 
    {
        Err(why) => panic!("couldn't open {}: {}", rom_path.display(), why),
        Ok(file) => file
    };
    let mut rom_array: [u8; 0x2000] = [0; 0x2000];
    rom_file.read(&mut rom_array);
    let rom: &mut[u8] = &mut rom_array[..];


    let mut dram_array: [u8; 0x8000] = [0; 0x8000]; //reserve 32KB of memory address space
    let dram: &mut[u8] = &mut dram_array[..];


    let mut pia_in_array: [u8; 3] = [0; 3];         //set up Peripheral Interface Adapter registers
    let mut pia_out_array: [u8; 3] = [0; 3];
    let pia_in: &mut[u8] = &mut pia_in_array[..];
    let pia_out: &mut[u8] = &mut pia_out_array[..];


    let memory: &mut[Segment] =                     //define memory map
    &mut[
        Segment::new(dram, 0, true, true),
        Segment::new(rom, 0xe000, false, true),
        Segment::new(pia_in, 0xd010, false, true),
        Segment::new(pia_out, 0xd010, true, false)
    ];

    let clock: u64 = unpacked_settings.get("cpu_speed").unwrap().parse().unwrap();
    let pia_refresh: u64 = clock / unpacked_settings.get("terminal_speed").unwrap().parse::<u64>().unwrap();                     //The real Apple 1 terminal updated every 16.7 milliseconds. clock / 60 provides a close approximate to the original, diving clock by higher values provides faster print speeds
    let video_refresh: u64 = 1000000000 / 120;

    let mut nm65 = CpuStatus::new(clock); //create and initialize registers and other cpu state

    let mut cycle_total: u64 = 0;
    let mut frame_time: time::Duration = time::Duration::ZERO;

    let mut terminal_buf: VecDeque<u8> = VecDeque::new();
    let mut i_char: Option<char> = None;
    let mut pasted_text: String = "".to_string();
    let mut pasted_chars: Chars = pasted_text.chars();
    let mut pasting: bool = false;
    let mut printing: bool = false;

    //Begin initializing SDL2 window...

    let sdl_context = sdl2::init().unwrap();                        //initialize all relevant SDL2 systems
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_subsystem = sdl2::ttf::init().unwrap();

    let resolution_multiplier: u32 = unpacked_settings.get("resolution_multiplier").unwrap().parse().unwrap();

    let font_path = Path::new("PrintChar21.ttf");                           //Get the Apple font
    let font = ttf_subsystem.load_font(font_path, 8 * resolution_multiplier as u16).unwrap();
                                                                                    //create a window and canvas
    let window = video_subsystem.window("TV Terminal (rust65 Apple I)", 280 * resolution_multiplier, 192 * resolution_multiplier)
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
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => if !pasting { i_char = Some(0xd as char) },
                Event::KeyDown { keycode: Some(Keycode::Insert), .. } => 
                    if video_subsystem.clipboard().has_clipboard_text() 
                    {
                        pasted_text = video_subsystem.clipboard().clipboard_text().unwrap();
                        pasted_chars = pasted_text.chars();
                        pasting = true;
                    },
                Event::TextInput { text: t, .. } => if !pasting { i_char = t.chars().next() },
                _ => ()
            }
        }

        if nm65.running                                           //if true, let's run 6502 code
        {
            let instruction_time = time::Instant::now();
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
                cycle_total += u64::from(cycles_just_used);

                if cycle_total > pia_refresh                                                    //should we update peripherals this frame?
                {

                    if pasting && !printing
                    {
                        let p_next: Option<char> = pasted_chars.next();
                        match p_next
                        {
                            Some(c) => i_char = Some(c),
                            None => pasting = false
                        }
                    }

                    cycle_total = 0;                                                                //reset count
                    printing = terminal::pia(memory, &mut terminal_buf, &mut i_char);        //update the peripherals (keyboard, display)
                }

                //sleep for the amount of time dictated by cycles taken and the CPU speed

                let mut wait_time = time::Duration::from_nanos(cycles_just_used as u64 * nm65.clock_time);
                let spent_time = instruction_time.elapsed();
                
                if wait_time > spent_time
                {
                    wait_time -= spent_time; 
                    spin_sleep::sleep(wait_time); 
                }

                frame_time += spent_time + wait_time;

                if frame_time >= time::Duration::from_nanos(video_refresh)
                {
                    terminal::render_screen(&mut screen, &texture_creator, &mut terminal_buf, &font);
                    frame_time = time::Duration::ZERO;
                }
            }
        }

        else        //CPU is paused, drop into interactive monitor
        {   
            let continue_loop: bool = nm65.debug_mode(memory);
            if !continue_loop { return }

            printing = terminal::pia(memory, &mut terminal_buf, &mut i_char);
            terminal::render_screen(&mut screen, &texture_creator, &mut terminal_buf, &font);
        }
    }
}