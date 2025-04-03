use crate::rvm_file;
use crate::rvm_htab;
use crate::rvm_lex;
use crate::rvm_memory::RvmMem;
use crate::rvm_memory::RvmRegU;
use crate::rvm_preprocessor;
use crate::rvm_prog::RvmProg;
use std::collections::HashMap;
use std::fs::File;

const RvmOpcodeMap : [&str; 32] = [
    "nop", "int", "mov",
    "push", "pop", "pushf", "popf",
    "inc", "dec", "add", "sub", "mul", "div", "mod", "rem",
    "not", "xor", "or", "and", "shl", "shr",
    "cmp", "jmp", "call", "ret",
    "je", "jne", "jg", "jge", "jl", "jle",
    "prn"
];

const RvmRegisterMap : [&str; 17] = [
    "eax", "ebx", "ecx", "edx",
    "esi", "edi", "esp", "ebp",
    "eip", "r08", "r09", "r10", "r11",
    "r12", "r13", "r14", "r15"
];
pub struct RvmCtx {
    pub prog: RvmProg,
    pub mem: RvmMem
}

impl RvmCtx {
    pub fn new() -> Self {
        let mut ctx = RvmCtx {
            prog: RvmProg::new(),
            mem: RvmMem::new()
        };
        ctx.mem.rvm_stack_create();
        ctx
    }

    fn instr_to_opcode(instr: &str) -> i32 {
        let mut opcode = -1;
        for (i, op) in RvmOpcodeMap.iter().enumerate() {
            if instr == *op {
                opcode = i as i32;
                break;
            }
        }
        opcode
    }

    fn token_to_register(&mut self, tok: &str) -> Option<i32> {
        for (i, r) in RvmRegisterMap.iter().enumerate() {
            if tok == *r {
                if let RvmRegU::I32(reg_val) = self.mem.registers[i] {
                    return Some(reg_val);
                } else {
                    return None;
                }
            }
        }
        None
    }

    pub fn rvm_add_value(&mut self, val: i32) -> i32 {
        self.prog.values.push(val); 
        val
    }

    pub fn rvm_parse_value(&mut self, s: &str) -> i32 {
        let res: Result<i32, std::num::ParseIntError>;
        if let Some(delimiter_index) = s.find('|') {
            let identifier = &s[delimiter_index + 1..];
    
            let base = match identifier.chars().next() {
                Some('h') => 16, 
                Some('b') => 2,  
                _ => 10,        
            };
            // Parse the value using the determined base
            res = i32::from_str_radix(&s[..delimiter_index], base);
        } 
        else {
            res = s.parse::<i32>();
        }
        match res {
            Ok(value) => value,
            Err(_) => {
                // Handle the error case, e.g., return a default value or panic
                println!("Error parsing value: {}", s);
                0
            }
        }
    }

    pub fn rvm_parse_labels(&mut self, tokens: &Vec<Vec<String>>) -> i32 {
        let mut num_instr : u32 = 0;
        for i in 0..tokens.len() {
            let mut valid_instruction : bool = false;
            for j in 0..tokens[i].len() {
                let mut tok = tokens[i][j].clone();
                
                // If the token is empty skip it
                if tok.is_empty() {
                    continue;
                }

                // Check the source line for a valid instruction
                if RvmCtx::instr_to_opcode(&tok) != -1 {
                    valid_instruction = true;
                }

                // Check for a label delimiter 
                if let Some(label_delimiter_pos) = tok.find(':') {
                    tok.truncate(label_delimiter_pos);

                    // If the label is "start" make it the entry point
                    if tok == "start" {
                        // Set the entry point to the current instruction
                        self.prog.start = num_instr as i32;
                    }

                    // Check if the label already exists
                    if let Some(label_addr) = self.prog.labels.rvm_htab_find(&tok) {
                        println!("Error: Duplicate label found: {}", tok);
                        return 1;
                    } 

                    // Add the label to the hash table
                    self.prog.labels.rvm_htab_add(&tok, num_instr as i32, "");
                } else {
                    continue;
                }
            }
            if (valid_instruction) {
                num_instr += 1;
            }
        }
        0
    }

