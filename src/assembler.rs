use std::borrow::Cow;
use std::collections::HashMap;

use crate::assembler::TokenType::*;
use crate::vm::Inst;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Token(TokenType, (usize, usize));

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenType {
    Id,
    Int,
    Comma,
    NewLine,
    Colon,
    Eof,
}

pub fn read_all_tokens(source: &[u8]) -> Vec<Token> {
    let mut res = Vec::new();
    let mut ptr = 0usize;

    loop {
        if ptr >= source.len() {
            res.push(Token(Eof, (ptr, ptr)));
            break;
        }

        match source[ptr] {
            b';' => {
                while ptr < source.len() && source[ptr] != b'\n' {
                    ptr += 1;
                }
            }
            b' ' => {
                ptr += 1;
            }
            b',' => {
                res.push(Token(Comma, (ptr, ptr + 1)));
                ptr += 1;
            }
            b'\n' => {
                res.push(Token(NewLine, (ptr, ptr + 1)));
                ptr += 1;
            }
            b':' => {
                res.push(Token(Colon, (ptr, ptr + 1)));
                ptr += 1;
            }
            b'0'..=b'9' => {
                let start = ptr;
                let mut hex = false;

                if source[ptr] == b'0'
                    && ptr + 2 < source.len()
                    && source[ptr + 1] == b'x'
                    && source[ptr + 2].is_ascii_digit() {
                    hex = true;
                    ptr += 2;
                }

                loop {
                    if ptr >= source.len() { break; }
                    if hex {
                        if !source[ptr].is_ascii_hexdigit() { break; }
                    } else {
                        if !source[ptr].is_ascii_digit() { break; }
                    }
                    ptr += 1;
                }
                res.push(Token(Int, (start, ptr)));
            }
            b'a'..=b'z' | b'_' => {
                let start = ptr;
                while let b'a'..=b'z' | b'_' = source[ptr] {
                    ptr += 1;
                    if ptr >= source.len() { break; }
                }
                res.push(Token(Id, (start, ptr)));
            }
            _ => {
                // Ignored
                ptr += 1;
            }
        }
    }

    res
}

pub fn print_tokens(source: &[u8], tokens: &Vec<Token>) {
    for Token(ty, span) in tokens {
        match ty {
            Id => print!(" Id({})", String::from_utf8_lossy(&source[span.0..span.1])),
            Int => print!(" Int({})", String::from_utf8_lossy(&source[span.0..span.1])),
            Comma => print!(","),
            NewLine => print!("\n"),
            Colon => print!(": "),
            Eof => print!(" EOF\n"),
        }
    }
}

pub struct Parser<'a> {
    source: &'a [u8],
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Clone, Debug)]
pub enum ParsedInst {
    Label { label: String },
    Nop,
    Exit,
    Jump { label: String },
    Then,
    Otherwise,
    SetByte { dst: u32, val: u8 },
    SetShort { dst: u32, val: u16 },
    Push { src: u32 },
    Pop { dst: u32 },
    Add { dst: u32, src: u32 },
    Sub { dst: u32, src: u32 },
    Mul { dst: u32, src: u32 },
    Div { dst: u32, src: u32 },
    Mod { dst: u32, src: u32 },
    Neg { dst: u32 },
    GreaterThan { left: u32, right: u32 },
    LessThan { left: u32, right: u32 },
    GreaterEqual { left: u32, right: u32 },
    LessEqual { left: u32, right: u32 },
    Equal { left: u32, right: u32 },
    NotEqual { left: u32, right: u32 },
    Return,
    Call { label: String },
    Mov { dst: u32, src: u32 },
    Debug { src: u32, mode: u32 },
}

