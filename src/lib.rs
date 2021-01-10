#![feature(asm)]

use std::ffi::CString;

pub mod error;
pub mod x64;
pub mod loader64;
mod utils;

pub trait Binary {
    fn run_binary(&mut self, argc: u64, argv: Vec<CString>, if_debug: bool) -> error::Result<()>;
    fn run(&self) -> error::Result<()>;

    fn credits() {
        println!("[** -=-=-=-=-=- ## Developed by nasm - RE ## -=-=-=-=-=-=- **]");
    }
}


