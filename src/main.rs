static dram: [u8; 0xffff] = [0; 0xffff]; //reserve 64KB of memory address space

static a: u8 = 0; //create Accumulator and X + Y index registers
static x: u8 = 0;
static y: u8 = 0;

static pc: u16 = 0xfffc; //create program counter
static sr: u8 = 0b00100100;

mod mem;

fn main() {
    println!("Starting emulator...");

    

    println!("{0}, {1}, {2}", a, x, y);
}
