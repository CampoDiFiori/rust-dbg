use capstone::{
    arch::{x86::ArchMode, BuildsCapstone},
    Capstone,
};
use color_eyre::Result;
use eyre::WrapErr;
use nix::{
    sys::{
        ptrace::{self, AddressType},
        wait::{self, wait, WaitStatus},
    },
    unistd::{fork, ForkResult, Pid},
};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use tracing::{debug, instrument, trace};

const BREAKPOINT_INSTRUCTION: i64 = 0xCC;

#[derive(Debug)]
pub struct Debugger {
    pub object: object::File<'static>,
    pub base_address: usize,
    pub disassembler: Capstone,
    pub pid: Pid,
    pub breakpoints: HashMap<usize, i64>,
}

impl Debugger {
    pub fn new(base_address: usize, object: object::File<'static>, pid: Pid) -> Result<Self> {
        let disassembler = Capstone::new().x86().mode(ArchMode::Mode64).build()?;

        Ok(Debugger {
            base_address,
            object,
            disassembler,
            pid,
            breakpoints: HashMap::new(),
        })
    }

    #[instrument(ret, err, skip(self))]
    pub fn set_breakpoint(&mut self, address_offset: usize) -> Result<()> {
        let address = self.base_address + address_offset;

        let original_instruction = ptrace::read(self.pid, address as AddressType)? as i64;
        trace!("Read the current instruction at 0x{address:x}");

        {
            let instruction_bytes = original_instruction.to_le_bytes();
            let disassembled = self
                .disassembler
                .disasm_all(&instruction_bytes, address as u64)?;

            if let Some(instruction) = disassembled.iter().next() {
                trace!(
                    "Setting breakpoint at 0x{:x} (0x{:x}): {} {}",
                    instruction.address(),
                    address_offset,
                    instruction.mnemonic().unwrap(),
                    instruction.op_str().unwrap_or("")
                );
            } else {
                trace!("Unable to disassemble instruction at 0x{:x}", address);
            }
        }

        trace!("Set the breakpoint by writing the INT 3 instruction");
        ptrace::write(
            self.pid,
            address as AddressType,
            BREAKPOINT_INSTRUCTION as std::ffi::c_long,
        )?;

        trace!("Store the original instruction");
        self.breakpoints.insert(address, original_instruction);

        Ok(())
    }

    #[instrument(ret, err, skip(self))]
    fn handle_breakpoint(&mut self) -> Result<()> {
        let regs = ptrace::getregs(self.pid)?;
        let breakpoint_address = (regs.rip - 1) as usize;

        if let Some(&original_instruction) = self.breakpoints.get(&breakpoint_address) {
            // Restore the original instruction
            ptrace::write(
                self.pid,
                breakpoint_address as AddressType,
                original_instruction as std::ffi::c_long,
            )?;

            // Set the instruction pointer back to the start of the instruction
            let mut new_regs = regs;
            new_regs.rip -= 1;
            ptrace::setregs(self.pid, new_regs)?;

            println!("Breakpoint hit at address 0x{:x}", breakpoint_address);
        }

        Ok(())
    }

    pub fn wait_for_tracee(&self) -> Result<()> {
        wait()?;
        Ok(())
    }

    #[instrument(ret, err, skip(self))]
    pub fn run(&mut self) -> Result<()> {
        loop {
            ptrace::cont(self.pid, None)?;
            let wait_status = wait()?;
            trace!("Tracer: received wait status: {wait_status:?}");

            let regs = ptrace::getregs(self.pid)?;
            trace!("Got regs: {regs:?}");
            if self
                .breakpoints
                .keys()
                .any(|&addr| (regs.rip - 1) as usize == addr)
            {
                self.handle_breakpoint()?;
            }
        }
    }
}

#[instrument]
pub fn spawn_process(executable: &str) -> Result<Pid> {
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            trace!("Parent: Child pid is {child}");
            Ok(child)
        }
        ForkResult::Child => {
            ptrace::traceme()?;
            trace!("Child: Invoked ptrace::traceme. Executing the binary...");
            std::process::Command::new(executable).exec();
            unreachable!();
        }
    }
}
