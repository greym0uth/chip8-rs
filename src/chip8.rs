extern crate rand;

use rand::Rng;

static FONT_SPRITES: [u8; 16 * 5] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Chip8 {
    I: u16, // Memory address register
    M: [u8; 4096], // RAM
    S: [u16; 16], // Stack
    V: [u8; 16], // Registers
    pc: u16, // Program counter
    sp: u8, // Stack pointer
    dt: u8, // Display timer
    st: u8, // Sound timer
    input: [bool; 16], // Input buffer
    pub display: [u64; 32],
    wait: bool, // Whether the chip is halted for input
    store_input_at: u8, // Where to store input after halt
    rng: rand::prelude::ThreadRng, // A RNG thread
}

impl Chip8 {
    pub fn init(&mut self) {
        // Load fonts into memory starting at 0x0
        for index in 0..FONT_SPRITES.len() {
            self.M[index] = FONT_SPRITES[index];
        }

        // Create the rng
        self.rng = rand::thread_rng();
    }

    // Load a program into RAM
    pub fn load(&mut self, program: &[u8]) {
        for index in 0..program.len() {
            self.M[0x200 + index] = program[index];
        }
    }

    // Update the input buffer
    pub fn update_input(&mut self, new_input: [bool; 16]) {
        for key in 0..16 {
            if self.input[key] != new_input[key] {
                self.input[key] = new_input[key];

                // If waiting for input update register with key pressed and continue execution
                if new_input[key] && self.wait {
                    self.V[self.store_input_at as usize] = key as u8;
                    self.store_input_at = 0;
                    self.wait = false;
                }
            }
        }
    }

