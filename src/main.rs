use std::env;

mod rvm_file;
mod rvm_htab;
mod rvm_lex;
mod rvm_preprocessor;
mod rvm_memory;
mod rvm_prog;
mod rvm;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = rvm::RvmCtx::new();

    if vm.rvm_vm_interpret(args[1].as_str()) == 0 {
        vm.rvm_vm_run();
    }
}
