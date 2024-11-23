use capstone::{
    arch::{x86::ArchMode, BuildsCapstone},
    Capstone,
};
use color_eyre::Result;
use nix::{
    sys::{
        ptrace::{self, AddressType},
        wait::{wait, WaitStatus},
    },
    unistd::{fork, ForkResult, Pid},
};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use tracing::{debug, instrument};

const BREAKPOINT_INSTRUCTION: i64 = 0xCC;

pub type Address = usize;

pub struct Debugger {
    #[allow(dead_code)]
    pub object: object::File<'static>,
    pub base_address: usize,
    pub disassembler: Capstone,
    pub pid: Pid,
    pub a2l_loader: addr2line::Loader,
    pub breakpoints: HashMap<Address, BreakPoint>,
}

pub struct BreakPoint {
    pub original_instruction: i64,
    pub relative_addr: Address,
}

impl Debugger {
    pub fn new(
        executable: &str,
        base_address: usize,
        object: object::File<'static>,
        pid: Pid,
    ) -> Result<Self> {
        let disassembler = Capstone::new().x86().mode(ArchMode::Mode64).build()?;
        let loader = addr2line::Loader::new(executable).unwrap();

        Ok(Debugger {
            base_address,
            object,
            disassembler,
            pid,
            a2l_loader: loader,
            breakpoints: HashMap::new(),
        })
    }

    #[instrument(ret, err, skip(self))]
    pub fn set_breakpoint(&mut self, relative_addr: usize) -> Result<()> {
        let address = self.base_address + relative_addr;

        let original_instruction = ptrace::read(self.pid, address as AddressType)? as i64;
        debug!("Read the current instruction at 0x{address:x}");

        {
            let instruction_bytes = original_instruction.to_le_bytes();
            let disassembled = self
                .disassembler
                .disasm_all(&instruction_bytes, address as u64)?;

            if let Some(instruction) = disassembled.iter().next() {
                debug!(
                    "Setting breakpoint at 0x{:x} (0x{:x}): {} {}",
                    instruction.address(),
                    relative_addr,
                    instruction.mnemonic().unwrap(),
                    instruction.op_str().unwrap_or("")
                );
            } else {
                debug!("Unable to disassemble instruction at 0x{:x}", address);
            }
        }

        debug!("Set the breakpoint by writing the INT 3 instruction");
        ptrace::write(
            self.pid,
            address as AddressType,
            BREAKPOINT_INSTRUCTION as std::ffi::c_long,
        )?;

        debug!("Store the original instruction");
        self.breakpoints.insert(
            address,
            BreakPoint {
                original_instruction,
                relative_addr,
            },
        );

        Ok(())
    }

    #[instrument(ret, err, skip(self))]
    fn handle_breakpoint(&self) -> Result<()> {
        let regs = ptrace::getregs(self.pid)?;
        let breakpoint_address = (regs.rip - 1) as usize;

        let Some(bp) = self.breakpoints.get(&breakpoint_address) else {
            eyre::bail!("No breakpoint found for address {breakpoint_address}");
        };

        {
            // Restore the original instruction
            ptrace::write(
                self.pid,
                breakpoint_address as AddressType,
                bp.original_instruction as std::ffi::c_long,
            )?;

            // Set the instruction pointer back to the start of the instruction
            let mut new_regs = regs;
            new_regs.rip -= 1;
            ptrace::setregs(self.pid, new_regs)?;

            debug!("Breakpoint hit at address 0x{:x}", breakpoint_address);
        }

        {
            let Ok(Some(location)) = self.a2l_loader.find_location(bp.relative_addr as _) else {
                debug!("No location in source found for address 0x{breakpoint_address:2x}");
                return Ok(());
            };

            let Some((file_name, line_nr)) = location.file.zip(location.line) else {
                debug!("No file or line found for address 0x{breakpoint_address:2x}");
                return Ok(());
            };

            let file = std::fs::read_to_string(file_name)?;
            let line = file.lines().nth(line_nr as usize - 1).unwrap();

            debug!("{file_name}:{line_nr}: {line}");
        }

        Ok(())
    }

    #[instrument(ret, err, skip(self))]
    pub fn run(&mut self) -> Result<()> {
        loop {
            ptrace::cont(self.pid, None)?;
            let wait_status = wait()?;
            debug!("Tracer: received wait status: {wait_status:?}");

            if let WaitStatus::Exited(_, _) = wait_status {
                break;
            }

            let regs = ptrace::getregs(self.pid)?;
            debug!("Got regs: {regs:?}");
            if self
                .breakpoints
                .keys()
                .any(|&addr| (regs.rip - 1) as usize == addr)
            {
                self.handle_breakpoint()?;
            }
        }

        Ok(())
    }
}

#[instrument]
pub fn spawn_process(executable: &str) -> Result<Pid> {
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            debug!("Parent: Child pid is {child}");
            Ok(child)
        }
        ForkResult::Child => {
            ptrace::traceme()?;
            debug!("Child: Invoked ptrace::traceme. Executing the binary...");
            std::process::Command::new(executable).exec();
            unreachable!();
        }
    }
}
