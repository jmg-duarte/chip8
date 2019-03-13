use crate::display::Display;
use crate::keypad::Keypad;
use rand::Rng;

const RAM_SLOTS: usize = 4096;
const N_REGISTERS: usize = 16;
const STACK_LENGTH: usize = 16;

pub struct Cpu {
    pub display: Display,
    pub keypad: Keypad,
    pub memory: [u8; RAM_SLOTS],
    pub v_registers: [u8; N_REGISTERS],
    pub stack: [u16; STACK_LENGTH],
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    // Memory Address
    pub i: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            display: Display::new(),
            keypad: Keypad::new(),
            memory: [0; RAM_SLOTS],
            v_registers: [0; N_REGISTERS],
            stack: [0; STACK_LENGTH],
            i: 0,
            program_counter: 0,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn process_opcode(&mut self, opcode: u16) {
        // break into nibbles
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        match (op_1, op_2, op_3, op_4) {
            // Clear the display
            (0x0, 0x0, 0xE, 0x0) => {
                self.display.cls();
            }
            // Return from subroutine
            (0x0, 0x0, 0xE, 0xE) => {
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.stack_pointer -= 1;
            }
            // Jump to nnn - set the program counter to nnn
            (0x1, _, _, _) => {
                self.program_counter = opcode & 0x0FFF;
            }
            // Call subroutine nnn
            (0x2, _, _, _) => {
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = opcode & 0x0FFF;
            }
            (0x3, _, _, _) => {
                self.program_counter +=
                    if self.v_registers[op_2 as usize] == (opcode & 0x00FF) as u8 {
                        2
                    } else {
                        0
                    };
            }
            (0x4, _, _, _) => {
                self.program_counter +=
                    if self.v_registers[op_2 as usize] != (opcode & 0x00FF) as u8 {
                        2
                    } else {
                        0
                    };
            }
            (0x5, _, _, 0x0) => {
                self.program_counter +=
                    if self.v_registers[op_2 as usize] == self.v_registers[op_3 as usize] {
                        2
                    } else {
                        0
                    };
            }
            (0x6, _, _, _) => {
                self.v_registers[op_2 as usize] = (opcode & 0x00FF) as u8;
            }
            (0x7, _, _, _) => {
                self.v_registers[op_2 as usize] += (opcode & 0x00FF) as u8;
            }
            (0x8, _, _, 0x0) => {
                self.v_registers[op_2 as usize] = self.v_registers[op_3 as usize];
            }
            (0x8, _, _, 0x1) => {
                self.v_registers[op_2 as usize] = (op_2 | op_3) as u8;
            }
            (0x8, _, _, 0x2) => {
                self.v_registers[op_2 as usize] = (op_2 & op_3) as u8;
            }
            (0x8, _, _, 0x3) => {
                self.v_registers[op_2 as usize] = (op_2 ^ op_3) as u8;
            }
            (0x8, _, _, 0x4) => {
                let mut res: u16 = op_2 as u16 + op_3 as u16;
                self.v_registers[0xF] = if res > 255 {
                    res = res & 0xFFFF;
                    1
                } else {
                    0
                };
                self.v_registers[op_2 as usize] = res as u8;
            }
            (0x8, _, _, 0x5) => {
                self.v_registers[0xF] =
                    if self.v_registers[op_2 as usize] < self.v_registers[op_3 as usize] {
                        1
                    } else {
                        0
                    };
                self.v_registers[op_2 as usize] =
                    self.v_registers[op_2 as usize] - self.v_registers[op_3 as usize];
            }
            (0x8, _, _, 0x6) => {
                self.v_registers[0xF] = if (self.v_registers[op_2 as usize] & 0x1) == 0x1 {
                    1
                } else {
                    0
                };
                self.v_registers[op_2 as usize] /= 2;
            }
            (0x8, _, _, 0x7) => {
                self.v_registers[0xF] =
                    if self.v_registers[op_2 as usize] < self.v_registers[op_3 as usize] {
                        1
                    } else {
                        0
                    };
                self.v_registers[op_2 as usize] =
                    self.v_registers[op_3 as usize] - self.v_registers[op_2 as usize];
            }
            (0x8, _, _, 0xE) => {
                self.v_registers[0xF] = if (self.v_registers[op_2 as usize] & 0x1) == 0x1 {
                    1
                } else {
                    0
                };
                self.v_registers[op_2 as usize] *= 2;
            }
            (0x9, _, _, 0x0) => {
                if self.v_registers[op_2 as usize] != self.v_registers[op_3 as usize] {
                    self.program_counter *= 2
                }
            }
            (0xA, _, _, _) => {
                self.i = opcode & 0x0FFF;
            }
            (0xB, _, _, _) => {
                self.program_counter = (opcode & 0x0FFF) + self.v_registers[0] as u16;
            }
            (0xC, _, _, _) => {
                let num = rand::thread_rng().gen_range(0, 256) as u8;
                self.v_registers[op_2 as usize] = ((opcode | 0x00FF) as u8) | num;
            }
            (0xD, _, _, _) => {
                let x = op_2 as u8;
                let y = op_3 as u8;
                let mut pixel: u8;
                for yline in 0..op_4 {
                    pixel = self.memory[(self.i + yline) as usize];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let old_pixel = self.display.get_pixel(x + xline, y + yline as u8);
                            if old_pixel {
                                self.v_registers[0xF] = 1;
                            }
                            self.display
                                .set_pixel(x + xline, y + yline as u8, old_pixel ^ true);
                        }
                    }
                }
                self.program_counter += 2;
            }
            (0xE, _, 0x9, 0xE) => {
                if self
                    .keypad
                    .is_key_down(self.v_registers[op_2 as usize] as usize)
                {
                    self.program_counter += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                if !self
                    .keypad
                    .is_key_down(self.v_registers[op_2 as usize] as usize)
                {
                    self.program_counter += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                self.v_registers[op_2 as usize] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                // ???
            }
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.v_registers[op_2 as usize];
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.v_registers[op_2 as usize];
            }
            (0xF, _, 0x1, 0xE) => {
                self.i += op_2;
            }
            (0xF, _, 0x2, 0x9) => {
                self.i = (self.v_registers[op_2 as usize] * 5) as u16;
            }
            (0xF, _, 0x3, 0x3) => {
                let location = self.i as usize;
                self.memory[location] = self.v_registers[op_2 as usize] / 100;
                self.memory[location + 1] = ((op_2 / 10) % 10) as u8;
                self.memory[location + 2] = ((op_2 % 100) % 10) as u8;
            }
            (0xF, _, 0x5, 0x5) => {
                for i in 0..op_2 as usize {
                    self.memory[self.i as usize + i] = self.v_registers[i];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                for i in 0..op_2 as usize {
                    self.v_registers[i] = self.memory[self.i as usize + i];
                }
            }
            (_, _, _, _) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    #[test]
    fn opcode_jp() {
        let mut cpu = Cpu::new();
        cpu.process_opcode(0x1A2A);
        assert_eq!(
            cpu.program_counter, 0x0A2A,
            "the program counter is updated"
        );
    }

    #[test]
    fn opcode_call() {
        let mut cpu = Cpu::new();
        let addr = 0x23;
        cpu.program_counter = addr;

        cpu.process_opcode(0x2ABC);

        assert_eq!(
            cpu.program_counter, 0x0ABC,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.stack_pointer, 1, "the stack pointer is incremented");
        assert_eq!(
            cpu.stack[0],
            addr + 2,
            "the stack stores the previous address"
        );
    }

    #[test]
    fn opcode_se_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 0xFE;

        // vx == kk
        cpu.process_opcode(0x31FE);
        assert_eq!(cpu.program_counter, 4, "the stack pointer skips");

        // vx != kk
        cpu.process_opcode(0x31FA);
        assert_eq!(cpu.program_counter, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_byte() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 0xFE;

        // vx == kk
        cpu.process_opcode(0x41FE);
        assert_eq!(cpu.program_counter, 2, "the stack pointer is incremented");

        // vx != kk
        cpu.process_opcode(0x41FA);
        assert_eq!(cpu.program_counter, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_se_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 1;
        cpu.v_registers[2] = 3;
        cpu.v_registers[3] = 3;

        // vx == vy
        cpu.process_opcode(0x5230);
        assert_eq!(cpu.program_counter, 4, "the stack pointer skips");

        // vx != vy
        cpu.process_opcode(0x5130);
        assert_eq!(cpu.program_counter, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 1;
        cpu.v_registers[2] = 3;
        cpu.v_registers[3] = 3;

        // vx == vy
        cpu.process_opcode(0x9230);
        assert_eq!(cpu.program_counter, 2, "the stack pointer is incremented");

        // vx != vy
        cpu.process_opcode(0x9130);
        assert_eq!(cpu.program_counter, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_add_vx_kkk() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 3;

        cpu.process_opcode(0x7101);
        assert_eq!(cpu.v_registers[1], 4, "Vx was incremented by one");
    }

    #[test]
    fn opcode_ld_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 3;
        cpu.v_registers[0] = 0;

        cpu.process_opcode(0x8010);
        assert_eq!(cpu.v_registers[0], 3, "Vx was loaded with vy");
    }

    #[test]
    fn opcode_or_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[2] = 0b01101100;
        cpu.v_registers[3] = 0b11001110;

        cpu.process_opcode(0x8231);
        assert_eq!(
            cpu.v_registers[2], 0b11101110,
            "Vx was loaded with vx OR vy"
        );
    }

    #[test]
    fn opcode_and_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[2] = 0b01101100;
        cpu.v_registers[3] = 0b11001110;

        cpu.process_opcode(0x8232);
        assert_eq!(
            cpu.v_registers[2], 0b01001100,
            "Vx was loaded with vx AND vy"
        );
    }

    #[test]
    fn opcode_xor_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[2] = 0b01101100;
        cpu.v_registers[3] = 0b11001110;

        cpu.process_opcode(0x8233);
        assert_eq!(
            cpu.v_registers[2], 0b10100010,
            "Vx was loaded with vx XOR vy"
        );
    }

    #[test]
    fn opcode_add_vx_vy() {
        let mut cpu = Cpu::new();
        cpu.v_registers[1] = 10;
        cpu.v_registers[2] = 100;
        cpu.v_registers[3] = 250;

        cpu.process_opcode(0x8124);
        assert_eq!(cpu.v_registers[1], 110, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v_registers[0xF], 0, "no overflow occured");

        cpu.process_opcode(0x8134);
        assert_eq!(cpu.v_registers[1], 0x68, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v_registers[0xF], 1, "overflow occured");
    }

    #[test]
    fn opcode_ld_i_vx() {
        let mut cpu = Cpu::new();
        cpu.v_registers[0] = 5;
        cpu.v_registers[1] = 4;
        cpu.v_registers[2] = 3;
        cpu.v_registers[3] = 2;
        cpu.i = 0x300;

        // load v0 - v2 into memory at i
        cpu.process_opcode(0xF255);
        assert_eq!(
            cpu.memory[cpu.i as usize], 5,
            "V0 was loaded into memory at i"
        );
        assert_eq!(
            cpu.memory[cpu.i as usize + 1],
            4,
            "V1 was loaded into memory at i + 1"
        );
        assert_eq!(
            cpu.memory[cpu.i as usize + 2],
            3,
            "V2 was loaded into memory at i + 2"
        );
        assert_eq!(cpu.memory[cpu.i as usize + 3], 0, "i + 3 was not loaded");
    }

    #[test]
    fn opcode_ld_b_vx() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.v_registers[2] = 234;

        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF233);
        assert_eq!(cpu.memory[cpu.i as usize], 2, "hundreds");
        assert_eq!(cpu.memory[cpu.i as usize + 1], 3, "tens");
        assert_eq!(cpu.memory[cpu.i as usize + 2], 4, "digits");
    }

    #[test]
    fn opcode_ld_vx_i() {
        let mut cpu = Cpu::new();
        cpu.i = 0x300;
        cpu.memory[cpu.i as usize] = 5;
        cpu.memory[cpu.i as usize + 1] = 4;
        cpu.memory[cpu.i as usize + 2] = 3;
        cpu.memory[cpu.i as usize + 3] = 2;

        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF265);
        assert_eq!(cpu.v_registers[0], 5, "V0 was loaded from memory at i");
        assert_eq!(cpu.v_registers[1], 4, "V1 was loaded from memory at i + 1");
        assert_eq!(cpu.v_registers[2], 3, "V2 was loaded from memory at i + 2");
        assert_eq!(cpu.v_registers[3], 0, "i + 3 was not loaded");
    }

    #[test]
    fn opcode_ret() {
        let mut cpu = Cpu::new();
        let addr = 0x23;
        cpu.program_counter = addr;

        // jump to 0x0ABC
        cpu.process_opcode(0x2ABC);
        // return
        cpu.process_opcode(0x00EE);

        assert_eq!(
            cpu.program_counter, 0x25,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.stack_pointer, 0, "the stack pointer is decremented");
    }

    #[test]
    fn opcode_ld_i_addr() {
        let mut cpu = Cpu::new();

        cpu.process_opcode(0x61AA);
        assert_eq!(cpu.v_registers[1], 0xAA, "V1 is set");
        assert_eq!(
            cpu.program_counter, 2,
            "the program counter is advanced two bytes"
        );

        cpu.process_opcode(0x621A);
        assert_eq!(cpu.v_registers[2], 0x1A, "V2 is set");
        assert_eq!(
            cpu.program_counter, 4,
            "the program counter is advanced two bytes"
        );

        cpu.process_opcode(0x6A15);
        assert_eq!(cpu.v_registers[10], 0x15, "V10 is set");
        assert_eq!(
            cpu.program_counter, 6,
            "the program counter is advanced two bytes"
        );
    }

    #[test]
    fn opcode_axxx() {
        let mut cpu = Cpu::new();
        cpu.process_opcode(0xAFAF);

        assert_eq!(cpu.i, 0x0FAF, "the 'i' register is updated");
        assert_eq!(
            cpu.program_counter, 2,
            "the program counter is advanced two bytes"
        );
    }

}
