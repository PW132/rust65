extern crate sdl2;

use crate::bus;
use crate::bus::Segment;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::event::{Event, WindowEvent};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};

const KBD: usize = 0;
const KBDCR: usize = 1;
const DSP: usize = 2;
const DSPCR: usize = 3;

const IN: usize = 2;
const OUT: usize = 3;

pub fn pia(memory: &mut [Segment], buf: &mut Vec<u8>, input: &mut Option<u8>) {
    if memory[IN].data[DSP] > 127
    //is bit 7 of DSP set?
    {
        let mut out_char: u8 = memory[OUT].data[DSP] & !0b10000000; //get byte and convert to valid ASCII
        if out_char == 0xd {
            out_char = 0xa;
        } //convert any Carriage Returns to Line Feeds

        buf.push(out_char); //add converted character to the text buffer
        memory[IN].data[DSP] &= !0b10000000; //clear bit 7 to let woz monitor know we got the byte
    }

    if input.is_some() {
        memory[IN].data[KBD] = input.unwrap().to_ascii_uppercase() | 0b10000000;
        *input = None;

        memory[IN].data[KBDCR] |= 0b10000000;
    }

    return;
}

pub fn render_screen(screen: &mut Canvas<Window>, texture_creator: &TextureCreator<WindowContext>, terminal_buf: &Vec<u8>, font: &Font)
{
    screen.clear();

    let text = font.render(&String::from_utf8_lossy(&terminal_buf))
        .blended_wrapped(Color::RGB(255, 255, 255), 640);
    if text.is_ok()
    {
        let text_texture = text.unwrap().as_texture(&texture_creator).unwrap();
        let text_dimensions = text_texture.query();
        screen.copy(&text_texture, None, Some(Rect::new(0,0,text_dimensions.width,text_dimensions.height)));
    }

    screen.present();
}