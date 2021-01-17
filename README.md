# dumbleodor
Library to load executable files

# Example
```rust
#![feature(start)]

use std::ffi::CString;
use dumbleodor::*;

#[start]
pub fn main(argc: isize, argv: *const *const u8) -> isize {
    let mut binary: x64::Binary64 = x64::Binary64::new(0, 0);
    let argv_array: Vec<CString> = loader64::raw_to_cstr(argv, argc as u64);

    binary.run_binary(argc as u64, argv_array, true).unwrap();

    0
}

/*
In the Cargo.toml you have to include the dumbleodor's path:

# ...

[dependencies]
dumbleodor = {path="../dumbleodor"}

# ...

Output:

$ cargo run /bin/uname -a
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/test_dumbloader /bin/uname -a`
[+] copy /bin/uname
[+] PIC - Position independant executable => /bin/uname
[+] interp found: /lib64/ld-linux-x86-64.so.2
[+] mapping /lib64/ld-linux-x86-64.so.2
[+] PIC - Position independent executable => /lib64/ld-linux-x86-64.so.2
[+] 16eadcd2d000 - 16eadcd2dfff fff
[+] 16eadcd2e000 - 16eadcd50fff 22fff
[+] 16eadcd51000 - 16eadcd58fff 7fff
[+] 16eadcd5a000 - 16eadcd5bfff 1fff
[+] 16eadcd5c000 - 16eadcd5dfff 1fff
[+] All the PT_LOAD are mapped !
[+] /lib64/ld-linux-x86-64.so.2 mapped
[+] Entry point for /lib64/ld-linux-x86-64.so.2: 16eadcd2e100
[+] 1972eb348000 - 1972eb349fff 1fff
[+] 1972eb34a000 - 1972eb34dfff 3fff
[+] 1972eb34e000 - 1972eb34ffff 1fff
[+] 1972eb351000 - 1972eb352fff 1fff
[+] All the PT_LOAD are mapped !
[+] ep in the target: 1972eb34aaa0
=> Entry point: 16eadcd2e100
Linux off 5.8.0-38-generic #43~20.04.1-Ubuntu SMP Tue Jan 12 16:39:47 UTC 2021 x86_64 x86_64 x86_64 GNU/Linux
*/
```
