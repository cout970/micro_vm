use std::intrinsics::transmute;
use std::ops::Add;

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
    Then,
    Otherwise,
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
    Debug,
}

const SP_REGISTER: usize = 15;
const AT_REGISTER: usize = 14;
const INSTRUCTION_LEN: [u16; 26] = [
    1, // Nop
    1, // Exit
    2, // JumpFw
    2, // JumpBw
    1, // Then
    1, // Otherwise
    3, // SetByte
    4, // SetShort
    2, // Push
    2, // Pop
    3, // Add
    3, // Sub
    3, // Mul
    3, // Div
    3, // Mod
    2, // Neg
    3, // GreaterThan
    3, // LessThan
    3, // GreaterEqual
    3, // LessEqual
    3, // Equal
    3, // NotEqual
    1, // Return
    3, // Call
    3, // Mov
    3  // Debug
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
            let inst_parsed = unsafe { transmute::<u8, Inst>(inst) };
//            println!("{:?}", inst_parsed); // DEBUG
            match inst_parsed {
                Inst::Nop => {}
                Inst::Exit => { return; }
                Inst::JumpFw => {
                    self.pc += self.ram[self.pc as usize] as u16 + 1;
                }
                Inst::JumpBw => {
                    self.pc = ((self.pc as i16) + (self.ram[self.pc as usize] as i8) as i16 - 1i16) as u16;
                }
                Inst::Then => {
                    if !self.skip_flag {
                        self.pc += INSTRUCTION_LEN[self.ram[self.pc as usize] as usize];
                    }
                }
                Inst::Otherwise => {
                    if self.skip_flag {
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
                    let low_bytes = self.ram[(self.pc + 2) as usize] as u16;
                    let high_bytes = self.ram[(self.pc + 1) as usize] as u16;

                    if reg != 0 {
                        self.registers[reg as usize] = (high_bytes << 8) | low_bytes;
                    }
                    self.pc += 3;
                }
                Inst::Push => {
                    let sp = self.registers[SP_REGISTER];
                    self.registers[SP_REGISTER] -= 2;

                    let reg = self.ram[self.pc as usize];
                    self.ram[(sp + 1) as usize] = self.registers[reg as usize] as u8;
                    self.ram[sp as usize] = (self.registers[reg as usize] >> 8) as u8;
                    self.pc += 1;
                }
                Inst::Pop => {
                    self.registers[SP_REGISTER] += 2;
                    let sp = self.registers[SP_REGISTER];

                    let reg = self.ram[self.pc as usize];
                    if reg != 0 {
                        self.registers[reg as usize] = self.ram[(sp + 1) as usize] as u16;
                        self.registers[reg as usize] |= (self.ram[sp as usize] as u16) << 8;
                    }
                    self.pc += 1;
                }
                Inst::Add => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if a != 0 {
                        let x = self.registers[a as usize] as i16;
                        let y = self.registers[b as usize] as i16;
                        self.registers[a as usize] = x.wrapping_add(y) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Sub => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if a != 0 {
                        let x = self.registers[a as usize] as i16;
                        let y = self.registers[b as usize] as i16;
                        self.registers[a as usize] = x.wrapping_sub(y) as u16;
                        ;
                    }
                    self.pc += 2;
                }
                Inst::Mul => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if a != 0 {
                        self.registers[a as usize] = ((self.registers[a as usize] as i16) * (self.registers[b as usize] as i16)) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Div => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if self.registers[b as usize] != 0 && a != 0 {
                        self.registers[a as usize] = ((self.registers[a as usize] as i16) / (self.registers[b as usize] as i16)) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Mod => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if self.registers[b as usize] != 0 && a != 0 {
                        self.registers[a as usize] = ((self.registers[a as usize] as i16) % (self.registers[b as usize] as i16)) as u16;
                    }
                    self.pc += 2;
                }
                Inst::Neg => {
                    let reg = self.ram[self.pc as usize];
                    if reg != 0 {
                        self.registers[reg as usize] = (-(self.registers[reg as usize] as i16)) as u16;
                    }
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
                Inst::Return => {
                    self.registers[SP_REGISTER] += 2;
                    let sp = self.registers[SP_REGISTER];

                    self.registers[AT_REGISTER] = self.ram[(sp + 1) as usize] as u16;
                    self.registers[AT_REGISTER] |= (self.ram[sp as usize] as u16) << 8;

                    self.pc = self.registers[AT_REGISTER];
                }
                Inst::Call => {
                    let low_bytes = self.ram[(self.pc + 1) as usize] as u16;
                    let high_bytes = self.ram[(self.pc) as usize] as u16;
                    self.pc += 2;

                    // Store return addr
                    let sp = self.registers[SP_REGISTER];
                    self.registers[SP_REGISTER] -= 2;

                    self.ram[(sp + 1) as usize] = self.pc as u8;
                    self.ram[sp as usize] = (self.pc >> 8) as u8;

                    // Jump to subroutine
                    self.pc = (high_bytes << 8) | low_bytes;
                }
                Inst::Mov => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    if a != 0 {
                        self.registers[a as usize] = self.registers[b as usize];
                    }
                    self.pc += 2;
                }
                Inst::Debug => {
                    let a = self.ram[self.pc as usize];
                    let b = self.ram[(self.pc + 1) as usize];

                    match b {
                        0 => println!("{}", self.registers[a as usize]),
                        1 => println!("{:x}", self.registers[a as usize]),
                        2 => println!("{:05}", self.registers[a as usize]),
                        3 => println!("{:04X}", self.registers[a as usize]),
                        4 => println!("0x{:04X}", self.registers[a as usize]),
                        10 => print!("{}", self.registers[a as usize]),
                        11 => print!("{:x}", self.registers[a as usize]),
                        12 => print!("{:05}", self.registers[a as usize]),
                        13 => print!("{:04X}", self.registers[a as usize]),
                        14 => print!("0x{:04X}", self.registers[a as usize]),
                        _ => self.print(),
                    }

                    self.pc += 2;
                }
                _ => {
                    panic!("Invalid instruction: {}", inst);
                }
            }
        }
    }

    pub fn print(&self) {
        print!("{{");
        print!("\n  pc: {}", self.pc);
        print!("\n  ram: [\n");
        self.disassembly();
        print!("  ]");
        print!("\n  registers: [");
        for i in 0..16 {
            print!("\n    {:4} (0x{:04X})", self.registers[i], self.registers[i]);
        }
        print!("\n  ]");
        print!("\n}}");
    }

    pub fn disassembly(&self) {
        let mut pos = 0;

        while pos < self.ram.len() {
            let byte = self.ram[pos];
            let inst = unsafe { transmute::<u8, Inst>(byte) };
            let len = INSTRUCTION_LEN[byte as usize];

            print!("{:04X}  {:?}", pos, inst);
            for i in 1..(len as usize) {
                if pos + i >= self.ram.len() {
                    continue;
                }
                print!(" {:02X}", self.ram[pos + i]);
            }
            println!();

            pos += len as usize;
        }
    }
}