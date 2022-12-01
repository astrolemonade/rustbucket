#![allow(dead_code)]

#[derive(Debug)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                0x00 => return,
                0xA1 => {
                    self.lda(&AddressingMode::IndirectX);
                    self.program_counter;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xAA => self.tax(),
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::IndirectY);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::ZeroPageX);
                    self.program_counter += 1;
                }
                0xB9 => {
                    self.lda(&AddressingMode::AbsoluteY);
                    self.program_counter += 1;
                }
                0xBD => {
                    self.lda(&AddressingMode::AbsoluteX);
                    self.program_counter += 1;
                }
                0xE8 => self.inx(),
                _ => {}
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => panic!("Mode {:?} is not supported", mode),
        }
    }
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lda_immediate() {
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0x05, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.register_a, 0x05);
    }

    #[test]
    fn lda_zero_page() {
        let mut cpu = CPU::new();
        let program = vec![0xA5, 0x80, 0x00];
        cpu.mem_write(0x80, 0xFF);
        cpu.load_and_run(program);

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_zero_page_x() {
        let mut cpu = CPU::new();
        let program = vec![0xB5, 0x80, 0x00];
        cpu.mem_write(0x8F, 0xFF);
        cpu.load(program);
        cpu.reset();
        cpu.register_x = 0x0F;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_absolute() {
        let mut cpu = CPU::new();
        let program = vec![0xAD, 0xAD, 0xDE, 0x00];
        cpu.mem_write(0xDEAD, 0xFF);
        cpu.load_and_run(program);

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_absolute_x() {
        let mut cpu = CPU::new();
        let program = vec![0xBD, 0x00, 0xDE, 0x00];
        cpu.mem_write(0xDEAD, 0xFF);
        cpu.load(program);
        cpu.reset();
        cpu.register_x = 0xAD;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_absolute_y() {
        let mut cpu = CPU::new();
        let program = vec![0xB9, 0x00, 0xDE, 0x00];
        cpu.mem_write(0xDEAD, 0xFF);
        cpu.load(program);
        cpu.reset();
        cpu.register_y = 0xAD;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_indirect_x() {
        let mut cpu = CPU::new();
        let program = vec![0xA1, 0x02, 0x00];
        cpu.mem_write_u16(0x04, 0x8086);
        cpu.mem_write(0x8086, 0xFF);
        cpu.load(program);
        cpu.reset();
        cpu.register_x = 0x02;
        cpu.run();
        //println!("{:?}", cpu.memory);

        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn lda_indirect_y() {
        let mut cpu = CPU::new();
        let program = vec![0xB1, 0x02, 0x00];
        cpu.mem_write(0x8086, 0xFF);
        cpu.mem_write_u16(0x02, 0x8000);
        cpu.load(program);
        cpu.reset();
        cpu.register_y = 0x86;
        cpu.run();

        assert_eq!(cpu.register_a, 0xFF);
    }
}
