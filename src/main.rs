use std::thread;
use std::time::{Duration, Instant};

use minifb::{Scale, Window, WindowOptions};

#[allow(unused)]
struct CPU {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    display: [bool; 2048],
    keys: [bool; 16],
}

const FONT_SET: [u8; 80] = [
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

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 2048],
            keys: [false; 16],
        };

        cpu.memory[0..80].copy_from_slice(&FONT_SET);
        cpu
    }
    pub fn fetch(&mut self) -> u16 {
        let mut byte1 = self.memory[self.pc as usize] as u16;
        byte1 <<= 8;
        let byte2 = self.memory[(self.pc + 1) as usize] as u16;
        let result = byte1 | byte2;
        self.pc += 2;
        result
    }
    pub fn execute(&mut self, opcode: u16) {
        let n1 = (opcode & 0xF000) >> 12;
        let n2 = (opcode & 0x0F00) >> 8;
        let n3 = (opcode & 0x00F0) >> 4;
        let n4 = opcode & 0x000F;

        let x = n2 as usize;
        let y = n3 as usize;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match (n1, n2, n3, n4) {
            (1, _, _, _) => {
                self.pc = nnn;
            }
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = nnn;
            }
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            (6, _, _, _) => {
                self.v[x] = nn;
            }
            (7, _, _, _) => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            (0xA, _, _, _) => {
                self.i = nnn;
            }
            (3, _, _, _) => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            (8, _, _, _) => match n4 {
                0 => {
                    self.v[x] = self.v[y];
                }
                1 => {
                    self.v[x] = self.v[x] | self.v[y];
                }
                2 => {
                    self.v[x] = self.v[x] & self.v[y];
                }
                3 => {
                    self.v[x] = self.v[x] ^ self.v[y];
                }
                4 => {
                    let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if overflow { 1 } else { 0 };
                }
                5 => {
                    let (result, underflow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if underflow { 0 } else { 1 };
                }
                6 => {
                    let lsb = self.v[x] & 1;
                    self.v[x] >>= 1;
                    self.v[0xF] = lsb;
                }
                7 => {
                    let (result, underflow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xF] = if underflow { 0 } else { 1 };
                }
                0xE => {
                    let msb = self.v[x] >> 7;
                    self.v[x] <<= 1;
                    self.v[0xF] = msb;
                }
                _ => unimplemented!("Opcode {:04X} not implemented yet", opcode),
            },
            (9, _, _, 0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            (0, 0, 0xE, 0) => {
                self.display = [false; 2048];
            }
            (0xC, _, _, _) => {
                let num: u8 = rand::random();
                self.v[x] = num & nn;
            }
            (0xD, _, _, _) => {
                let x_coord = (self.v[x] as usize) % 64;
                let y_coord = (self.v[y] as usize) % 32;

                let height = n4 as usize;

                self.v[0xF] = 0;

                for row in 0..height {
                    let sprite_byte = self.memory[(self.i as usize) + row];
                    for col in 0..8 {
                        let sprite_pixel = sprite_byte & (0x80 >> col);
                        if sprite_pixel != 0 {
                            let target_x = x_coord + col;
                            let target_y = y_coord + row;

                            if target_x < 64 && target_y < 32 {
                                let index = target_x + (target_y * 64);

                                if self.display[index] == true {
                                    self.v[0xF] = 1;
                                }
                                self.display[index] ^= true;
                            }
                        }
                    }
                }
            }
            (0xE, _, 9, 0xE) => {
                let idx = self.v[x] as usize;
                if self.keys[idx] == true {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                let idx = self.v[x] as usize;
                if self.keys[idx] == false {
                    self.pc += 2;
                }
            }
            (0xF, _, _, _) => match nn {
                0x07 => {
                    self.v[x] = self.delay_timer;
                }
                0x15 => {
                    self.delay_timer = self.v[x];
                }
                0x18 => {
                    self.sound_timer = self.v[x];
                }
                0x0A => {
                    let mut key_pressed = false;
                    for (i, pressed) in self.keys.iter().enumerate() {
                        if *pressed {
                            self.v[x] = i as u8;
                            key_pressed = true;
                            break;
                        }
                    }
                    if !key_pressed {
                        self.pc -= 2;
                    }
                }
                0x29 => {
                    self.i = (self.v[x] as u16) * 5;
                }
                0x1E => {
                    self.i += self.v[x] as u16;
                }
                0x55 => {
                    for idx in 0..=x {
                        self.memory[self.i as usize + idx] = self.v[idx];
                    }
                }
                0x65 => {
                    for idx in 0..=x {
                        self.v[idx] = self.memory[(self.i as usize) + idx];
                    }
                }
                0x33 => {
                    let i = self.i as usize;
                    let value = self.v[x];
                    self.memory[i] = value / 100;
                    self.memory[i + 1] = (value % 100) / 10;
                    self.memory[i + 2] = value % 10;
                }
                _ => unimplemented!("FX opcode {:04X} not implemented", opcode),
            },
            _ => unimplemented!("Opcode {:04X} not implemented yet", opcode),
        }
    }
    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}

fn main() {
    let mut cpu = CPU::new();

    cpu.memory[0x200] = 0x12;
    cpu.memory[0x201] = 0x00;

    cpu.display[0] = true;
    cpu.display[1] = true;
    cpu.display[2] = true;

    let mut buffer: Vec<u32> = vec![0; 64 * 32];
    let mut window = Window::new(
        "My CHIP-8 Emulator",
        64,
        32,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    let mut last_timer_update = Instant::now();

    loop {
        let opcode = cpu.fetch();
        cpu.execute(opcode);

        if last_timer_update.elapsed() >= Duration::from_millis(16) {
            cpu.delay_timer = cpu.delay_timer.saturating_sub(1);
            cpu.sound_timer = cpu.sound_timer.saturating_mul(1);
            last_timer_update = Instant::now();
        }

        for (i, &pixel) in cpu.display.iter().enumerate() {
            buffer[i] = if pixel { 0xFFFFFF } else { 0x000000 }
        }
        window.update_with_buffer(&buffer, 64, 32).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
}
