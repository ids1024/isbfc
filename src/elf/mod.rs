use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::transmute;

mod types;
use types::*;

// Minimal ELF support, sufficient for a very simple 64-bit static Linux
// executable.

// Sources:
// * /usr/include/elf.h
// * https://wiki.osdev.org/ELF_Tutorial
// * https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// * https://www.muppetlabs.com/~breadbox/software/tiny/teensy.html
// * https://unix.stackexchange.com/questions/132036/why-does-readelf-show-system-v-as-my-os-instead-of-linux
// * http://www.sco.com/developers/gabi/latest/ch4.eheader.html

pub fn elf64_write(f: &mut impl Write, text: &[u8], bss_size: u64) -> io::Result<()> {
    let size = text.len() as u64;
    let hdr_size = (EHDR_SIZE + 2 * PHDR_SIZE) as u64;
    let hdr_size_padded = (hdr_size + 0x1000 - 1) & !(0x1000 - 1);

    let ehdr = Elf64_Ehdr {
        e_ident: Elf64_Ident {
            ei_mag: ELFMAG,
            ei_class: ELFCLASS64,
            ei_data: ELFDATA2LSB,
            ei_version: 1,
            ei_osabi: ELFOSABI_SYSV,
            ei_abiversion: 0,
            ei_pad: [0; 7]
        },
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
    f.seek(SeekFrom::Start(
        ehdr.e_shoff + ehdr.e_shstrndx as u64 * SHDR_SIZE as u64,
    ))?;
    f.read(&mut shdr_buf)?;
    let shdr: Elf64_Shdr = unsafe { transmute(shdr_buf) };

    // Read string table section
    let mut strtab = Vec::with_capacity(shdr.sh_size as usize);
    strtab.resize(shdr.sh_size as usize, 0);
    f.seek(SeekFrom::Start(shdr.sh_offset))?;
    f.read(&mut strtab)?;

    Ok(strtab)
}

pub fn elf64_get_section(
    f: &mut (impl Read + Seek),
    name: &[u8],
) -> io::Result<Option<Elf64_Shdr>> {
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
