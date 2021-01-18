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
dumbleodor = {path="/ur/path/of/dumbleodor"}

# ...

Output:

$ cargo run /home/nasm/.cargo/bin/cargo --version
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/test_dumbloader /home/nasm/.cargo/bin/cargo --version`
cargo 1.49.0-nightly (2af662e22 2020-11-12)
*/
```
