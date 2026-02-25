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
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 2048],
        }
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
    cpu.memory[0x200] = 0x1A;
    cpu.memory[0x201] = 0x2B;
    let opcode = cpu.fetch();
    cpu.execute(opcode);
    println!("PC = {:04X}", cpu.pc);
}
