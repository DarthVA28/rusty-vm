use core::panic;

const MIN_MEMORY_SIZE: usize = 64 * 1024 * 1024; // 64 MB
const NUM_REGISTERS: usize = 17;
const MIN_STACK_SIZE: usize = 2 * 1024 * 1024; // 2 MB

#[derive(Clone)]
pub enum RvmRegU {
    I32(i32),
    I32ADDR(i32),
    I16 {h : i16, l : i16},
}

pub struct RvmMem {
    pub flags: u32, 
    pub remainder: i32,
    pub mem_space: Vec<u8>,
    pub registers: Vec<RvmRegU>
}

impl RvmMem {
    pub fn new() -> Self {
        RvmMem {
            flags: 0,
            remainder: 0,
            mem_space: vec![0; MIN_MEMORY_SIZE],
            registers: vec![RvmRegU::I32(0); NUM_REGISTERS]
        }
    }

    pub fn rvm_stack_create(&mut self) {
        // 0x7 will have the base of the stack
        // 0x6 will have the current top of the stack
        // 
        self.registers[0x7] = RvmRegU::I32ADDR(MIN_STACK_SIZE as i32);
        self.registers[0x6] = RvmRegU::I32ADDR(MIN_STACK_SIZE as i32);
    }

    pub fn rvm_stack_push(&mut self, item : i32) {
        if let RvmRegU::I32ADDR(mut sp) = self.registers[0x6] {
            let new_sp = sp - 4;
            self.mem_space[new_sp as usize..sp as usize].copy_from_slice(&item.to_le_bytes());
        }
    }

    pub fn rvm_stack_pop(&mut self) -> i32 {
        if let RvmRegU::I32ADDR(mut sp) = self.registers[0x6] {
            let new_sp = sp + 4;
            let ret = i32::from_le_bytes(self.mem_space[sp as usize..new_sp as usize].try_into().unwrap());
            self.registers[0x6] = RvmRegU::I32ADDR(new_sp);
            ret
        } else {
            panic!("Invalid stack pointer");
        }
    }
}
