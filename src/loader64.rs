use std::{ffi::CString,
          fs::{copy, OpenOptions, File},
          io::Read,
          os::unix::prelude::*,
          str};

use libc::{
    c_void, memset, mmap, size_t, MAP_ANON, MAP_ANONYMOUS, MAP_FAILED, MAP_FILE, MAP_FIXED,
    MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE,
};
use rand::{thread_rng, Rng};
use xmas_elf::program::{parse_program_header, ProgramHeader, Type};

use crate::error::Result;
use crate::x64::auxvt::{
    Auxvt, AT_PHNUM, AT_EGID, AT_ENTRY, AT_EUID, AT_EXECFN, AT_GID, AT_PHDR, AT_PHENT, AT_PLATFORM,
    AT_RANDOM, AT_SYSINFO_EHDR, AT_UID,
};

use crate::{debug, page_begin, page_offset, round_page};

pub(crate) fn copy_wrapper(filename: &str, target: &str) -> std::io::Result<()> {
    copy(filename, &target)?;
    Ok(())
}

pub(crate) fn open_wrapper(filename: &str) -> Result<(Vec<u8>, File)> {
    let mut fd_w: File = OpenOptions::new()
        .read(true)
        .open(&filename)
        .expect("Unable to open file");

    let mut tbytes = Vec::new();

    fd_w.read_to_end(&mut tbytes)?;
    Ok((tbytes, fd_w))
}

/* lmao */
fn gen_base_address() -> u64 {
    let mut rng = thread_rng();
    (rng.gen_range((0x31337 >> 12) << 12, (0x564bb8db1000 >> 12) << 12) >> 12) << 12
}

/* Is pie based or not */
pub(crate) fn is_rel(header: &xmas_elf::header::Header, fbuf: &Vec<u8>) -> Result<bool> {
    for i in 0..header.pt2.ph_count() {
        if parse_program_header(fbuf, *header, i).unwrap().virtual_addr() == 0x0
            && parse_program_header(fbuf, *header, i).unwrap().get_type().unwrap() == Type::Load {
            return Ok(true);
        }
    }

    Ok(false)
}

fn map_interp(interp: &str, if_debug: bool) -> Result<(u64, u64)> {
    let target: &str = "interp";

    /* copy locally the interp*/
    copy_wrapper(interp, target).unwrap();

    let (fbuf, fd) = open_wrapper(&interp)?;
    let header: xmas_elf::header::Header = xmas_elf::header::parse_header(&fbuf[..]).unwrap();

    let mut auxv: Auxvt = Auxvt::new_null();

    debug!(format!("{} {}", "[+] mapping", interp), if_debug);

    auxv.fd = fd.as_raw_fd() as u64;

    if is_rel(&header, &fbuf)? {
        debug!(
            format!(
                "{}{}",
                "[+] PIC - Position independent executable => ", interp
            ),
            if_debug
        );
    }

    let tuple = manual_map(&header, &fbuf, &mut auxv, if_debug, true)?;
    let (ep, base_addr) = (tuple.0, tuple.1.unwrap());
    debug!(format!("{} {} {}", "[+]", interp, "mapped"), if_debug);

    Ok((ep, base_addr))
}