impl<'a> Parser<'a> {
    pub fn new(source: &[u8], tokens: Vec<Token>) -> Parser {
        Parser {
            source,
            tokens,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ParsedInst>, String> {
        let mut inst = Vec::new();

        loop {
            let Token(token_type, span) = self.tk(self.pos);
            match token_type {
                Id => self.parse_instruction(&mut inst)?,
                Int => return Err(format!("Unexpected token Int at {:?}", span)),
                Comma => return Err(format!("Unexpected token Comma at {:?}", span)),
                NewLine => { self.pos += 1; }
                Colon => return Err(format!("Unexpected token Colon at {:?}", span)),
                Eof => break,
            }
        }

        Ok(inst)
    }

    fn parse_instruction(&mut self, inst: &mut Vec<ParsedInst>) -> Result<(), String> {
        let name = self.consume_id()?;

        match name.as_ref() {
            "nop" => inst.push(ParsedInst::Nop),
            "exit" => inst.push(ParsedInst::Exit),
            "jmp" => {
                let arg1 = self.consume_id()?;
                inst.push(ParsedInst::Jump { label: arg1 });
            }
            "then" => inst.push(ParsedInst::Then),
            "else" => inst.push(ParsedInst::Otherwise),
            "set" => {
                let arg1 = self.parse_reg()?;
                self.consume(Comma)?;
                let arg2 = self.consume_int()? as u32;

                if arg2 > 255 {
                    inst.push(ParsedInst::SetShort { dst: arg1, val: arg2 as u16 });
                } else {
                    inst.push(ParsedInst::SetByte { dst: arg1, val: arg2 as u8 });
                }
            }
            "push" => {
                let arg1 = self.parse_reg()?;
                inst.push(ParsedInst::Push { src: arg1 })
            }
            "pop" => {
                let arg1 = self.parse_reg()?;
                inst.push(ParsedInst::Pop { dst: arg1 })
            }
            "add" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Add { dst: arg1, src: arg2 })
            }
            "sub" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Sub { dst: arg1, src: arg2 })
            }
            "mul" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Mul { dst: arg1, src: arg2 })
            }
            "div" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Div { dst: arg1, src: arg2 })
            }
            "mod" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Mod { dst: arg1, src: arg2 })
            }
            "neg" => {
                let arg1 = self.parse_reg()?;
                inst.push(ParsedInst::Neg { dst: arg1 })
            }
            "gt" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::GreaterThan { left: arg1, right: arg2 })
            }
            "lt" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::LessThan { left: arg1, right: arg2 })
            }
            "ge" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::GreaterEqual { left: arg1, right: arg2 })
            }
            "le" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::LessEqual { left: arg1, right: arg2 })
            }
            "eq" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Equal { left: arg1, right: arg2 })
            }
            "neq" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::NotEqual { left: arg1, right: arg2 })
            }
            "ret" => {
                inst.push(ParsedInst::Return)
            }
            "call" => {
                let label = self.consume_id()?;
                inst.push(ParsedInst::Call { label })
            }
            "mov" => {
                let (arg1, arg2) = self.parse_2reg()?;
                inst.push(ParsedInst::Mov { dst: arg1, src: arg2 })
            }
            "dbg" => {
                let arg1 = self.parse_reg()?;
                self.consume(Comma)?;
                let mode = self.consume_int()? as u32;
                inst.push(ParsedInst::Debug { src: arg1, mode })
            }
            _ => {
                if self.expect(Colon).is_ok() {
                    self.consume(Colon)?;
                    inst.push(ParsedInst::Label { label: name });
                } else {
                    return Err(format!("Found unexpected symbol {:?}", name));
                }
            }
        }
        if self.tk(self.pos).0 != Eof {
            self.consume(NewLine)?;
        }
        Ok(())
    }

    fn parse_reg(&mut self) -> Result<u32, String> {
        let text = self.consume_id()?;
        match text.as_ref() {
            "zero" | "z" => Ok(0),
            "$0" | "a" => Ok(1),
            "$1" | "b" => Ok(2),
            "$2" | "c" => Ok(3),
            "$3" | "d" => Ok(4),
            "$4" | "e" => Ok(5),
            "$5" | "f" => Ok(6),
            "$6" | "g" => Ok(7),
            "$7" | "h" => Ok(8),
            "$8" | "i" => Ok(9),
            "$9" | "j" => Ok(10),
            "$10" | "k" => Ok(11),
            "$11" | "l" => Ok(12),
            "$12" | "m" => Ok(13),
            "at" => Ok(14),
            "sp" => Ok(15),
            _ => Err(format!("Expected register name, found {:?}", text))
        }
    }

    fn parse_2reg(&mut self) -> Result<(u32, u32), String> {
        let arg1 = self.parse_reg()?;
        self.consume(Comma)?;
        let arg2 = self.parse_reg()?;

        Ok((arg1, arg2))
    }

    fn consume_int(&mut self) -> Result<i32, String> {
        let name = self.expect_int()?;
        self.pos += 1;
        Ok(name)
    }

    fn expect_int(&self) -> Result<i32, String> {
        let Token(token_type, span) = self.tk(self.pos);
        if token_type != &Int {
            Err(format!("Expected Int but found {:?}", token_type))
        } else {
            let digits = self.str(*span);
            digits.parse::<i32>().map_err(|err| format!("Unable to parse integer: {:?}", err))
        }
    }

    fn consume_id(&mut self) -> Result<String, String> {
        let name = self.expect_id()?;
        self.pos += 1;
        Ok(name)
    }

    fn expect_id(&self) -> Result<String, String> {
        let Token(token_type, span) = self.tk(self.pos);
        if token_type != &Id {
            Err(format!("Expected Id but found {:?}", token_type))
        } else {
            Ok(self.str(*span).to_string())
        }
    }

    fn consume(&mut self, ty: TokenType) -> Result<(), String> {
        self.expect(ty)?;
        self.pos += 1;
        Ok(())
    }

    fn expect(&self, ty: TokenType) -> Result<(), String> {
        let Token(token_type, span) = self.tk(self.pos);
        if token_type != &ty {
            Err(format!("Expected {:?} but found {:?} at {:?}", ty, token_type, span))
        } else {
            Ok(())
        }
    }

    fn tk(&self, pos: usize) -> &Token {
        let pos = pos.min(self.tokens.len());
        &self.tokens[pos]
    }

    fn str(&self, span: (usize, usize)) -> Cow<str> {
        String::from_utf8_lossy(&self.source[span.0..span.1])
    }
}

