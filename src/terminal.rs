extern crate sdl2;

use crate::bus::Segment;

use sdl2::pixels::Color;
use sdl2::event::{Event, WindowEvent};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

use std::collections::VecDeque;

const KBD: usize = 0;
const KBDCR: usize = 1;
const DSP: usize = 2;
const DSPCR: usize = 3;

const IN: usize = 2;
const OUT: usize = 3;

pub fn pia(memory: &mut [Segment], buf: &mut VecDeque<u8>, input: &mut Option<char>) -> bool
{
    let mut printed: bool = false;

    if memory[IN].data[DSP] > 0x7f   //is bit 7 of DSP set?
    {
        let mut out_char: u8 = memory[OUT].data[DSP] & !0x80;     //get byte and convert to valid ASCII

        if out_char != 0x0                                      //make sure we're not passing a null character to the buffer
        {
            if out_char == 0xd                                        //convert any Carriage Returns to Line Feeds
            { 
                out_char = 0xa;
            }

            buf.push_back(out_char);                //add converted character to the text buffer
            scroll(buf);
        }

        memory[IN].data[DSP] &= !0x80;          //clear bit 7 to let woz monitor know we got the byte

        printed = true;
    }

    if input.is_some() {
        memory[IN].data[KBD] = input.unwrap().to_ascii_uppercase() as u8 | 0x80;
        *input = None;

        memory[IN].data[KBDCR] |= 0x80;

        //println!("in: {} {}", memory[IN].data[KBD], memory[IN].data[KBD] as char);
    }

    return printed;
}


pub fn scroll(buf: &mut VecDeque<u8>) //handle scrolling the display if the buffer is full
{
    let mut rows_used = 0;
    let mut characters_in_row = 0;

    for i in 0 .. buf.len() //start by measuring how many lines of text have been used, by going through the buffer looking for wraps and newlines
    {
        if buf[i] == 0xa || characters_in_row >= 39
        {
            rows_used += 1;
            characters_in_row = 0
        }
        else 
        {
            characters_in_row += 1    
        }
    }

    if rows_used >= 24 //if the screen is about to fill completely, then delete all of the first line, finishing at a wrap or newline
    {
        characters_in_row = 0;

        while characters_in_row <= 39
        {
            let removed_char = buf.pop_front();
            characters_in_row += 1;

            if removed_char.is_some()
            {
                if removed_char.unwrap() == 0xa { break; }
            }
        }
    }
}


pub fn render_screen(screen: &mut Canvas<Window>, texture_creator: &TextureCreator<WindowContext>, terminal_buf: &mut VecDeque<u8>, font: &Font)
{
    screen.clear();

    let str = &String::from_utf8_lossy(terminal_buf.make_contiguous());
    
    let text = font.render(str).blended_wrapped(Color::RGB(255, 255, 255), 560);
    
    if text.is_ok()
    {
        let text_texture = text.unwrap().as_texture(&texture_creator).unwrap();
        let text_dimensions = text_texture.query();
        screen.copy(&text_texture, None, Some(Rect::new(0,0,text_dimensions.width,text_dimensions.height)));
    }

    screen.present();
}