/// =-=-=-=-=-=--
/// @ph: Program Header to mmap
/// @isrel: Is the binary pie based (true) or not (false)
/// @base_address: base address of the binary if it is not independant executable
/// =-=-=-=-=-=--
/// 
/// map_ptload maps a PT_LOAD segment
/// TODO: patch the crash when it's loading a no-pie binary dynamically linked
fn map_ptload(
    ph: &ProgramHeader,
    isrel: bool,
    to_mmap: *const u8,
    aux: &Auxvt,
    if_debug: bool,
) -> Result<*mut u8> {
    let mmap_chunk: *mut c_void;

    let mut prot = 0x0;
    let flags = MAP_FILE | MAP_PRIVATE | MAP_FIXED;

    let vaddr = ph.virtual_addr();
    let _sz = round_page!(page_offset!(vaddr) + ph.mem_size()) as size_t;
    let offt = page_begin!(ph.offset()) as i64;
    let file_sz = ph.file_size() as size_t;
    let memsz = ph.mem_size() as size_t;

    let mut base_address = 0x0;

    if isrel {
        base_address = to_mmap as u64;
    }

    let seg_start = page_begin!(base_address + vaddr) as u64;
    let seg_end_main = round_page!(base_address + vaddr + file_sz as u64) as u64;

    if ph.flags().is_read() {
        prot |= PROT_READ;
    }

    if ph.flags().is_write() {
        prot |= PROT_WRITE;
    }

    if ph.flags().is_execute() {
        prot |= PROT_EXEC;
    }

    mmap_chunk = unsafe {
        mmap(
            (page_begin!(seg_start) as u64) as *mut c_void,
            round_page!(seg_end_main - seg_start) as usize,
            prot | PROT_WRITE,
            flags,
            aux.fd as i32,
            offt,
        )
    };

    if memsz > file_sz {
        /*
        let map_fsz = round_page!(page_offset!(vaddr as u64) + file_sz as u64);
        let map_memsz = round_page!(page_offset!(vaddr as u64) + memsz as u64);
        */

        let bss_start = base_address + vaddr + file_sz as u64;
        let map_bss_start = round_page!(bss_start) as u64 + 1;
        let _map_addr_bss = map_bss_start;

        let seg_end = round_page!(base_address + memsz as u64 + vaddr) as u64;

        unsafe {
            memset(
                (bss_start) as *mut c_void,
                0x0,
                ((map_bss_start) - bss_start) as usize,
            )
        };

        if seg_end > map_bss_start {
            unsafe {
                let test = mmap(
                    (_map_addr_bss) as *mut c_void,
                    round_page!(seg_end - map_bss_start) as usize,
                    prot | PROT_WRITE,
                    MAP_ANONYMOUS | MAP_FIXED | MAP_PRIVATE,
                    -1,
                    0x0,
                );
                if test == MAP_FAILED {
                    panic!("...");
                }
            };
            debug!(
                format!(
                    "{}{:x} - {:x} {:x}",
                    "[+] ",
                    page_begin!(seg_start) as u64,
                    ((seg_start) as u64) + round_page!((seg_end_main - seg_start)) as u64,
                    round_page!(seg_end_main - seg_start)
                ),
                if_debug
            );
            debug!(
                format!(
                    "{}{:x} - {:x} {:x}",
                    "[+] ",
                    (_map_addr_bss),
                    (_map_addr_bss as u64 + round_page!(seg_end - bss_start) as u64) as u64,
                    round_page!(seg_end - bss_start)
                ),
                if_debug
            );

            return Ok(mmap_chunk as *mut u8);
        }
    }

    debug!(
        format!(
            "{}{:x} - {:x} {:x}",
            "[+] ",
            page_begin!(seg_start) as u64,
            round_page!((page_begin!(seg_start) as u64) + (seg_end_main - seg_start)),
            round_page!(seg_end_main - seg_start)
        ),
        if_debug
    );

    Ok(mmap_chunk as *mut u8)
}

/// Cast *const *const u8 of n entries to Vec<CString>
pub fn raw_to_cstr(s: *const *const u8, n: u64) -> Vec<CString> {
    let mut r = Vec::new();

    for i in 0..n {
        unsafe {
            r.push(CString::from_raw(
                *((s as u64 + i * 8) as *mut u64) as *mut i8,
            ));
        }
    }

    let mut idx = 0;

    while unsafe { *((s as u64 + (n + idx + 1) * 8) as *mut u64) } != 0x0 {
        unsafe {
            r.push(CString::from_raw(
                *((s as u64 + (n + idx + 1) * 8) as *mut u64) as *mut i8,
            ))
        };

        idx += 1;
    }

    r
}

/// # Arguments
/// * `aux`: auxilary vector struct
/// * `args`: vec of NULL byte terminated strings
/// * `n`: argc
/// 
/// It returns a custom stack parsed according to the aparameters
pub(crate) fn pop_stack(aux: &Auxvt, args: &Vec<CString>, n: u64) -> Result<u64> {
    let random = "abfgehrjbfgshbctefgshjkql\0".as_bytes().as_ptr() as *mut u8;
    let platform = "x86_64\0".as_bytes().as_ptr() as *mut u8;

    let mut vec = Vec::new();

    vec.push(n - 1);

    for i in 1..n as usize {
        vec.push(args[i].as_ptr() as u64);
    }

    vec.push(0x0 as u64);

    for i in (n as usize)..args.len() {
        vec.push(args[i].as_ptr() as u64);
    }

    vec.push(0x0 as u64);

    if aux.base_interp != 0x0 {
        vec.push(AT_SYSINFO_EHDR);
        vec.push(aux.base_interp);
        // AT_SYSINFO_EHDR = 7
    }

    vec.push(AT_PHDR);
    vec.push(aux.base_phdr);
    // AT_PHDR = 3

    vec.push(AT_PHENT);
    vec.push(aux.sz_phdr_entry);
    // AT_PHENT = 4

    vec.push(AT_PHNUM);
    vec.push(aux.n);
    // argc

    vec.push(AT_ENTRY);
    vec.push(aux.ep);
    // AT_ENTRY = 9

    vec.push(AT_UID);
    vec.push(1000);
    // AT_UID = 11

    vec.push(AT_EUID);
    vec.push(1000);
    // AT_EUID = 12

    vec.push(AT_GID);
    vec.push(1000);
    // AT_GID = 13

    vec.push(AT_EGID);
    vec.push(1000);
    // AT_EGID = 14

    vec.push(AT_EXECFN);
    vec.push(aux.target_name);
    // AT_EXECFN = 31

    vec.push(AT_RANDOM);
    vec.push(random as u64);
    // AT_RANDOM = 25

    vec.push(AT_PLATFORM);
    vec.push(platform as u64);
    // AT_PLATFORM = 15

    let prot = PROT_WRITE | PROT_READ;
    let flags = MAP_ANON | MAP_PRIVATE;

    let mmap_chunk =
        unsafe { mmap(0x0 as *mut c_void, 0x32000, prot, flags, -1, 0x0) };
    /* We mmap a stack */

    for i in 0..vec.len() {
        unsafe { *(((mmap_chunk as usize) + 0x25000 + (i * 8)) as *mut u64) = vec[i] };
    }

    Ok(mmap_chunk as u64 + 0x25000)
}

