pub mod auxvt;

use std::ffi::CString;
use std::os::unix::io::AsRawFd;

use crate::Binary;
use crate::error;
use crate::loader64;
use auxvt::*;

use crate::{debug};

#[derive(Debug)]
pub struct Binary64 {
    pub ep: u64,
    pub stack: u64,
}

impl Binary64 {
    pub const fn new(ep: u64, stack: u64) -> Self {
        Self {
            ep,
            stack
        }
    }

}

impl Binary for Binary64 {
    fn run_binary(
        &mut self,
        argc: u64,
        vec_stackb: Vec<CString>,
        if_debug: bool,
    ) -> error::Result<()> {
        /*
            argv: argv[0] filename of itself
                        argv[1] target
                        ...
        */

        let target: &str = "b773d8799365b679247102a9424db5cd";
        let fbuf: Vec<u8>;
        let fd_w: std::fs::File;

        assert!(argc >= 2);

        debug!(
            format!("{}{}", "[+] copy ", vec_stackb[1].to_str().unwrap()),
            if_debug
        );

        match loader64::copy_wrapper(&vec_stackb[1].to_str().unwrap(), target) {
            Ok(_) => (),
            Err(_) => panic!("[-] Fatal copy"),
        };

        let tuple = loader64::open_wrapper(&target)?;

        fbuf = tuple.0;
        fd_w = tuple.1;

        let header: xmas_elf::header::Header = match xmas_elf::header::parse_header(&fbuf[..]) {
            Ok(header) => header,
            Err(error) => panic!("{}", error),
        };

        let mut auxv = Auxvt::new_null();

        auxv.fd = fd_w.as_raw_fd() as u64;
        auxv.target_name = vec_stackb[1].as_ptr() as u64;

        if loader64::is_rel(&header, &fbuf)? == true {
            debug!(
                format!(
                    "{}{:}",
                    "[+] PIC - Position independant executable => ",
                    vec_stackb[1].to_str().unwrap()
                ),
                if_debug
            );
        }

        self.ep = loader64::manual_map(&header, &fbuf, &mut auxv, if_debug, false)?.0;

        debug!(format!("{}{:x}", "=> Entry point: ", self.ep), if_debug);

        self.stack = loader64::pop_stack(&auxv, &vec_stackb, argc)?;

        self.run()?;

        Ok(())
    }

    fn run(&self) -> error::Result<()> {
        unsafe {
            asm!(
            "mov rsp, {1}",
            "mov rax, {0}",
            "xor rbx, rbx",
            "xor rcx, rcx",
            "xor rdx, rdx",
            "xor rdi, rdi",
            "xor rsi, rsi",
            "xor r8, r8",
            "xor r9, r9",
            "xor r10, r10",
            "xor r11, r11",
            "xor r12, r12",
            "xor r13, r13",
            "xor r14, r14",
            "xor r15, r15",
            "xor rbp, rbp",
            "jmp rax", in(reg) self.ep,
                       in(reg) self.stack
            );
        };

        Ok(())
    }
}