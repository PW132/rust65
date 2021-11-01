fn main() {
    println!("Starting emulator...");

    let dram: [u8; 0xffff]; //reserve 64KB of memory address space

    let (a, x, y): (u8, u8, u8) = (0, 0, 0); //create Accumulator and X + Y index registers
    let pc: u16 = 0xfffc; //create program counter
    let sr: u8 = 0b00100100;

    println!("{0}, {1}, {2}", a, x, y);
}
