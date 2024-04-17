pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const REGISTER_LENGTH: usize = 16;
const STACK_SIZE: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_register: [u8; REGISTER_LENGTH],
    i_register: u16,
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    delay_timer: u8,
    sound_timer: u8,
}
impl Emulator {
    fn new() -> Self {
        let mut emulator = Self {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_register: [0; REGISTER_LENGTH],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        };
        emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emulator
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    fn reset(&mut self) {
        self.program_counter = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_register = [0; REGISTER_LENGTH];
        self.i_register = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // fetch instruction
        let op = self.fetch();
        // decode instruction
        // execute instruction
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        // fetches the opcode ( all Chip-8 opcodes are exactly 2 bytes)
        let upper_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte: u16 = self.ram[(self.program_counter + 1) as usize] as u16;
        let op = (upper_byte << 8) | lower_byte;
        self.program_counter += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //beep
                todo!();
            }
            self.sound_timer -= 1;
        }
    }

    fn execute(&mut self, op: u16) {
        // separate out hex digits (bytes) of the opcode
        let first_byte = (op & 0xF000) >> 12;
        let second_byte = (op & 0xF000) >> 8;
        let third_byte = (op & 0xF000) >> 4;
        let fourth_byte = op & 0xF000;
        // figure out what opcode it is
        match (first_byte, second_byte, third_byte, fourth_byte) {
            // 0000 - NOP (no op)
            (0, 0, 0, 0) => return,
            // 00E0 - CLS (clear screen)
            (0, 0, 0xe, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // 00EE - RET (return from subroutine)
            (0, 0, 0xe, 0xe) => {
                // move the program counter to the specified address and resume execution from there
                let return_address = self.pop();
                self.program_counter = return_address;
            }
            // 1NNN - JMP NNN (jump)
            (1, _, _, _) => {
                let jump_address = op & 0xfff; // NNN
                self.program_counter = jump_address;
            }
            // 2NNN - CALL NNN (call subroutine)
            (2, _, _, _) => {
                let call_address = op & 0xfff; // NNN
                self.push(self.program_counter);
                self.program_counter = call_address;
            }
            // 3XNN - SKIP NEXT IF VX == NN (if equals)
            (3, _, _, _) => {
                let x = second_byte as usize;
                let nn = (op & 0xff) as u8; // NN
                if self.v_register[x] == nn {
                    self.program_counter += 2;
                }
            }
            // 4XNN - SKIP NEXT IF VX == NN (if not equal)
            (4, _, _, _) => {
                let x = second_byte as usize;
                let nn = (op & 0xff) as u8; // nn
                if self.v_register[x] != nn {
                    self.program_counter += 2;
                }
            }

            // 5XY0 - SKIP NEXT IF VX == VY (comparison equals)
            (5, _, _, 0) => {
                let x = second_byte as usize;
                let y = third_byte as usize;
                if self.v_register[x] == self.v_register[y] {
                    self.program_counter += 2;
                }
            }
            // 6XNN - VX == NN
            (6, _, _, _) => {}

            // 6XNN - VX = NN (set v register at the second digit X to the provide value NN)
            (_, _, _, _) => unimplemented!("Opcode not yet implemented: {}", op),
        }
    }
}