    pub fn cycle(&mut self) {
        if self.wait { return }

        let opcode: u16 = (self.M[self.pc as usize] as u16) << 8 | self.M[(self.pc + 1) as usize] as u16;

        match opcode & 0xf000 {
            0x0000 => {
                match opcode & 0x00ff {
                    0x00e0 => self.clear_screen(), // 00e0: Clear screen
                    0x00ee => self.return_from_sub(), // 00ee: Return from subroutine,
                    _ => {}
                }
            },
            0x1000 => self.jump(opcode & 0x0fff), // Jump to location nnn
            0x2000 => self.call(opcode & 0x0fff), // 2nnn: Call subroutine at nnn,
            0x3000 => self.skip_if_reg_equals_byte(((opcode & 0x0f00) >> 8) as u8, (opcode & 0x00ff) as u8), // 3xkk: Skip next instruction if Vx == kk
            0x4000 => self.skip_if_reg_not_equals_byte(((opcode & 0x0f00) >> 8) as u8, (opcode & 0x00ff) as u8), // 4xkk: Skip next instruction if Vx != kk
            0x5000 => self.skip_if_reg_equals_reg(((opcode & 0x0f00) >> 8) as u8, ((opcode & 0x00f0) >> 4) as u8), // 5xy0: Skip next instruction if Vx = Vy
            0x6000 => self.set_register(((opcode & 0x0f00) >> 8) as u8, (opcode & 0x00ff) as u8),  // 6xkk: Set Vx to kk
            0x7000 => self.add_to_register(((opcode & 0x0f00) >> 8) as u8, (opcode & 0x00ff) as u8), // 7xkk: Set Vx = Vx + kk
            0x8000 => {
                let x = ((opcode & 0x0f00) >> 8) as u8;
                let y = ((opcode & 0x00f0) >> 4) as u8;

                match opcode & 0x000f {
                    0x0000 => self.copy_to_register(x, y), // 8xy0: Set Vx = Vy
                    0x0001 => self.or_with_register(x, y), // 8xy1: Set Vx = Vx OR Vy
                    0x0002 => self.and_with_register(x, y), // 8xy2: Set Vx = Vx AND Vy
                    0x0003 => self.xor_with_register(x, y), // 8xy3: Set Vx = Vx XOR Vy
                    0x0004 => self.add_registers(x, y), // 8xy4: Set Vx = Vx + Vy, set VF = carry
                    0x0005 => self.sub(x, y), // 8xy5: Set Vx = Vx - Vy, set VF = NOT borrow
                    0x0006 => self.shift_right(x), // 8xy6: Set Vx = Vx SHR 1
                    0x0007 => self.sub_reverse(x, y), // 8xy7: Set Vx = Vy - Vx, set VF = NOT borrow
                    0x000e => self.shift_left(x), // 8xye: Set Vx = Vx SHL 1
                    _ => {}
                }
            },
            0x9000 => self.skip_if_reg_not_equals_reg(((opcode & 0x0f00) >> 8) as u8, ((opcode & 0x00f0) >> 4) as u8), // 9xy0: Skip next instruction if Vx != Vy
            0xa000 => self.set_i(opcode & 0x0fff), // annn: Sets I to the address nnn
            0xb000 => self.jump((opcode & 0x0fff) + (self.V[0] as u16)), // bnnn: Jump to location nnn + V0
            0xc000 => self.random(((opcode & 0x0f00) >> 8) as u8, (opcode & 0x00ff) as u8), // cxkk: Set Vx = random byte AND kk
            0xd000 => self.update_display(((opcode & 0x0f00) >> 8) as u8, ((opcode & 0x00f0) >> 4) as u8, (opcode & 0x000f) as u8), // dxyn: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            0xe000 => {
                let x = ((opcode & 0x0f00) >> 8) as u8;
                match opcode & 0x00ff {
                    0x009e => self.skip_if_pressed(x), // ex9e: Skip next instruction if key with the value of Vx is pressed
                    0x00A1 => self.skip_if_not_pressed(x), // exa1: Skip next instruction if key with the value of Vx is not pressed
                    _ => {}
                }
            },
            0xf000 => {
                let x = ((opcode & 0x0f00) >> 8) as u8;
                match opcode & 0x00ff {
                    0x0007 => self.set_reg_to_dt(x), // fx07: Set Vx = delay timer value
                    0x000a => self.wait_for_input(x), // fx0a: Wait for a key press, store the value of the key in Vx
                    0x0015 => self.set_dt(x), // fx15: Set delay timer = Vx
                    0x0018 => self.set_st(x), // fx18: Set sound timer = Vx
                    0x001E => self.i_plus_reg(x), // fx1e: Set I = I + Vx
                    0x0029 => self.set_i_digit_sprite(x), // fx29: Set I = location of sprite for digit Vx
                    0x0033 => self.bcd(x), // fx33: Store BCD representation of Vx in memory locations I, I+1, and I+2
                    0x0055 => self.store_regs_through(x), // fx55: Store registers V0 through Vx in memory starting at location I
                    0x0065 => self.read_to_regs(x), // fx65: Read registers V0 through Vx from memory starting at location I
                    _ => { println!("Opcode {:X} not found.", opcode) }
                }
            },
            _ => {}
        }

        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // TODO: BEEP
            }
            self.st -= 1;
        }
    }

    fn add_registers(&mut self, x: u8, y: u8) {
        let (value, carry) = self.V[x as usize].overflowing_add(self.V[y as usize]);
        if carry { self.V[0xf] = 1; }
        self.V[x as usize] = value;
        self.pc += 2;
    }
    
    fn add_to_register(&mut self, x: u8, byte: u8) {
        let (value, _c) = self.V[x as usize].overflowing_add(byte);
        self.V[x as usize] = value;
        self.pc += 2;
    }

    fn and_with_register(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] & self.V[y as usize];
        self.pc += 2;
    }

    fn bcd(&mut self, x: u8) {
        let digit = self.V[x as usize];
        let hundreds = digit / 100;
        self.M[self.I as usize] = hundreds as u8;
        self.M[self.I as usize + 1] = ((digit / 10) - (hundreds * 10)) as u8;
        self.M[self.I as usize + 2] = (digit % 10) as u8;
        self.pc += 2;
    }

    fn call(&mut self, loc: u16) {
        self.S[self.sp as usize] = self.pc + 2;
        self.sp += 1;
        self.pc = loc;
    }

    fn clear_screen(&mut self) {
        self.display = [0; 32];
        self.pc += 2;
    }

    fn copy_to_register(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[y as usize];
        self.pc += 2;
    }
    
    fn i_plus_reg(&mut self, x:u8) {
        self.I = self.I + self.V[x as usize] as u16;
        self.pc += 2;
    }

    fn jump(&mut self, loc: u16) {
        self.pc = loc;
    }

    fn or_with_register(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] | self.V[y as usize];
        self.pc += 2;
    }

    fn random(&mut self, x: u8, byte: u8) {
        self.V[x as usize] = self.rng.gen::<u8>() & byte as u8;
        self.pc += 2;
    }

    fn read_to_regs(&mut self, x: u8) {
        for r in 0..(x as usize + 1) {
            self.V[r] = self.M[self.I as usize + r]
        }
        self.pc += 2;
    }

    fn return_from_sub(&mut self) {
        if self.sp > 0 {
            self.sp -= 1;
            self.pc = self.S[self.sp as usize];
            self.S[self.sp as usize] = 0;
        }
    }

    fn set_dt(&mut self, x: u8) {
        self.dt = self.V[x as usize];
        self.pc += 2;
    }

    fn set_i(&mut self, byte: u16) {
        self.I = byte;
        self.pc += 2;
    }

    fn set_i_digit_sprite(&mut self, x: u8) {
        self.I = self.V[x as usize] as u16 * 5;
        self.pc += 2;
    }

    fn set_register(&mut self, x: u8, byte: u8) {
        self.V[x as usize] = byte;
        self.pc += 2;
    }

    fn set_reg_to_dt(&mut self, x: u8) {
        self.V[x as usize] = self.dt;
        self.pc += 2;
    }

    fn set_st(&mut self, x: u8) {
        self.st = self.V[x as usize];
        self.pc += 2;
    }

    fn shift_left(&mut self, x: u8) {
        let vx = self.V[x as usize];
        self.V[0xf] = vx & 0x8;
        self.V[x as usize] = vx << 1;
        self.pc += 2;
    }

    fn shift_right(&mut self, x: u8) {
        let vx = self.V[x as usize];
        self.V[0xf] = vx & 0x01;
        self.V[x as usize] = vx >> 1;
        self.pc += 2;
    }

    fn skip_if(&mut self, condition: bool) {
        self.pc += if condition { 4 } else { 2 };
    }

    fn skip_if_pressed(&mut self, x: u8) {
        self.skip_if(self.input[self.V[x as usize] as usize]);
    }

    fn skip_if_not_pressed(&mut self, x: u8) {
        self.skip_if(!self.input[self.V[x as usize] as usize]);
    }

    fn skip_if_reg_equals_byte(&mut self, x: u8, byte: u8) {
        self.skip_if(self.V[x as usize] == byte);
    }

    fn skip_if_reg_not_equals_byte(&mut self, x: u8, byte: u8) {
        self.skip_if(self.V[x as usize] != byte);
    }

    fn skip_if_reg_equals_reg(&mut self, x: u8, y: u8) {
        self.skip_if(self.V[x as usize] == self.V[y as usize]);
    }

    fn skip_if_reg_not_equals_reg(&mut self, x: u8, y: u8) {
        self.pc += if self.V[x as usize] != self.V[y as usize] { 4 } else { 2 };
    }

    fn store_regs_through(&mut self, x: u8) {
        for r in 0..(x as usize + 1) {
            self.M[self.I as usize + r] = self.V[r];
        }
        self.pc += 2;
    }

    fn sub(&mut self, x: u8, y: u8) {
        let vx = self.V[x as usize];
        let vy = self.V[y as usize];
        self.V[0xf] = if vx > vy { 1 } else { 0 };
        self.V[x as usize] = vx.wrapping_sub(vy);
        self.pc += 2;
    }

    fn sub_reverse(&mut self, x: u8, y: u8) {
        let vx = self.V[x as usize];
        let vy = self.V[y as usize];
        self.V[0xf] = if vy > vx { 1 } else { 0 };
        self.V[x as usize] = vy.wrapping_sub(vx);
        self.pc += 2;
    }

    fn update_display(&mut self, x: u8, y: u8, n: u8) {
        let vx = self.V[x as usize];
        let vy = self.V[y as usize];
        let mut erased = false;

        for index in 0..n {
            let byte = self.M[(self.I + index as u16) as usize];
            let y_offset = (vy as u16 + index as u16) % 32;
            let row = self.display[y_offset as usize];

            if vx + 8 > 64 {
                let overflow = (vx + 8) - 64;
                let new_row = (byte as u64) << (64 - overflow) | (byte >> overflow) as u64;
                if !erased && (row & new_row > 0) {
                    erased = true;
                }
                self.display[y_offset as usize] = row ^ new_row;
            } else {
                let offset_sprite = (byte as u64) << (56 - vx);
                if !erased && row & offset_sprite > 0 {
                    erased = true;
                }
                self.display[y_offset as usize] = row ^ offset_sprite;
            }
        }

        self.V[0xf] = if erased { 1 } else { 0 };

        self.pc += 2;
    }

    fn wait_for_input(&mut self, x: u8) {
        self.wait = true;
        self.store_input_at = x;
        self.pc += 2;
    }
    
    fn xor_with_register(&mut self, x: u8, y: u8) {
        self.V[x as usize] = self.V[x as usize] ^ self.V[y as usize];
        self.pc += 2;
    }
}

