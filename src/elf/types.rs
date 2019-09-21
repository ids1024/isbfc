#![allow(non_camel_case_types)]

use static_assertions::assert_eq_size;

type Elf64_Half = u16;
type Elf64_Word = u32;
type Elf64_Xword = u64;
type Elf64_Sxword = i64;
type Elf64_Addr = u64;
type Elf64_Off = u64;

pub const ELFMAG: [u8; 4] = *b"\x7fELF";
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const ELFOSABI_SYSV: u8 = 0;

pub const ET_EXEC: u16 = 2;
pub const EM_X86_64: u16 = 62;
pub const PT_LOAD: u32 = 1;

pub const PF_X: u32 = 1;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;

pub const EHDR_SIZE: usize = 64;
pub const PHDR_SIZE: usize = 56;
pub const SHDR_SIZE: usize = 64;

assert_eq_size!(ident_size_assert; Elf64_Ident, [u8; 16]);
assert_eq_size!(ehdr_size_assert; Elf64_Ehdr, [u8; EHDR_SIZE]);
assert_eq_size!(phdr_size_assert; Elf64_Phdr, [u8; PHDR_SIZE]);
assert_eq_size!(shdr_size_assert; Elf64_Shdr, [u8; SHDR_SIZE]);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Ident {
    pub ei_mag: [u8; 4],
    pub ei_class: u8,
    pub ei_data: u8,
    pub ei_version: u8,
    pub ei_osabi: u8,
    pub ei_abiversion: u8,
    pub ei_pad: [u8; 7],
}

// ELF header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Ehdr {
    pub e_ident: Elf64_Ident,
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
pub struct Elf64_Phdr {
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,
    pub r_info: Elf64_Xword,
    pub r_addend: Elf64_Sxword,
}
