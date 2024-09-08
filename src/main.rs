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

    trace!("{:?}", nix::sys::wait::wait().unwrap());

    let base_address0 = utils::get_base_address_from_ip(pid, &object)?;
    let base_address = utils::get_base_address(pid, executable).unwrap();
    let base_address2 = utils::get_base_address2(pid).unwrap();

    trace!("0: 0x{base_address0:x}, 1: 0x{base_address:x}, 2: 0x{base_address2:x}");

    let mut debugger = Debugger::new(base_address2, object, pid)?;

    trace!("Tracee's base address is  0x{base_address:02x}");

    debugger.set_breakpoint(main_addr)?;
    debugger.wait_for_tracee()?;
    // std::thread::sleep(std::time::Duration::from_secs();
    // debugger.run()?;

    Ok(())
}

fn main() {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize color-eyre for error reporting
    color_eyre::install().unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
        )
        .init();

    main_inner().unwrap();
}
