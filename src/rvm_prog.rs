use crate::rvm_htab;
use crate::rvm_htab::RvmHtabCtx;
use crate::rvm_memory;

pub struct RvmProg {
    pub start: i32,
    pub instructions: Vec<i32>,
    pub args: Vec<Vec<i32>>,
    pub values: Vec<i32>,
    pub defines: RvmHtabCtx,
    pub labels: RvmHtabCtx
}

impl RvmProg {
    pub fn new() -> Self {
        Self {
            start: 0,
            instructions: Vec::new(),
            args: Vec::new(),
            values: Vec::new(),
            defines: RvmHtabCtx::new(),
            labels: RvmHtabCtx::new()
        }
    }
}