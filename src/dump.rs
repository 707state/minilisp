use object::write::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub fn write_executable_aarch64(instructions: &[u32], obj_path: &str, exe_path: &str) {
    #[cfg(target_os = "macos")]
    write_macho(obj_path, instructions);

    #[cfg(target_os = "linux")]
    write_elf(obj_path, instructions);

    // 调用系统 linker
    #[cfg(target_os = "macos")]
    let ld_status = Command::new("ld")
        .args([
            "-arch",
            "arm64",
            obj_path,
            "-o",
            exe_path,
            "-L",
            "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib/",
            "-lSystem",
            "-platform_version",
            "macos",
            "14.0.0",
            "14.0.0",
        ])
        .status()
        .expect("Failed to execute ld");

    #[cfg(target_os = "linux")]
    let ld_status = Command::new("ld")
        .args([
            "-m",
            "aarch64elf",
            obj_path,
            "-o",
            exe_path,
            "-L",
            "/lib/aarch64-linux-gnu/",
            "-lc", // 链接 libc
            "--dynamic-linker",
            "/lib/ld-linux-aarch64.so.1",
        ])
        .status()
        .expect("Failed to execute ld");

    if ld_status.success() {
        println!("Successfully linked {} executable.", exe_path);
    } else {
        panic!("Linking failed with status: {:?}", ld_status.code());
    }
}

#[cfg(target_os = "macos")]
fn write_macho(path: &str, instructions: &[u32]) {
    let mut obj = Object::new(
        BinaryFormat::MachO,
        Architecture::Aarch64,
        object::Endianness::Little,
    );

    let mut text_bytes = Vec::new();
    for inst in instructions {
        text_bytes.extend_from_slice(&inst.to_le_bytes());
    }

    let section_id = obj.add_section(b"__TEXT".to_vec(), b"__text".to_vec(), SectionKind::Text);
    obj.append_section_data(section_id, &text_bytes, 4);

    obj.add_symbol(Symbol {
        name: b"main".to_vec(), // Mach-O entry symbol
        value: 0,
        size: text_bytes.len() as u64,
        kind: SymbolKind::Text,
        section: SymbolSection::Section(section_id),
        scope: SymbolScope::Linkage,
        weak: false,
        flags: SymbolFlags::None,
    });

    let bytes = obj.write().unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&bytes).unwrap();
}

#[cfg(target_os = "linux")]
fn write_elf(path: &str, instructions: &[u32]) {
    let mut obj = Object::new(
        BinaryFormat::Elf,
        Architecture::Aarch64,
        object::Endianness::Little,
    );

    let mut text_bytes = Vec::new();
    for inst in instructions {
        text_bytes.extend_from_slice(&inst.to_le_bytes());
    }

    let section_id = obj.add_section(b".text".to_vec(), b".text".to_vec(), SectionKind::Text);
    obj.append_section_data(section_id, &text_bytes, 4);

    let main_symbol = obj.add_symbol(Symbol {
        name: b"_start".to_vec(), // ELF entry symbol
        value: 0,
        size: text_bytes.len() as u64,
        kind: SymbolKind::Text,
        section: SymbolSection::Section(section_id),
        scope: SymbolScope::Linkage,
        weak: false,
        flags: SymbolFlags::None,
    });

    // 设置 ELF entry

    let bytes = obj.write().unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&bytes).unwrap();
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use object::{
        LittleEndian,
        macho::{self, N_EXT},
        read::macho::{MachHeader, Nlist},
    };
    #[cfg(target_os = "macos")]
    #[test]
    fn test_object_read() -> Result<(), Box<dyn Error>> {
        let data = fs::read("/Users/jask/test/learn_assembly/dump.o")?;
        let header = macho::MachHeader64::<LittleEndian>::parse(&*data, 0)?;
        let endian = header.endian()?;
        let mut commands = header.load_commands(endian, &*data, 0)?;
        while let Some(command) = commands.next()? {
            if let Some(symtab_command) = command.symtab()? {
                let symbols =
                    symtab_command.symbols::<macho::MachHeader64<_>, _>(endian, &*data)?;
                for symbol in symbols.iter() {
                    if symbol.n_type & N_EXT != 0 {
                        let name = symbol.name(endian, symbols.strings())?;
                        println!("{}", String::from_utf8_lossy(name));
                    }
                }
            }
        }
        Ok(())
    }
}
