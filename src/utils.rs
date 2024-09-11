use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use eyre::Result;
use nix::unistd::Pid;

pub fn get_base_address_from_procfs(pid: Pid) -> Result<usize, Box<dyn std::error::Error>> {
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
