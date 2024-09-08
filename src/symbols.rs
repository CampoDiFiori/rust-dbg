use std::{ops::Range, path::Path};

use addr2line::{fallible_iterator::FallibleIterator, Loader, Location};
use object::{Object, ObjectSection, ObjectSymbol, SectionKind, Symbol};

use crate::{files::ProjectFileCache, AppError};

pub fn find_main_symbol_address(
    loader: &Loader,
    object: &object::File,
    file_cache: &ProjectFileCache,
) -> Result<usize, AppError> {
    if !object.has_debug_symbols() {
        return Err(AppError::NoDebugSymbols);
    }

    for symbol in object.symbols() {
        let location = loader.find_location(symbol.address())?;

        let Some(location) = location else {
            continue;
        };

        if location
            .file
            .map(|f| !file_cache.contains(f.as_ref()))
            .unwrap_or(true)
        {
            continue;
        }

        let Ok(symbol_name) = symbol.name().map(rustc_demangle::demangle) else {
            continue;
        };

        print_symbol_location(Some(location), &symbol, file_cache);

        if symbol_name.to_string().contains("::main::") {
            return Ok(symbol.address() as usize);
        }
    }

    Err(AppError::NoMainFunction)
}

pub fn dump_file_symbols(executable: &Path, file_cache: &ProjectFileCache) -> Result<(), AppError> {
    let buf = std::fs::read(executable)?;
    let file = object::File::parse(&*buf)?;

    if !file.has_debug_symbols() {
        return Err(AppError::NoDebugSymbols);
    }

    let loader = addr2line::Loader::new(executable)?;

    for symbol in file.symbols() {
        let location = loader.find_location(symbol.address())?;
        print_symbol_location(location, &symbol, file_cache);
    }

    // Collect executable sections
    let executable_sections: Vec<Range<u64>> = file
        .sections()
        .filter_map(|section| {
            if section.kind() == SectionKind::Text {
                Some(section.address()..section.address() + section.size())
            } else {
                None
            }
        })
        .collect();

    // Iterate through all addresses in executable sections
    for section_range in executable_sections {
        for address in section_range.step_by(1) {
            let frames = loader.find_frames(address)?;

            for frame in frames.iterator() {
                print_location(address, frame?.location, file_cache);
            }
        }
    }

    Ok(())
}

pub fn print_symbol_location(
    location: Option<Location<'_>>,
    symbol: &Symbol,
    file_cache: &ProjectFileCache,
) {
    let Some(location) = location else {
        return;
    };

    let Ok(symbol_name) = symbol.name().map(rustc_demangle::demangle) else {
        return;
    };

    if location
        .file
        .map(|f| !file_cache.contains(f.as_ref()))
        .unwrap_or(true)
    {
        return;
    }

    println!(
        "{:?} 0x{:02x} {:?} {:?} {:?}",
        symbol_name.to_string(),
        symbol.address(),
        location.file,
        location.line,
        location.column
    );
}

pub fn print_location(address: u64, location: Option<Location<'_>>, file_cache: &ProjectFileCache) {
    let Some(location) = location else {
        return;
    };

    if location
        .file
        .map(|f| !file_cache.contains(f.as_ref()))
        .unwrap_or(true)
    {
        return;
    }

    println!(
        "0x{:016x} {:?} {:?} {:?}",
        address, location.file, location.line, location.column
    );
}