pub fn new_chip8() -> Chip8 {
    Chip8 {
        I: 0,
        M: [0; 4096],
        S: [0; 16],
        V: [0; 16],
        pc: 0x200,
        sp: 0,
        dt: 0,
        st: 0,
        display: [0; 32],
        input: [false; 16],
        wait: false,
        store_input_at: 0,
        rng: rand::thread_rng()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> Chip8 {
        let mut chip = new_chip8();
        chip.init();
        chip
    }

    #[test]
    fn should_clear_screen() {
        let mut chip = init();
        chip.clear_screen();
        assert_eq!(chip.display, [0; 32]);
    }

    #[test]
    fn shouldnt_return_from_subroutine() {
        let mut chip = init();
        let pc = chip.pc;
        chip.return_from_sub();
        assert_eq!(chip.pc, pc);
        assert_eq!(chip.sp, 0);
        assert_eq!(chip.S, [0; 16]);
    }

    #[test]
    fn should_return_from_subroutine() {
        let mut chip = init();
        chip.S[0] = 0x201;
        chip.sp = 1;
        chip.return_from_sub();
        assert_eq!(chip.pc, 0x0201);
        assert_eq!(chip.sp, 0);
        assert_eq!(chip.S, [0; 16]);
    }

    #[test]
    fn should_jump() {
        let mut chip = init();
        chip.jump(0x201);
        assert_eq!(chip.pc, 0x201);
    }

    #[test]
    fn should_call() {
        let mut chip = init();
        chip.call(0x201);
        assert_eq!(chip.pc, 0x201);
        assert_eq!(chip.S[0], 0x202);
        assert_eq!(chip.sp, 1);
    }

    #[test]
    fn should_skip_when_vx_and_byte_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 1;
        chip.skip_if_reg_equals_byte(0, 1);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn shouldnt_skip_when_vx_and_byte_not_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 1;
        chip.skip_if_reg_equals_byte(0, 0);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn should_skip_when_vx_and_byte_not_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 1;
        chip.skip_if_reg_not_equals_byte(0, 0);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn shouldnt_skip_when_vx_and_byte_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 1;
        chip.skip_if_reg_not_equals_byte(0, 1);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn should_skip_when_vx_and_vy_not_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 1;
        chip.skip_if_reg_not_equals_reg(0, 1);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn shouldnt_skip_when_vx_and_vy_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.skip_if_reg_not_equals_reg(0, 1);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn should_skip_when_vx_and_vy_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.skip_if_reg_equals_reg(0, 1);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn shouldnt_skip_when_vx_and_vy_not_equal() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[1] = 1;
        chip.skip_if_reg_equals_reg(0, 1);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn should_set_register() {
        let mut chip = init();
        chip.set_register(0, 1);
        assert_eq!(chip.V[0], 1);
    }

    #[test]
    fn should_add_to_register() {
        let mut chip = init();
        chip.add_to_register(0, 1);
        assert_eq!(chip.V[0], 1);
    }

    #[test]
    fn should_copy_to_register() {
        let mut chip = init();
        chip.V[1] = 1;
        chip.copy_to_register(0, 1);
        assert_eq!(chip.V[0], 1);
    }

    #[test]
    fn should_or_with_register() {
        let mut chip = init();
        chip.V[0] = 0xf0;
        chip.V[1] = 0xf;
        chip.or_with_register(0, 1);
        assert_eq!(chip.V[0], 0xff);
    }

    #[test]
    fn should_and_with_register() {
        let mut chip = init();
        chip.V[0] = 0xf0;
        chip.V[1] = 0xf;
        chip.and_with_register(0, 1);
        assert_eq!(chip.V[0], 0x0);
    }

    #[test]
    fn should_xor_with_register() {
        let mut chip = init();
        chip.V[0] = 0xf8;
        chip.V[1] = 0xf;
        chip.xor_with_register(0, 1);
        assert_eq!(chip.V[0], 0xf7);
    }

    #[test]
    fn should_add_registers() {
        let mut chip = init();
        chip.V[1] = 4;
        chip.add_registers(0, 1);
        assert_eq!(chip.V[0], 4);
    }

    #[test]
    fn should_add_registers_and_carry() {
        let mut chip = init();
        chip.V[0] = 0xff;
        chip.V[1] = 4;
        chip.add_registers(0, 1);
        assert_eq!(chip.V[0], 3);
        assert_eq!(chip.V[0xf], 1);
    }

    #[test]
    fn should_set_i() {
        let mut chip = init();
        chip.set_i(1);
        assert_eq!(chip.I, 1);
    }

    #[test]
    fn should_set_register_to_random() {
        let mut chip = init();
        chip.random(0, 0xff);
    }

    #[test]
    fn should_update_display() {
        let mut chip = init();
        chip.V[0] = 2;
        chip.I = 2048;
        chip.M[2048] = 0x0f;
        chip.M[2049] = 0xf0;
        chip.M[2050] = 0x0f;
        chip.M[2051] = 0xf0;
        chip.update_display(0, 0, 4);
        assert_eq!((chip.display[0]), 0);
        assert_eq!((chip.display[1]), 0);
        assert_eq!((chip.display[2] >> 54), 0x0f);
        assert_eq!((chip.display[3] >> 54), 0xf0);
        assert_eq!((chip.display[4] >> 54), 0x0f);
        assert_eq!((chip.display[5] >> 54), 0xf0);
        assert_eq!(chip.V[0xf], 0);
    }

    #[test]
    fn should_display_digit() {
        let mut chip = init();
        chip.update_display(0, 0, 5);
        assert_eq!((chip.display[0] >> 56), 0xf0);
        assert_eq!((chip.display[1] >> 56), 0x90);
        assert_eq!((chip.display[2] >> 56), 0x90);
        assert_eq!((chip.display[3] >> 56), 0x90);
        assert_eq!((chip.display[4] >> 56), 0xf0);
    }

    #[test]
    fn should_update_display_and_wrap() {
        let mut chip = init();
        chip.V[0] = 60;
        chip.V[1] = 30;
        chip.I = 2048;
        chip.M[2048] = 0x0f;
        chip.M[2049] = 0xf0;
        chip.M[2050] = 0x0f;
        chip.M[2051] = 0xf0;
        chip.update_display(0, 1, 4);
        assert_eq!(chip.display[0], 0xf000000000000000);
        assert_eq!(chip.display[1], 0x000000000000000f);
        assert_eq!(chip.display[30], 0xf000000000000000);
        assert_eq!(chip.display[31], 0x000000000000000f);
        assert_eq!(chip.V[0xf], 0);
    }

    #[test]
    fn should_update_display_and_set_erased() {
        let mut chip = init();
        chip.I = 2048;
        chip.M[2048] = 0x0f;
        chip.M[2049] = 0xf0;
        chip.M[2050] = 0x0f;
        chip.M[2051] = 0xf0;
        chip.display[0] = 0x0f00000000000000;
        chip.update_display(0, 0, 4);
        assert_eq!((chip.display[0] >> 56), 0x00);
        assert_eq!((chip.display[1] >> 56), 0xf0);
        assert_eq!((chip.display[2] >> 56), 0x0f);
        assert_eq!((chip.display[3] >> 56), 0xf0);
        assert_eq!(chip.V[0xf], 1);
    }

    #[test]
    fn should_skip_if_pressed() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 2;
        chip.input[2] = true;
        chip.skip_if_pressed(0);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn shouldnt_skip_if_not_pressed() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 2;
        chip.input[2] = false;
        chip.skip_if_pressed(0);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn shouldnt_skip_if_pressed() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 2;
        chip.input[2] = true;
        chip.skip_if_not_pressed(0);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn should_skip_if_not_pressed() {
        let mut chip = init();
        chip.pc = 0;
        chip.V[0] = 2;
        chip.input[2] = false;
        chip.skip_if_not_pressed(0);
        assert_eq!(chip.pc, 4);
    }

    #[test]
    fn should_wait_for_input() {
        let mut chip = init();
        let mut new_input = [false; 16];
        new_input[6] = true;
        chip.wait_for_input(1);
        assert_eq!(chip.wait, true);
        assert_eq!(chip.store_input_at, 1);
        chip.update_input(new_input);
        assert_eq!(chip.V[1], 6);
        assert_eq!(chip.wait, false);
        assert_eq!(chip.store_input_at, 0);
    }

    #[test]
    fn should_set_dt() {
        let mut chip = init();
        chip.V[0] = 10;
        chip.set_dt(0);
        assert_eq!(chip.dt, 10);
    }

    #[test]
    fn should_set_vx_to_dt() {
        let mut chip = init();
        chip.dt = 10;
        chip.set_reg_to_dt(0);
        assert_eq!(chip.V[0], 10);
    }

    #[test]
    fn should_set_st() {
        let mut chip = init();
        chip.V[0] = 5;
        chip.set_st(0);
        assert_eq!(chip.st, 5);
    }

    #[test]
    fn should_set_i_plus_vx() {
        let mut chip = init();
        chip.I = 5;
        chip.V[0] = 5;
        chip.i_plus_reg(0);
        assert_eq!(chip.I, 10);
    }

    #[test]
    fn should_set_i_to_digit_locations() {
        let mut chip = init();
        chip.V[0] = 2;
        chip.V[1] = 0xa;
        chip.set_i_digit_sprite(0);
        assert_eq!(chip.I, 2 * 5);
        chip.set_i_digit_sprite(1);
        assert_eq!(chip.I, 0xa * 5);
    }

    #[test]
    fn should_store_registers() {
        let mut chip = init();
        chip.V[0] = 1;
        chip.V[1] = 2;
        chip.V[2] = 3;
        chip.I = 1000;
        chip.store_regs_through(2);
        assert_eq!(chip.M[1000], 1);
        assert_eq!(chip.M[1001], 2);
        assert_eq!(chip.M[1002], 3);
    }

    #[test]
    fn should_read_registers() {
        let mut chip = init();
        chip.M[1000] = 1;
        chip.M[1001] = 2;
        chip.M[1002] = 3;
        chip.I = 1000;
        chip.read_to_regs(2);
        assert_eq!(chip.V[0], 1);
        assert_eq!(chip.V[1], 2);
        assert_eq!(chip.V[2], 3);
    }

    #[test]
    fn should_bcd() {
        let mut chip = init();
        chip.V[0] = 245;
        chip.I = 1000;
        chip.bcd(0);
        assert_eq!(chip.M[1000], 2);
        assert_eq!(chip.M[1001], 4);
        assert_eq!(chip.M[1002], 5);
    }
}
