use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub port: u16,
    pub build_command: Option<String>,
    pub run_command: String,
    pub deploy_command: Option<String>,
    pub env_vars: HashMap<String, String>,
}

impl Project {
    pub fn new(name: String, path: String, port: u16, run_command: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            name,
            path,
            port,
            build_command: None,
            run_command,
            deploy_command: None,
            env_vars: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ProjectManager {
    projects: Vec<Project>,
    running: HashMap<String, Child>,
    config_path: PathBuf,
}

impl ProjectManager {
    pub fn new() -> Self {
        let config_path = Self::getConfigPath();
        let projects = Self::loadProjects(&config_path);

        Self {
            projects,
            running: HashMap::new(),
            config_path,
        }
    }

    fn getConfigPath() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("wsup");
        fs::create_dir_all(&path).ok();
        path.push("projects.json");
        path
    }

    fn loadProjects(path: &PathBuf) -> Vec<Project> {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn saveProjects(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.projects)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }

    pub fn getProjects(&self) -> &Vec<Project> {
        &self.projects
    }

    pub fn addProject(&mut self, project: Project) {
        self.projects.push(project);
        self.saveProjects().ok();
    }

    pub fn removeProject(&mut self, id: &str) {
        self.projects.retain(|p| p.id != id);
        self.saveProjects().ok();
    }

    pub fn startProject(&mut self, id: &str) -> Result<(), String> {
        let project = self.projects.iter()
            .find(|p| p.id == id)
            .ok_or("Project not found")?;

        if let Some(build_cmd) = &project.build_command {
            let build_result = Command::new("sh")
                .arg("-c")
                .arg(build_cmd)
                .current_dir(&project.path)
                .status()
                .map_err(|e| format!("Build failed: {}", e))?;

            if !build_result.success() {
                return Err("Build command failed".to_string());
            }
        }

        let child = Command::new("sh")
            .arg("-c")
            .arg(&project.run_command)
            .current_dir(&project.path)
            .envs(&project.env_vars)
            .spawn()
            .map_err(|e| format!("Failed to start: {}", e))?;

        self.running.insert(id.to_string(), child);
        Ok(())
    }

    pub fn stopProject(&mut self, id: &str) -> Result<(), String> {
        if let Some(mut child) = self.running.remove(id) {
            child.kill().map_err(|e| format!("Failed to kill: {}", e))?;
            Ok(())
        } else {
            Err("Project not running".to_string())
        }
    }

    pub fn isRunning(&self, id: &str) -> bool {
        self.running.contains_key(id)
    }

    pub fn deployProject(&self, id: &str) -> Result<(), String> {
        let project = self.projects.iter()
            .find(|p| p.id == id)
            .ok_or("Project not found")?;

        if let Some(deploy_cmd) = &project.deploy_command {
            if let Some(build_cmd) = &project.build_command {
                let build_result = Command::new("sh")
                    .arg("-c")
                    .arg(build_cmd)
                    .current_dir(&project.path)
                    .status()
                    .map_err(|e| format!("Build failed: {}", e))?;

                if !build_result.success() {
                    return Err("Build command failed".to_string());
                }
            }

            let deploy_result = Command::new("sh")
                .arg("-c")
                .arg(deploy_cmd)
                .current_dir(&project.path)
                .envs(&project.env_vars)
                .status()
                .map_err(|e| format!("Deploy failed: {}", e))?;

            if deploy_result.success() {
                Ok(())
            } else {
                Err("Deploy command failed".to_string())
            }
        } else {
            Err("No deploy command configured".to_string())
        }
    }
}
