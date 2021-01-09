#[derive(Debug, Clone, Copy)]
pub struct Auxvt {
    pub base_interp: u64,
    pub base_phdr: u64,
    pub sz_phdr_entry: u64,
    pub n: u64,
    pub ep: u64,
    pub target_name: u64,
    pub fd: u64,
}

impl Auxvt {
    pub const fn new(
        base_interp: u64,
        base_phdr: u64,
        sz_phdr_entry: u64,
        n: u64,
        ep: u64,
        target_name: u64,
        fd: u64,
    ) -> Self {
        Self {
            base_interp,
            base_phdr,
            sz_phdr_entry,
            n,
            ep,
            target_name,
            fd,
        }
    }

    pub const fn new_null() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0)
    }
}

///
/// const values for auxilary vectors
pub const AT_PHDR: u64 = 3;
pub const AT_PHENT: u64 = 4;
pub const AT_ENTRY: u64 = 9;
pub const AT_UID: u64 = 11;
pub const AT_EUID: u64 = 12;
pub const AT_GID: u64 = 13;
pub const AT_EGID: u64 = 14;
pub const AT_EXECFN: u64 = 31;
pub const AT_RANDOM: u64 = 25;
pub const AT_PLATFORM: u64 = 15;
pub const AT_PHNUM: u64 = 5;
pub const AT_SYSINFO_EHDR: u64 = 7;