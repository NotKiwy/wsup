use sysinfo::{System, Pid};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub port: u16,
    pub cpu_usage: f32,
    pub memory: u64,
    pub connections: u32,
    pub protocol: String,
}

impl ProcessInfo {
    pub fn commandDisplay(&self) -> String {
        if self.cmd.is_empty() {
            self.name.clone()
        } else {
            self.cmd.join(" ")
        }
    }
}

pub fn getLocalhostProcesses() -> Vec<ProcessInfo> {
    let port_map = getPortMappings();
    let cpu_map = getCpuUsage();

    let mut system = System::new();
    system.refresh_all();

    let mut processes = Vec::new();

    for ((pid, port), (connections, protocol)) in port_map {
        if let Some(process) = system.process(Pid::from_u32(pid)) {
            let name = process.name().to_string_lossy().to_string();
            let cmd: Vec<String> = process.cmd().iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            let cpu_usage = cpu_map.get(&pid).copied().unwrap_or(0.0);

            processes.push(ProcessInfo {
                pid,
                name: name.clone(),
                cmd: cmd.clone(),
                port,
                cpu_usage,
                memory: process.memory(),
                connections,
                protocol,
            });
        }
    }

    processes.sort_by_key(|p| p.port);
    processes
}

fn getPortMappings() -> HashMap<(u32, u16), (u32, String)> {
    let mut port_map: HashMap<(u32, u16), (u32, String)> = HashMap::new();
    let mut connection_counts: HashMap<(u32, u16), u32> = HashMap::new();

    let output = Command::new("lsof")
        .args(&["-iTCP", "-n", "-P"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    if let Some(name_port) = parts.get(8) {
                        if let Some(port_str) = name_port.split(':').last() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                let proto = parts.get(7).unwrap_or(&"TCP").to_string();
                                let state = parts.get(9).unwrap_or(&"");

                                if state.contains("LISTEN") {
                                    port_map.insert((pid, port), (0, proto.clone()));
                                }

                                if state.contains("ESTABLISHED") {
                                    *connection_counts.entry((pid, port)).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (key, count) in connection_counts {
        if let Some((conn, _)) = port_map.get_mut(&key) {
            *conn = count;
        }
    }

    port_map
}

fn getCpuUsage() -> HashMap<u32, f32> {
    let mut cpu_map = HashMap::new();

    let output = Command::new("ps")
        .args(&["-eo", "pid,%cpu"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(pid) = parts[0].parse::<u32>() {
                    if let Ok(cpu) = parts[1].parse::<f32>() {
                        cpu_map.insert(pid, cpu);
                    }
                }
            }
        }
    }

    cpu_map
}

pub fn killProcess(pid: u32) -> Result<(), String> {
    let output = Command::new("kill")
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to kill process: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Failed to kill process {}", pid))
    }
}
