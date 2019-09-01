#![allow(non_camel_case_types)]

use std::ops::Range;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::transmute;

use static_assertions::assert_eq_size;

// Minimal ELF support, sufficient for a very simple 64-bit static Linux
// executable.

// Sources:
// * /usr/include/elf.h
// * https://wiki.osdev.org/ELF_Tutorial
// * https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// * https://www.muppetlabs.com/~breadbox/software/tiny/teensy.html
// * https://unix.stackexchange.com/questions/132036/why-does-readelf-show-system-v-as-my-os-instead-of-linux
// * http://www.sco.com/developers/gabi/latest/ch4.eheader.html

type Elf64_Half = u16;
type Elf64_Word = u32;
type Elf64_Xword = u64;
type Elf64_Addr = u64;
type Elf64_Off = u64;

const EI_MAG: Range<usize> = 0..4;
const EI_CLASS: usize = 4;
const EI_DATA: usize = 5;
const EI_VERSION: usize = 6;
const EI_OSABI: usize = 7;

const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const ELFOSABI_SYSV: u8 = 0;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;

const PF_X: u32 = 1 << 0;
const PF_W: u32 = 1 << 1;
const PF_R: u32 = 1 << 2;

const EHDR_SIZE: usize = 64;
const PHDR_SIZE: usize = 56;
const SHDR_SIZE: usize = 64;

assert_eq_size!(ehdr_size_assert; Elf64_Ehdr, [u8; EHDR_SIZE]);
assert_eq_size!(phdr_size_assert; Elf64_Phdr, [u8; PHDR_SIZE]);
assert_eq_size!(shdr_size_assert; Elf64_Shdr, [u8; SHDR_SIZE]);

// ELF header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Elf64_Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: Elf64_Half,
    pub e_machine: Elf64_Half,
    pub e_version: Elf64_Word,
    pub e_entry: Elf64_Addr,
    pub e_phoff: Elf64_Off,
    pub e_shoff: Elf64_Off,
    pub e_flags: Elf64_Word,
    pub e_ehsize: Elf64_Half,
    pub e_phentsize: Elf64_Half,
    pub e_phnum: Elf64_Half,
    pub e_shentsize: Elf64_Half,
    pub e_shnum: Elf64_Half,
    pub e_shstrndx: Elf64_Half,
}

// Program header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Elf64_Phdr {
    pub p_type: Elf64_Word,
    pub p_flags: Elf64_Word,
    pub p_offset: Elf64_Off,
    pub p_vaddr: Elf64_Addr,
    pub p_paddr: Elf64_Addr,
    pub p_filesz: Elf64_Xword,
    pub p_memsz: Elf64_Xword,
    pub p_align: Elf64_Xword,
}

// Symbol header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Shdr {
    pub sh_name: Elf64_Word,
    pub sh_type: Elf64_Word,
    pub sh_flags: Elf64_Xword,
    pub sh_addr: Elf64_Addr,
    pub sh_offset: Elf64_Off,
    pub sh_size: Elf64_Xword,
    pub sh_link: Elf64_Word,
    pub sh_info: Elf64_Word,
    pub sh_addralign: Elf64_Xword,
    pub sh_entsize: Elf64_Xword,
}

pub fn elf64_write(f: &mut impl Write, text: &[u8], bss_size: u64) -> io::Result<()> {
    let size = text.len() as u64;
    let hdr_size = (EHDR_SIZE + 2 * PHDR_SIZE) as u64;
    let hdr_size_padded = (hdr_size + 0x1000 - 1) & !(0x1000 - 1);

    let mut e_ident = [0u8; 16];
    e_ident[EI_MAG].copy_from_slice(b"\x7fELF");
    e_ident[EI_CLASS] = ELFCLASS64;
    e_ident[EI_DATA] = ELFDATA2LSB;
    e_ident[EI_VERSION] = 1;
    e_ident[EI_OSABI] = ELFOSABI_SYSV;

    let ehdr = Elf64_Ehdr {
        e_ident,
        e_type: ET_EXEC,
        e_machine: EM_X86_64,
        e_version: 1,
        e_entry: 0x401000,
        // Put program header immediately after ELF header
        e_phoff: EHDR_SIZE as u64,
        // Don't include a section header table
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: EHDR_SIZE as u16,
        e_phentsize: PHDR_SIZE as u16,
        e_phnum: 2,
        e_shentsize: SHDR_SIZE as u16,
        e_shnum: 0,
        e_shstrndx: 0,
    };

    let phdr_text = Elf64_Phdr {
        p_type: PT_LOAD,
        p_flags: PF_R | PF_X,
        p_offset: hdr_size_padded,
        p_vaddr: 0x401000,
        p_paddr: 0x401000,
        p_filesz: size,
        p_memsz: size,
        p_align: 0x1000,
    };

    let bss_offset = (size + 0x1000 - 1) & !(0x1000 - 1);

    let phdr_bss = Elf64_Phdr {
        p_type: PT_LOAD,
        p_flags: PF_R | PF_W,
        p_offset: hdr_size_padded + bss_offset,
        p_vaddr: 0x401000 + bss_offset,
        p_paddr: 0x401000 + bss_offset,
        p_filesz: 0,
        p_memsz: bss_size,
        p_align: 0x1000,
    };

    unsafe {
        f.write(&transmute::<_, [u8; EHDR_SIZE]>(ehdr))?;
        f.write(&transmute::<_, [u8; PHDR_SIZE]>(phdr_text))?;
        f.write(&transmute::<_, [u8; PHDR_SIZE]>(phdr_bss))?;
    }
    for _ in 0..(hdr_size_padded - hdr_size) {
        f.write(b"0")?;
    }
    f.write(text)?;
    Ok(())
}

fn elf64_read_strtab(f: &mut (impl Read + Seek), ehdr: &Elf64_Ehdr) -> io::Result<Vec<u8>> {
    // Read section header for the string table
    let mut shdr_buf = [0; SHDR_SIZE];
    f.seek(SeekFrom::Start(ehdr.e_shoff + ehdr.e_shstrndx as u64 * SHDR_SIZE as u64))?;
    f.read(&mut shdr_buf)?;
    let shdr: Elf64_Shdr = unsafe { transmute(shdr_buf) };

    // Read string table section
    let mut strtab = Vec::with_capacity(shdr.sh_size as usize);
    strtab.resize(shdr.sh_size as usize, 0);
    f.seek(SeekFrom::Start(shdr.sh_offset))?;
    f.read(&mut strtab)?;

    Ok(strtab)
}

pub fn elf64_get_section(f: &mut (impl Read + Seek), name: &[u8]) -> io::Result<Option<Elf64_Shdr>> {
    let mut ehdr_buf = [0; EHDR_SIZE];
    f.seek(SeekFrom::Start(0))?;
    f.read(&mut ehdr_buf)?;
    let ehdr: Elf64_Ehdr = unsafe { transmute(ehdr_buf) };

    let strtab = elf64_read_strtab(f, &ehdr)?;

    f.seek(SeekFrom::Start(ehdr.e_shoff))?;
    for _ in 0..ehdr.e_shnum {
        let mut shdr_buf = [0; SHDR_SIZE];
        f.read(&mut shdr_buf)?;
        let shdr: Elf64_Shdr = unsafe { transmute(shdr_buf) };

        let mut cur_name = &strtab[shdr.sh_name as usize..];
        if let Some(idx) = cur_name.iter().position(|c| *c == b'\0') {
            cur_name = &cur_name[..idx];
        }
        if cur_name == name {
            return Ok(Some(shdr));
        }
    }

    Ok(None)
}
