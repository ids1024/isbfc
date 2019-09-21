use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;

use crate::elf::{elf64_get_section, elf64_write};

fn object_to_binary(mut o_file: File) -> io::Result<(Vec<u8>, u64)> {
    let text = elf64_get_section(&mut o_file, b".text")?
        .unwrap();
    let bss = elf64_get_section(&mut o_file, b".bss")?
        .unwrap();
    let bss_offset = (text.sh_size + 0x1000 - 1) & !(0x1000 - 1);
    let bss_size = bss.sh_size;

    let bin = Command::new("ld")
        .arg("--oformat")
        .arg("binary")
        .arg("-Ttext")
        .arg("0x40_1000")
        .arg("-Tbss")
        .arg(format!("0x{:x}", 0x40_1000 + bss_offset))
        .arg("-o")
        .arg("/dev/stdout")
        .arg("/dev/stdin")
        .stdin(o_file)
        .output()?
        .stdout;

    Ok((bin, bss_size))
}

pub fn assemble(code: &str, out_name: &str, debug: bool) -> io::Result<Option<i32>> {
    let mut command = Command::new("as");
    if debug {
        command.arg("-g");
    }
    let mut child = command
        .arg("-o")
        .arg(out_name)
        .arg("-") // Standard input
        .stdin(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(code.as_bytes())?;

    Ok(child.wait()?.code())
}

pub fn link(o_name: &str, out_name: &str, minimal: bool) -> io::Result<Option<i32>> {
    let o_file = File::open(o_name)?;
    if minimal {
        let (bin, bss_size) = object_to_binary(o_file)?;

        let mut file = File::create(out_name)?;
        elf64_write(&mut file, &bin, bss_size)?;
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        file.set_permissions(permissions)?;
        Ok(Some(0))
    } else {
        Ok(Command::new("ld")
            .arg("-o")
            .arg(out_name)
            .arg("/dev/stdin")
            .stdin(o_file)
            .spawn()?
            .wait()?
            .code())
    }
}
