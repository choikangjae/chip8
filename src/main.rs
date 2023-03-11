const WIDTH: u8 = 64;
const HEIGHT: u8 = 32;

struct Processor {
    ram: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    display: [bool; 64 * 32],
    key: [u8; 16],
}

//delta somewhere

fn main() {
    //init processor.

    //load binary.

    //loop
        //opcode

        //render
}
