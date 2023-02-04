/* Rust65: an example 6502 system emulator in Rust
   Written by Peter Worthington, 2021 */

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
   
   use sdl2::keyboard::Keycode;
   use sdl2::pixels::Color;
   use sdl2::event::{Event, WindowEvent};
   use sdl2::rect::Rect;
   use sdl2::render::{Canvas, TextureCreator};
   use sdl2::ttf::Font;
   use sdl2::video::{Window, WindowContext};
   
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
   
   
       let mut cpu_running: bool = true;
       let mut cycle_total: i32 = 0;
       let mut last_cmd: String; //the command line buffer
   
       let mut terminal_buf: Vec<u8> = Vec::new();
       let mut i_char: Option<u8> = None;
   
       //Begin initializing SDL2 window...
   
       let sdl_context = sdl2::init().unwrap();                        //initialize all relevant SDL2 systems
       let video_subsystem = sdl_context.video().unwrap();
       let ttf_subsystem = sdl2::ttf::init().unwrap();
   
       let font_path = Path::new("PrintChar21.ttf");                           //Get the Apple font
       let font = ttf_subsystem.load_font(font_path, 16).unwrap();
                                                                                      //create a window and canvas
       let window = video_subsystem.window("TV Terminal (rust65 Apple I)", 640, 480)
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
                       cpu_running = false; 
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
   
           if cpu_running //if true, let's run 6502 code
           {
               let now = time::Instant::now();
               let check: Result<u8, String> = nm65.execute(memory); //execute an instruction, check for errors
   
               if check.is_err()
               {
                   println!("{}",check.unwrap_err());
                   nm65.status_report();
                   cpu_running = false;                                        //stop running if something goes wrong
   
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
                       terminal::render_screen(&mut screen, &texture_creator, &terminal_buf, &font);
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
               last_cmd = read!("{}\n");       //get text input and store it whole
               
               match last_cmd.trim()           //check for single-word commands with no arguments
               {
                   "verbose" => nm65.debug_text = !nm65.debug_text, //enable or disable debug commentary
                   "run" => cpu_running = true,                     //run command: start running code
                   "reset" => nm65.reset = true,                    //reset command: reset the CPU
                   "status" => nm65.status_report(),      //status command: get status of registers
   
                   "step" =>                                        //step command: run a single operation and display results
                   {   let check: Result<u8, String> = nm65.execute(memory);
                       if check.is_err()
                       {
                           println!("{}",check.unwrap_err());
                       }
                       else 
                       {
                           let cycles_taken: u8 = check.unwrap();
                           if nm65.debug_text {println!("Instruction used {} cycles...", cycles_taken)};
                           terminal::pia(memory, &mut terminal_buf, &mut i_char);
                           terminal::render_screen(&mut screen, &texture_creator, &terminal_buf, &font);
                       }
       
                       nm65.status_report(); 
                   },
   
                   "exit" => return,                                //exit command: close emulator
                   _ => println!("What?")
               }
               print!(">");
               stdout().flush();
           }
       }
   }