    pub fn rvm_parse_instr(&mut self, instr_toks: & Vec<String>) -> (i32, usize) {
        // Find the instruction in the opcode map
        for mut i in 0..instr_toks.len() {
            // Check if instr_toks[i] is empty 
            if instr_toks[i].is_empty() {
                continue;
            }
            let opcode = RvmCtx::instr_to_opcode(&instr_toks[i]);

            if opcode == -1 {
                continue
            }

            return (opcode, i); 
        }
        (-1, 0)
    }

    pub fn rvm_parse_args(&mut self, instr_toks: & Vec<String>, instr_place : usize) -> Vec<i32> {
        let mut args = Vec::new();
        for i in instr_place + 1..instr_toks.len() {
            if instr_toks[i].is_empty() {
                continue;
            }
            
            let mut token = instr_toks[i].clone();
            if let Some(newline_pos) = token.find('\n') {
                token.truncate(newline_pos);
            }

            // Check if the token specifies a register
            let reg: Option<i32> = self.token_to_register(&token);
            if let Some(regp) = reg {
                args.push(regp);
                continue;
            }

            // Check to see whether the token specifies an address
            if token.starts_with('[') {
                if let Some(end_pos) = token.find(']') {
                    token.truncate(end_pos);
                    let addr_val = self.rvm_parse_value(&token);
                    // Get the value at the address from the memory
                    args.push(self.mem.mem_space[addr_val as usize] as i32);
                    continue;
                } 
            }

            // Check if the argument is a label
            if let Some(addr) = self.prog.labels.rvm_htab_find(&token) {
                args.push(self.rvm_add_value(addr));
            }

            // Otherwise, parse the token as a value
            let tok_val = self.rvm_parse_value(&token);
            args.push(self.rvm_add_value(tok_val));
        }
        args
    }

    pub fn rvm_parse_program(&mut self, tokens: &Vec<Vec<String>>) -> i32{
        for i in 0..tokens.len() {
            let (opcode, instr_place) = self.rvm_parse_instr(&tokens[i]);
            
            if opcode == -1 {
                continue;
            }

            let args = self.rvm_parse_args(&tokens[i], instr_place);

            if args.len() == 0 {
                continue;
            }

            // Add the instruction to the program
            self.prog.instructions.push(opcode);

            // Add the arguments to the program
            self.prog.args.push(args);
        }
        // Sentinel instructions
        self.prog.args.push(vec![]); 
        self.prog.instructions.push(-0x1); 
        return 0
    }

