use addr2line::{Loader, Location};
use object::{Object, ObjectSymbol, Symbol};
use tracing::trace;

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

    trace!(
        "{:?} 0x{:02x} {:?} {:?} {:?}",
        symbol_name.to_string(),
        symbol.address(),
        location.file,
        location.line,
        location.column
    );
}
