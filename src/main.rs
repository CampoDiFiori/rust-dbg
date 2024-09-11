mod debugger;
mod error;
mod files;
mod symbols;
mod utils;

use color_eyre::Result;
use debugger::{spawn_process, Debugger};
use error::AppError;
use files::ProjectFileCache;
use symbols::find_main_symbol_address;
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

fn main_inner() -> Result<()> {
    let executable = "target/debug/examples/test";
    let file_cache = ProjectFileCache::new("/home/pdudko/cool".as_ref())?;

    let loader = addr2line::Loader::new(executable).unwrap();
    let buf = std::fs::read(executable)?.into_boxed_slice();
    let buf: &'static mut [u8] = Box::leak(buf);
    let object: object::File<'static> = object::File::parse(&*buf)?;

    // dump_file_symbols(executable.as_ref(), &file_cache).unwrap();
    let main_addr = find_main_symbol_address(&loader, &object, &file_cache).unwrap();
    let pid = spawn_process(executable)?;

    trace!("{:?}", nix::sys::wait::wait()?);

    let base_address = utils::get_base_address_from_procfs(pid).unwrap();

    let mut debugger = Debugger::new(executable, base_address, object, pid)?;

    trace!("Tracee's base address is  0x{base_address:02x}");

    debugger.set_breakpoint(main_addr)?;
    debugger.run()?;

    Ok(())
}

fn main() {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize color-eyre for error reporting
    color_eyre::install().unwrap();

    let writer = std::fs::File::create("cool.log").unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
        )
        .init();

    main_inner().unwrap();
}