    pub fn rvm_step(&mut self, instr_idx : i32) -> i32 {
        let mut new_idx = instr_idx; 
        let args = &self.prog.args[instr_idx as usize];

        match self.prog.instructions[instr_idx as usize] {
            0x0 => {
                // NO_OP
            }
            0x1 => {
                // INT (unimplemented)
            } 
            0x2 => {
                // MOV
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    self.mem.registers[dest as usize] = RvmRegU::I32(src);
                }
            } 
            0x3 => {
                // PUSH
                if args.len() == 1 {
                    let src = args[0];
                    self.mem.rvm_stack_push(src);
                }
            }
            0x4 => {
                // POP
                if args.len() == 1 {
                    let dest = args[0];
                    let val = self.mem.rvm_stack_pop();
                    self.mem.registers[dest as usize] = RvmRegU::I32(val);
                }
            }
            0x5 => {
                // PUSHF
                self.mem.rvm_stack_push(self.mem.flags as i32);
            }
            0x6 => {
                // POPF
                let val = self.mem.rvm_stack_pop();
                self.mem.flags = val as u32;
                
            }
            0x7 => {
                // INC
                if args.len() == 1 {
                    let reg = args[0];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[reg as usize] {
                        *val += 1;
                    }
                }
            }
            0x8 => {
                // DEC
                if args.len() == 1 {
                    let reg = args[0];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[reg as usize] {
                        *val -= 1;
                    }
                }
            }
            0x9 => {
                // ADD
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val += src;
                    }
                }
            }
            0xA => {
                // SUB
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val -= src;
                    }
                }
            }
            0xB => {
                // MUL
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val *= src;
                    }
                }
            }
            0xC => {
                // DIV
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val /= src;
                    }
                }
            }
            0xD => {
                // MOD
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        self.mem.remainder = *val % src;
                    }
                }
            }
            0xE => {
                // REM
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val = self.mem.remainder;
                    }
                }
            }
            0xF => {
                // NOT
                if args.len() == 1 {
                    let dest = args[0];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val = !*val;
                    }
                }
            }
            0x10 => {
                // XOR
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val ^= src;
                    }
                }
            }
            0x11 => {
                // OR
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val |= src;
                    }
                }
            }
            0x12 => {
                // AND
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val &= src;
                    }
                }
            }
            0x13 => {
                // SHL
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val <<= src;
                    }
                }
            }
            0x14 => {
                // SHR
                if args.len() == 2 {
                    let dest = args[0];
                    let src = args[1];
                    if let RvmRegU::I32(ref mut val) = self.mem.registers[dest as usize] {
                        *val >>= src;
                    }
                }
            }
            0x15 => {
                // CMP
                if args.len() == 2 {
                    let reg1 = args[0];
                    let reg2 = args[1];
                    if let RvmRegU::I32(val1) = self.mem.registers[reg1 as usize] {
                        if let RvmRegU::I32(val2) = self.mem.registers[reg2 as usize] {
                            self.mem.flags = ((val1 == val2) as u32) | (((val1 > val2) as u32) << 1);
                        }
                    }
                }
            }
            0x16 => {
                // JMP
                if args.len() == 1 {
                    let addr = args[0];
                    new_idx = addr;
                }
            }
            0x17 => {
                // CALL
                if args.len() == 1 {
                    let addr = args[0];
                    self.mem.rvm_stack_push(instr_idx);
                    new_idx = addr;
                }
            }
            0x18 => {
                // RET
                if args.len() == 0 {
                    new_idx = self.mem.rvm_stack_pop();
                }
            }
            0x19 => {
                // JE
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x1 != 0 {
                        new_idx = addr-1;
                    }
                }
            }
            0x1A => {
                // JNE
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x1 == 0 {
                        new_idx = addr-1;
                    }
                }
            }
            0x1B => {
                // JG
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x2 != 0 {
                        new_idx = addr-1;
                    }
                }
            }
            0x1C => {
                // JGE
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x3 == 0  {
                        new_idx = addr-1;
                    }
                }
            }
            0x1D => {
                // JL
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x3 != 0 {
                        new_idx = addr-1;
                    }
                }
            }
            0x1E => {
                // JLE
                if args.len() == 1 {
                    let addr = args[0];
                    if self.mem.flags & 0x2 != 0 {
                        new_idx = addr-1;
                    }
                }
            }
            0x1F => {
                // PRN
                if args.len() == 1 {
                    let reg = args[0];
                    if let RvmRegU::I32(val) = self.mem.registers[reg as usize] {
                        println!("{}", val);
                    }
                }   
            }
            _ => {
                panic!("Unknown opcode: {}", self.prog.instructions[instr_idx as usize]);
            }
        }
        
        new_idx
    }

    pub fn rvm_vm_interpret(&mut self, filename: &str) -> i32 {
        let mut filp : Option<File> = None;
        // Attempt to open the file 
        if !filename.is_empty() {
            // Try twice
            for _ in 0..2 {
                let res = rvm_file::rvm_fopen(filename, ".vm", "r");
                match res {
                    Ok(file) => {
                        filp = Some(file);
                        break;
                    }
                    Err(e) => {
                        println!("Error opening file: {}", e);
                        filp = None;
                    }
                }
            }
        }

        if filp.is_none() {
            println!("Error: Unable to open file {}", filename);
            return 1;
        }

        let mut source = rvm_file::rvm_fcopy(filp.as_mut().unwrap()).unwrap();

        let mut preprocessor = rvm_preprocessor::RvmPreprocessor::new();

        let err = preprocessor.rvm_preprocess(&mut source);
        if err < 0 {
            return 1;
        }

        self.prog.defines = preprocessor.defines;

        let mut lexer_ctx = rvm_lex::RvmLexerCtx::new();

        lexer_ctx.rvm_lex(&source, &self.prog.defines);

        if self.rvm_parse_labels(&lexer_ctx.tokens) != 0 {
            return 1;
        }
        
        if self.rvm_parse_program(&lexer_ctx.tokens) != 0 {
            return 1;
        }

        return 0;
    }

    pub fn rvm_vm_run(&mut self) {
        // Set the index for the instruction to be executed
        self.mem.registers[0x8] = RvmRegU::I32(self.prog.start);
        let mut instr_idx = self.prog.start;
        loop {
            if self.prog.instructions[instr_idx as usize] == -0x1 {
                break;
            }
            instr_idx = self.rvm_step(instr_idx);
        }
    }
}