/// # Arguments
/// * `header`: ref to the header of the target binary
/// * `fbuf`: the target binary mapped
/// * `aux`: auxilary vector struct
/// * `is_debug`: enable or not debug output
///
/// manual_map maps all the PT_LOAD segments of a binary and checks if
/// there is an interpreter, if it is, it calls map_interp
pub(crate) fn manual_map(
    header: &xmas_elf::header::Header,
    fbuf: &Vec<u8>,
    aux: &mut Auxvt,
    if_debug: bool,
    is_interp: bool
) -> Result<(u64, Option<u64>)> {
    let mut program_header: ProgramHeader;
    let mut base_address: *const u8 = 0x0 as *const u8;

    let is_rel: bool = is_rel(&header, &fbuf)?;

    if is_rel {
        base_address = gen_base_address() as *const u8;
    } else {
        base_address = parse_program_header(&fbuf, header.clone(), 0)
            .unwrap()
            .virtual_addr() as *const u8;
    }

    let mut has_interp: bool = false;

    let mut base_interp: u64 = 0x0;
    let mut ep: u64 = header.pt2.entry_point();

    for i in 0..header.pt2.ph_count() {
        program_header = parse_program_header(&fbuf, header.clone(), i).unwrap();

        match program_header.get_type().unwrap() {
            Type::Load => {
                map_ptload(
                    &program_header,
                    is_rel,
                    base_address as *const u8,
                    aux,
                    if_debug,
                )?;
            },

            Type::Interp => {
                /* If we reach the Interp segment, it means that the binary isn't statically linked, 
                so we push each bytes of the interp's name which are in this segment */
                let mut interp: Vec<u8> = Vec::new();

                has_interp = true;

                for i in 0..program_header.file_size() - 1 {
                    interp.push(fbuf[(program_header.offset() + i) as usize]);
                }

                /* cast vec to str */
                let array_interp: &[u8] = interp.as_slice();
                let str_interp = str::from_utf8(array_interp).unwrap();

                debug!(format!("{}{}", "[+] interp found: ", str_interp), if_debug);

                /* And we mmap the interp from its filename */
                let t = map_interp(&str_interp, if_debug).unwrap();

                ep = t.0;
                base_interp = t.1;

                debug!(
                    format!(
                        "{}{}: {:x?}",
                        "[+] Entry point for ",
                        str_interp,
                        base_interp + ep
                    ),
                    if_debug
                );
            },

            _ => continue
        }
    }

    if has_interp {
        aux.base_interp = base_interp;
    }

    aux.base_phdr = header.pt2.ph_offset() + base_address as u64;
    /* Set certain auxilary vectors */
    aux.sz_phdr_entry = header.pt2.ph_entry_size() as u64;
    aux.n = header.pt2.ph_count() as u64;
    /* ep in the binary */
    aux.ep = header.pt2.entry_point() + base_address as u64;

    debug!("[+] All the PT_LOAD are mapped !", if_debug);

    if is_interp {
       Ok((ep, Some(base_address as u64)))
    } else if has_interp {
        /* we return the entry point in the interpreter */
        debug!(
            format!(
                "{}{:x}",
                "[+] ep in the target: ",
                parse_program_header(&fbuf, header.clone(), 0)
                    .unwrap()
                    .virtual_addr()
                    + base_address as u64
                    + ep
            ),
            if_debug
        );

        Ok((base_interp as u64 + ep, None))
    } else if is_rel {
        /* or directly the vaddr of the entry point (pie based binary) */
        Ok((base_address as u64 + header.pt2.entry_point(), None))
    } else {
        Ok((header.pt2.entry_point(), None))
    }
}
