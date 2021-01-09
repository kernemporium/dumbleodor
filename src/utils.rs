// Loader utils
#[macro_export]
macro_rules! page_begin {
    ($x:expr) => {
        ($x as i64 & !0xfff)
    };
}

#[macro_export]
macro_rules! round_page {
    ($x:expr) => {
        (page_begin!($x) + 0xfff)
    };
}

#[macro_export]
macro_rules! page_offset {
    ($x:expr) => {
        ($x & 0xfff)
    };
}

#[macro_export]
macro_rules! debug {
    ($msg:expr) => {
        println!("{}", $msg)
    };

    ($msg:expr, $if_debug:expr) => {
        if $if_debug == true {
            println!("{}", $msg);
        }
    };
}
