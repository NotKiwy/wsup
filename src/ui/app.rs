use crate::core::{ProcessInfo, ProjectManager};
use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct ProcessHistory {
    pub cpu: Vec<f32>,
    pub memory: Vec<u64>,
    pub connections: Vec<u32>,
}

impl ProcessHistory {
    fn new() -> Self {
        Self {
            cpu: Vec::new(),
            memory: Vec::new(),
            connections: Vec::new(),
        }
    }

    fn push(&mut self, cpu: f32, memory: u64, connections: u32) {
        self.cpu.push(cpu);
        self.memory.push(memory);
        self.connections.push(connections);

        if self.cpu.len() > 60 {
            self.cpu.remove(0);
            self.memory.remove(0);
            self.connections.remove(0);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Detail,
    ConfirmKill,
    Projects,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Port,
    Cpu,
    Memory,
    Connections,
    Name,
}

pub struct App {
    pub processes: Vec<ProcessInfo>,
    pub filtered_processes: Vec<ProcessInfo>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub view_mode: ViewMode,
    pub sort_mode: SortMode,
    pub sort_reversed: bool,
    pub search_query: String,
    pub search_mode: bool,
    pub project_manager: ProjectManager,
    pub projects_selected: usize,
    pub process_history: HashMap<u32, ProcessHistory>,
    pub last_refresh: Instant,
    pub update_available: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let update_available = checkForUpdates();

        Self {
            processes: Vec::new(),
            filtered_processes: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            should_quit: false,
            status_message: None,
            view_mode: ViewMode::List,
            sort_mode: SortMode::Port,
            sort_reversed: false,
            search_query: String::new(),
            search_mode: false,
            project_manager: ProjectManager::new(),
            projects_selected: 0,
            process_history: HashMap::new(),
            last_refresh: Instant::now(),
            update_available,
        }
    }

    pub fn shouldAutoRefresh(&self) -> bool {
        self.last_refresh.elapsed() >= Duration::from_secs(2)
    }

    pub fn refreshProcesses(&mut self) {
        self.processes = crate::core::getLocalhostProcesses();

        for proc in &self.processes {
            let history = self.process_history.entry(proc.pid).or_insert_with(ProcessHistory::new);
            history.push(proc.cpu_usage, proc.memory, proc.connections);
        }

        self.applySort();
        self.applyFilter();
        if self.selected >= self.filtered_processes.len() && !self.filtered_processes.is_empty() {
            self.selected = self.filtered_processes.len() - 1;
        }

        self.last_refresh = Instant::now();
    }

    pub fn cycleSort(&mut self) {
        let next_mode = match self.sort_mode {
            SortMode::Port => SortMode::Cpu,
            SortMode::Cpu => SortMode::Memory,
            SortMode::Memory => SortMode::Connections,
            SortMode::Connections => SortMode::Name,
            SortMode::Name => SortMode::Port,
        };

        if next_mode == SortMode::Port && self.sort_mode == SortMode::Name {
            self.sort_reversed = !self.sort_reversed;
        }

        self.sort_mode = next_mode;
        self.refreshProcesses();
    }

    pub fn toggleSortOrder(&mut self) {
        self.sort_reversed = !self.sort_reversed;
        self.refreshProcesses();
    }

    fn applySort(&mut self) {
        match self.sort_mode {
            SortMode::Port => {
                if self.sort_reversed {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.port));
                } else {
                    self.processes.sort_by_key(|p| p.port);
                }
            }
            SortMode::Cpu => {
                self.processes.sort_by(|a, b| {
                    if self.sort_reversed {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                    }
                });
            }
            SortMode::Memory => {
                if self.sort_reversed {
                    self.processes.sort_by_key(|p| p.memory);
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.memory));
                }
            }
            SortMode::Connections => {
                if self.sort_reversed {
                    self.processes.sort_by_key(|p| p.connections);
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.connections));
                }
            }
            SortMode::Name => {
                self.processes.sort_by(|a, b| {
                    if self.sort_reversed {
                        b.name.cmp(&a.name)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
        }
    }

    fn applyFilter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_processes = self.processes.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_processes = self.processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query)
                        || p.port.to_string().contains(&query)
                        || p.commandDisplay().to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
    }

    pub fn toggleSearch(&mut self) {
        self.search_mode = !self.search_mode;
        if !self.search_mode {
            self.search_query.clear();
            self.applyFilter();
        }
    }

    pub fn searchInput(&mut self, c: char) {
        self.search_query.push(c);
        self.applyFilter();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn searchBackspace(&mut self) {
        self.search_query.pop();
        self.applyFilter();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn showProjects(&mut self) {
        self.view_mode = ViewMode::Projects;
    }

    pub fn projectsNext(&mut self) {
        let len = self.project_manager.getProjects().len();
        if len > 0 {
            self.projects_selected = (self.projects_selected + 1) % len;
        }
    }

    pub fn projectsPrevious(&mut self) {
        let len = self.project_manager.getProjects().len();
        if len > 0 {
            if self.projects_selected > 0 {
                self.projects_selected -= 1;
            } else {
                self.projects_selected = len - 1;
            }
        }
    }

    pub fn toggleProject(&mut self) {
        let project_info = self.project_manager.getProjects()
            .get(self.projects_selected)
            .map(|p| (p.id.clone(), p.name.clone()));

        if let Some((id, name)) = project_info {
            let is_running = self.project_manager.isRunning(&id);
            if is_running {
                match self.project_manager.stopProject(&id) {
                    Ok(_) => self.status_message = Some(format!("✓ Stopped {}", name)),
                    Err(e) => self.status_message = Some(format!("✗ {}", e)),
                }
            } else {
                match self.project_manager.startProject(&id) {
                    Ok(_) => self.status_message = Some(format!("✓ Started {}", name)),
                    Err(e) => self.status_message = Some(format!("✗ {}", e)),
                }
            }
        }
    }

    pub fn deployProject(&mut self) {
        let project_info = self.project_manager.getProjects()
            .get(self.projects_selected)
            .map(|p| (p.id.clone(), p.name.clone()));

        if let Some((id, name)) = project_info {
            self.status_message = Some(format!("⏳ Deploying {}...", name));
            match self.project_manager.deployProject(&id) {
                Ok(_) => self.status_message = Some(format!("✓ Deployed {}", name)),
                Err(e) => self.status_message = Some(format!("✗ Deploy failed: {}", e)),
            }
        }
    }

    pub fn next(&mut self) {
        if !self.filtered_processes.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_processes.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_processes.is_empty() {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = self.filtered_processes.len() - 1;
            }
        }
    }

    pub fn adjustScroll(&mut self, visible_rows: usize) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected - visible_rows + 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn showDetail(&mut self) {
        self.view_mode = ViewMode::Detail;
    }

    pub fn showConfirmKill(&mut self) {
        self.view_mode = ViewMode::ConfirmKill;
    }

    pub fn backToList(&mut self) {
        self.view_mode = ViewMode::List;
        self.clearStatus();
    }

    pub fn killSelected(&mut self) {
        if let Some(proc) = self.processes.get(self.selected) {
            let pid = proc.pid;
            match crate::core::killProcess(pid) {
                Ok(_) => {
                    self.status_message = Some(format!("✓ Killed {} (PID: {})", proc.name, pid));
                    self.refreshProcesses();
                    self.view_mode = ViewMode::List;
                }
                Err(e) => {
                    self.status_message = Some(format!("✗ Error: {}", e));
                    self.view_mode = ViewMode::List;
                }
            }
        }
    }

    pub fn clearStatus(&mut self) {
        self.status_message = None;
    }

    pub fn getSelectedProcess(&self) -> Option<&ProcessInfo> {
        self.filtered_processes.get(self.selected)
    }

    pub fn getHistory(&self, pid: u32) -> Option<&ProcessHistory> {
        self.process_history.get(&pid)
    }

    pub fn getStats(&self) -> AppStats {
        let total_processes = self.processes.len();
        let filtered_count = self.filtered_processes.len();
        let total_ports: std::collections::HashSet<u16> =
            self.processes.iter().map(|p| p.port).collect();
        let total_cpu: f32 = self.processes.iter().map(|p| p.cpu_usage).sum();
        let total_memory: u64 = self.processes.iter().map(|p| p.memory).sum();
        let total_connections: u32 = self.processes.iter().map(|p| p.connections).sum();

        AppStats {
            total_processes,
            filtered_count,
            unique_ports: total_ports.len(),
            total_cpu,
            total_memory,
            total_connections,
        }
    }
}

fn checkForUpdates() -> Option<String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CrateResponse {
        #[serde(rename = "crate")]
        krate: CrateInfo,
    }

    #[derive(Deserialize)]
    struct CrateInfo {
        max_version: String,
    }

    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    const TIMEOUT_SECS: u64 = 2;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .ok()?;

    let response = client
        .get("https://crates.io/api/v1/crates/wsup")
        .header("User-Agent", format!("wsup/{}", CURRENT_VERSION))
        .send()
        .ok()?;

    let crate_info: CrateResponse = response.json().ok()?;
    let latest_version = crate_info.krate.max_version;

    if isNewer(&latest_version, CURRENT_VERSION) {
        Some(latest_version)
    } else {
        None
    }
}

fn isNewer(remote: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    if let (Some(remote_ver), Some(current_ver)) = (parse_version(remote), parse_version(current)) {
        remote_ver > current_ver
    } else {
        false
    }
}

pub struct AppStats {
    pub total_processes: usize,
    pub filtered_count: usize,
    pub unique_ports: usize,
    pub total_cpu: f32,
    pub total_memory: u64,
    pub total_connections: u32,
}
