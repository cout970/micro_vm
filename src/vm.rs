use std::intrinsics::transmute;

pub struct VM {
    registers: [u16; 16],
    ram: [u8; 1024],
    pc: u16,
    skip_flag: bool,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Inst {
    Nop,
    Exit,
    JumpFw,
    JumpBw,
    If,
    Unless,
    SetByte,
    SetShort,
    Push,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
    Return,
    Call,
    Mov,
}

const SP_REGISTER: usize = 15;
const INSTRUCTION_LEN: [u16; 25] = [
    1, // Nop
    1, // Exit
    2, // JumpFw
    2, // JumpBw
    1, // If
    1, // Unless
    3, // SetByte
    4, // SetShort
    2, // Push
    2, // Pop
    2, // Add
    2, // Sub
    2, // Mul
    2, // Div
    2, // Mod
    1, // Neg
    2, // GreaterThan
    2, // LessThan
    2, // GreaterEqual
    2, // LessEqual
    2, // Equal
    2, // NotEqual
    1, // Return
    2, // Call
    3  // Mov
];

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 16],
            ram: [0; 1024],
            pc: 0,
            skip_flag: false,
        }
    }

    pub fn reset(&mut self) {
        self.registers[SP_REGISTER] = self.ram.len() as u16 - 2;
        self.pc = 0;
    }

    pub fn set(&mut self, i: u8) {
        self.ram[self.pc as usize] = i;
        self.pc += 1;
    }

    pub fn run(&mut self) {
        loop {
            let inst = self.ram[self.pc as usize];
            self.pc += 1;
            match unsafe { transmute::<u8, Inst>(inst) } {
                Inst::Nop => {}
                Inst::Exit => { return; }
                Inst::JumpFw => {
                    self.pc += self.ram[self.pc as usize] as u16 + 1;
                }
                Inst::JumpBw => {
                    self.pc -= self.ram[self.pc as usize] as u16 + 1;
                }
                Inst::If => {
                    if self.skip_flag {
                        self.pc += INSTRUCTION_LEN[self.ram[self.pc as usize] as usize];
                    }
                }
                Inst::Unless => {
                    if !self.skip_flag {
                        self.pc += INSTRUCTION_LEN[self.ram[self.pc as usize] as usize];
                    }
                }
                Inst::SetByte => {
                    let reg = self.ram[self.pc as usize];
                    let byte = self.ram[(self.pc + 1) as usize] as u16;

                    if reg != 0 {
                        self.registers[reg as usize] = byte;
                    }
                    self.pc += 2;
                }
                Inst::SetShort => {
                    let reg = self.ram[self.pc as usize];
                    let low_bytes = self.ram[(self.pc + 1) as usize] as u16;
                    let high_bytes = self.ram[(self.pc + 2) as usize] as u16;

                    if reg != 0 {
                        self.registers[reg as usize] = (high_bytes << 8) | low_bytes;
                    }
                    self.pc += 3;
                }
                Inst::Push => {
                    let sp = self.registers[SP_REGISTER];
                    self.registers[SP_REGISTER] -= 2;

                    let reg = self.ram[self.pc as usize];
                    self.ram[sp as usize] = self.registers[reg as usize] as u8;
                    self.ram[(sp + 1) as usize] = (self.registers[reg as usize] >> 8) as u8;
                    self.pc += 1;
                }
                Inst::Pop => {
                    self.registers[SP_REGISTER] += 2;
                    let sp = self.registers[SP_REGISTER];

                    let reg = self.ram[self.pc as usize];
                    if reg != 0 {
                        self.registers[reg as usize] = self.ram[sp as usize] as u16;
                        self.registers[reg as usize] |= (self.ram[(sp + 1) as usize] as u16) << 8;
                    }
                    self.pc += 1;
                }
                Inst::Add => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.registers[a as usize] = ((self.registers[a as usize] as i16) + (self.registers[b as usize] as i16)) as u16;
                    self.pc += 2;
                }
                Inst::Sub => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.registers[a as usize] = ((self.registers[a as usize] as i16) - (self.registers[b as usize] as i16)) as u16;
                    self.pc += 2;
                }
                Inst::Mul => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.registers[a as usize] = ((self.registers[a as usize] as i16) * (self.registers[b as usize] as i16)) as u16;
                    self.pc += 2;
                }
                Inst::Div => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if self.registers[b as usize] != 0 {
                        self.registers[a as usize] = ((self.registers[a as usize] as i16) / (self.registers[b as usize] as i16)) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Mod => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if self.registers[b as usize] != 0 {
                        self.registers[a as usize] = ((self.registers[a as usize] as i16) % (self.registers[b as usize] as i16)) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Neg => {
                    let reg = self.ram[self.pc as usize];
                    self.registers[reg as usize] = (-(self.registers[reg as usize] as i16)) as u16;
                    self.pc += 1;
                }
                Inst::GreaterThan => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = (self.registers[a as usize] as i16) > (self.registers[b as usize] as i16);
                    self.pc += 2;
                }
                Inst::LessThan => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = (self.registers[a as usize] as i16) < (self.registers[b as usize] as i16);
                    self.pc += 2;
                }
                Inst::GreaterEqual => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = (self.registers[a as usize] as i16) >= (self.registers[b as usize] as i16);
                    self.pc += 2;
                }
                Inst::LessEqual => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = (self.registers[a as usize] as i16) <= (self.registers[b as usize] as i16);
                    self.pc += 2;
                }
                Inst::Equal => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = self.registers[a as usize] == self.registers[b as usize];
                    self.pc += 2;
                }
                Inst::NotEqual => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    self.skip_flag = self.registers[a as usize] != self.registers[b as usize];
                    self.pc += 2;
                }
                Inst::Return => {}
                Inst::Call => {}
                Inst::Mov => {}
                _ => {
                    panic!("Invalid instruction: {}", inst);
                }
            }
        }
    }

    pub fn print(&self) {
        print!("{{");
        print!("\n  pc: {}", self.pc);
        print!("\n  ram: [");
        for i in 0..16 {
            print!("\n    {:3} (0x{:02X})", self.ram[i], self.ram[i]);
        }
        print!("\n  ]");
        print!("\n  registers: [");
        for i in 0..16 {
            print!("\n    {:4} (0x{:04X})", self.registers[i], self.registers[i]);
        }
        print!("\n  ]");
        print!("\n}}");
    }
}