pub struct Compiler {
    symbol_table: HashMap<String, usize>,
    buffer: Vec<PrecompiledInst>,
    pos: usize,
}

#[derive(Clone, Debug)]
pub enum PrecompiledInst {
    JumpPlaceHolder(String, usize),
    CallPlaceHolder(String),
    Compiled1(Inst),
    Compiled2(Inst, u8),
    Compiled3(Inst, u8, u8),
    Compiled4(Inst, u8, u8, u8),
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            symbol_table: HashMap::new(),
            buffer: Vec::new(),
            pos: 0,
        }
    }

    pub fn compile(&mut self, insts: &Vec<ParsedInst>) -> Result<Vec<u8>, String> {
        let mut asm = Vec::new();
        self.precompile(insts);

        for inst in &self.buffer {
            match inst {
                PrecompiledInst::JumpPlaceHolder(label, pos) => {
                    let target = *self.symbol_table.get(label)
                        .ok_or_else(|| format!("Jump to invalid label: {:?}", label))?;
                    let diff = (target as isize) - (*pos as isize);

                    if diff > 0 {
                        asm.push(Inst::JumpFw as u8);
                    } else {
                        asm.push(Inst::JumpBw as u8);
                    }
                    asm.push(diff as u8);
                }
                PrecompiledInst::CallPlaceHolder(label) => {
                    let target = *self.symbol_table.get(label)
                        .ok_or_else(|| format!("Call to invalid label: {:?}", label))?;

                    asm.push(Inst::Call as u8);
                    asm.push((target >> 8) as u8);
                    asm.push(target as u8);
                }
                PrecompiledInst::Compiled1(i) => {
                    asm.push(*i as u8);
                }
                PrecompiledInst::Compiled2(i, a) => {
                    asm.push(*i as u8);
                    asm.push(*a as u8);
                }
                PrecompiledInst::Compiled3(i, a, b) => {
                    asm.push(*i as u8);
                    asm.push(*a as u8);
                    asm.push(*b as u8);
                }
                PrecompiledInst::Compiled4(i, a, b, c) => {
                    asm.push(*i as u8);
                    asm.push(*a as u8);
                    asm.push(*b as u8);
                    asm.push(*c as u8);
                }
            }
        }

        Ok(asm)
    }

    fn precompile(&mut self, insts: &Vec<ParsedInst>) -> Result<Vec<PrecompiledInst>, String> {
        for inst in insts {
            match inst {
                ParsedInst::Label { label } => { self.symbol_table.insert(label.clone(), self.pos); }
                ParsedInst::Nop => self.inst_1(Inst::Nop),
                ParsedInst::Exit => self.inst_1(Inst::Exit),
                ParsedInst::Jump { label } => {
                    self.buffer.push(PrecompiledInst::JumpPlaceHolder(label.clone(), self.pos));
                    self.pos += 2;
                }
                ParsedInst::Then => self.inst_1(Inst::Then),
                ParsedInst::Otherwise => self.inst_1(Inst::Otherwise),
                ParsedInst::SetByte { dst, val } => self.inst_3(Inst::SetByte, *dst as u8, *val),
                ParsedInst::SetShort { dst, val } => self.inst_4(Inst::SetShort, *dst as u8, (*val >> 8) as u8, *val as u8),
                ParsedInst::Push { src } => self.inst_2(Inst::Push, *src as u8),
                ParsedInst::Pop { dst } => self.inst_2(Inst::Pop, *dst as u8),
                ParsedInst::Add { dst, src } => self.inst_3(Inst::Add, *dst as u8, *src as u8),
                ParsedInst::Sub { dst, src } => self.inst_3(Inst::Sub, *dst as u8, *src as u8),
                ParsedInst::Mul { dst, src } => self.inst_3(Inst::Mul, *dst as u8, *src as u8),
                ParsedInst::Div { dst, src } => self.inst_3(Inst::Div, *dst as u8, *src as u8),
                ParsedInst::Mod { dst, src } => self.inst_3(Inst::Mod, *dst as u8, *src as u8),
                ParsedInst::Neg { dst } => self.inst_2(Inst::Neg, *dst as u8),
                ParsedInst::GreaterThan { left, right } => self.inst_3(Inst::GreaterThan, *left as u8, *right as u8),
                ParsedInst::LessThan { left, right } => self.inst_3(Inst::LessThan, *left as u8, *right as u8),
                ParsedInst::GreaterEqual { left, right } => self.inst_3(Inst::GreaterEqual, *left as u8, *right as u8),
                ParsedInst::LessEqual { left, right } => self.inst_3(Inst::LessEqual, *left as u8, *right as u8),
                ParsedInst::Equal { left, right } => self.inst_3(Inst::Equal, *left as u8, *right as u8),
                ParsedInst::NotEqual { left, right } => self.inst_3(Inst::NotEqual, *left as u8, *right as u8),
                ParsedInst::Return => self.inst_1(Inst::Return),
                ParsedInst::Call { label } => {
                    self.buffer.push(PrecompiledInst::CallPlaceHolder(label.clone()));
                    self.pos += 3;
                }
                ParsedInst::Mov { dst, src } => self.inst_3(Inst::Mov, *dst as u8, *src as u8),
                ParsedInst::Debug { src, mode } => self.inst_3(Inst::Debug, *src as u8, *mode as u8),
            }
        }

        self.inst_1(Inst::Exit);
        Ok(self.buffer.clone())
    }

    fn inst_1(&mut self, i: Inst) {
        self.buffer.push(PrecompiledInst::Compiled1(i));
        self.pos += 1;
    }

    fn inst_2(&mut self, i: Inst, a: u8) {
        self.buffer.push(PrecompiledInst::Compiled2(i, a));
        self.pos += 2;
    }

    fn inst_3(&mut self, i: Inst, a: u8, b: u8) {
        self.buffer.push(PrecompiledInst::Compiled3(i, a, b));
        self.pos += 3;
    }

    fn inst_4(&mut self, i: Inst, a: u8, b: u8, c: u8) {
        self.buffer.push(PrecompiledInst::Compiled4(i, a, b, c));
        self.pos += 4;
    }
}