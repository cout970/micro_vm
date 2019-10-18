#![allow(dead_code)]
// cargo watch -c -q -s 'cargo +nightly rustc -- -Awarnings -Zno-codegen && cargo test'

use crate::assembler::{ParsedInst, Parser, print_tokens, read_all_tokens, Compiler};

mod vm;
mod assembler;

fn main() {
    let bytes = include_bytes!("../sample.asm");
    let tokens = read_all_tokens(bytes);
    print_tokens(bytes, &tokens);

    let mut parser = Parser::new(bytes, tokens);
    let mut compiler = Compiler::new();

    let parsed = parser.parse().expect("Unable to parse");

    let pre = compiler.precompile(&parsed).expect("Unable to compiled");

    println!("{:#?}", pre);
//    let mut vm = VM::new();
//    vm.set(Inst::SetByte as u8);
//    vm.set(0);
//    vm.set(0xFF);
//
//    vm.set(Inst::Neg as u8);
//    vm.set(0);
//
//    vm.set(Inst::Exit as u8);
//    vm.reset();
//
//    vm.run();
//    vm.print();
}

#[cfg(test)]
mod tests {
    use crate::assembler::Compiler;

    use super::*;

    #[test]
    fn test_tokenizer() {
        let bytes = include_bytes!("../sample.asm");
        let tokens = read_all_tokens(bytes);

        print_tokens(bytes, &tokens);
    }

    #[test]
    fn test_parser() {
        let bytes = include_bytes!("../sample.asm");
        let tokens = read_all_tokens(bytes);
        let mut parser = Parser::new(bytes, tokens);

        match parser.parse() {
            Ok(ok) => { println!("{:#?}", ok) }
            Err(err) => { panic!("{}", err) }
        };
    }

    #[test]
    fn test_compiler() {
        let bytes = include_bytes!("../sample.asm");
        let tokens = read_all_tokens(bytes);
        let mut parser = Parser::new(bytes, tokens);

        let mut compiler = Compiler::new();
        let parsed = parser.parse().expect("Unable to parse");

        compiler.precompile(&parsed).expect("Unable to compiled");
    }
}