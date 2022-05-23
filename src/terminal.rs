extern crate sdl2;

use crate::bus;
use crate::bus::Segment;

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
        let mut out_char: u8 = memory[OUT].data[DSP] & !0b10000000; //get byte and convert to uppercase ASCII
        if out_char == 0xd {
            out_char = 0xa;
        } //convert any Carriage Returns to Line Feeds

        buf.push(out_char); //add converted character to the text buffer
        memory[IN].data[DSP] &= !0b10000000; //clear bit 7 to let woz monitor know we got the byte
        println!("OUTPUT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
    }

    if input.is_some() {
        memory[IN].data[KBD] = input.unwrap() | 0b10000000;
        *input = None;

        memory[IN].data[KBDCR] |= 0b10000000;

        println!("INPUT!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
    }

    return;
}
