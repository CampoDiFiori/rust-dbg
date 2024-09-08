use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use eyre::Result;
use nix::{
    sys::{ptrace, wait},
    unistd::Pid,
};
use object::Object;

pub fn get_base_address_from_ip(pid: Pid, obj: &object::File) -> Result<usize> {
    let regs = ptrace::getregs(pid)?;
    let entry_offset = obj.entry();

    Ok(regs.rip as usize)
}

pub fn get_base_address(pid: Pid, executable: &str) -> Option<usize> {
    let path = format!("/proc/{}/maps", pid);
    let file = std::fs::File::open(path).expect("Unable to open maps file");
    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        // Check if this is the executable mapping
        if line.contains("r-xp") && line.contains(executable) {
            let base_address = line.split('-').next().unwrap();
            return Some(usize::from_str_radix(base_address, 16).unwrap());
        }
    }
    None
}

pub fn get_base_address2(pid: Pid) -> Result<usize, Box<dyn std::error::Error>> {
    let maps_file = File::open(format!("/proc/{}/maps", pid))?;
    let reader = BufReader::new(maps_file);
    let exe_path = std::fs::read_link(format!("/proc/{}/exe", pid))?;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 6 {
            let addr_range = parts[0];
            let perms = parts[1];
            let path = PathBuf::from(parts.last().unwrap());

            // Check if this is the executable segment of our target binary
            if perms.starts_with('r') && path == exe_path {
                let base_addr = usize::from_str_radix(addr_range.split('-').next().unwrap(), 16)?;
                return Ok(base_addr);
            }
        }
    }

    Err("Base address not found in /proc/pid/maps".into())
}
