mod debugger;
mod error;
mod source_files;
mod symbols;
mod tui;
mod utils;

use color_eyre::Result;
use debugger::{spawn_process, Debugger};
use error::AppError;
use source_files::SourceFiles;
use symbols::find_main_symbol_address;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tui::run_tui;

fn run_debugger(exe_path: &str, source_files: SourceFiles) -> Result<()> {
    let a2l_loader = addr2line::Loader::new(exe_path).unwrap();
    let exe_text = std::fs::read(exe_path)?.into_boxed_slice();
    let exe_text: &'static mut [u8] = Box::leak(exe_text);
    let obj_file: object::File<'static> = object::File::parse(&*exe_text)?;

    // dump_file_symbols(executable.as_ref(), &file_cache).unwrap();
    let tracee_main_addr = find_main_symbol_address(&a2l_loader, &obj_file, &source_files).unwrap();
    let tracee_pid = spawn_process(exe_path)?;

    debug!("{:?}", nix::sys::wait::wait()?);

    let tracee_base_addr = utils::get_base_address_from_procfs(tracee_pid).unwrap();

    let mut debugger = Debugger::new(exe_path, tracee_base_addr, obj_file, tracee_pid)?;

    debug!("Tracee's base address is  0x{tracee_base_addr:02x}");

    debugger.set_breakpoint(tracee_main_addr)?;
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

    let project_dir = "/home/pdudko/cool/examples";
    let exe_path = "/home/pdudko/cool/target/debug/examples/test";

    let source_files = SourceFiles::new(project_dir.as_ref()).unwrap();
    run_debugger(exe_path, source_files).unwrap();

    // let tui_thread = std::thread::spawn(|| run_tui(source_files));
    // tui_thread.join().unwrap().unwrap();
}
