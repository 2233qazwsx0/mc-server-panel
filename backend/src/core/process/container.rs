use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;

use super::error::ProcessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub enabled: bool,
    pub namespace: Option<String>,
    pub cgroup_path: Option<PathBuf>,
    pub network_mode: NetworkMode,
    pub isolation_level: IsolationLevel,
    pub user: Option<ContainerUser>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Custom(String),
}

impl Default for NetworkMode {
    fn default() -> Self {
        Self::Bridge
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsolationLevel {
    Process,
    Namespace,
    Full,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::Namespace
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerUser {
    pub uid: u32,
    pub gid: u32,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            namespace: None,
            cgroup_path: None,
            network_mode: NetworkMode::default(),
            isolation_level: IsolationLevel::default(),
            user: None,
        }
    }
}

#[derive(Clone)]
pub struct ProcessContainer {
    config: ContainerConfig,
    pid: Option<u32>,
}

impl ProcessContainer {
    pub fn new(config: ContainerConfig) -> Self {
        Self {
            config,
            pid: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn get_network_mode(&self) -> NetworkMode {
        self.config.network_mode
    }

    pub fn get_isolation_level(&self) -> IsolationLevel {
        self.config.isolation_level
    }

    pub fn set_pid(&mut self, pid: u32) {
        self.pid = Some(pid);
    }

    pub fn get_pid(&self) -> Option<u32> {
        self.pid
    }

    pub fn configure_command(&self, cmd: &mut tokio::process::Command) -> Result<(), ProcessError> {
        match self.config.network_mode {
            NetworkMode::None => {
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
            }
            NetworkMode::Host => {}
            NetworkMode::Bridge => {}
            NetworkMode::Custom(_) => {}
        }

        if let Some(user) = &self.config.user {
            #[cfg(unix)]
            {
                #[allow(unused_imports)]
                use std::os::unix::process::CommandExt;
                cmd.uid(user.uid);
                cmd.gid(user.gid);
            }
        }

        Ok(())
    }

    pub fn apply_cgroup_limits(&self) -> Result<(), ProcessError> {
        if !self.config.enabled || self.config.cgroup_path.is_none() {
            return Ok(());
        }

        #[cfg(unix)]
        {
            use std::fs;
            let cgroup_path = self.config.cgroup_path.as_ref().unwrap();

            if !cgroup_path.exists() {
                fs::create_dir_all(cgroup_path)
                    .map_err(|e| ProcessError::CgroupError(e.to_string()))?;
            }

            Ok(())
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::ContainerError(
                "Cgroup not supported on this platform".to_string(),
            ))
        }
    }

    pub fn cleanup(&self) -> Result<(), ProcessError> {
        #[cfg(unix)]
        if let Some(cgroup_path) = &self.config.cgroup_path {
            if cgroup_path.exists() {
                std::fs::remove_dir(cgroup_path)
                    .map_err(|e| ProcessError::CgroupError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub fn get_config(&self) -> ContainerConfig {
        self.config.clone()
    }

    pub fn update_config(&mut self, config: ContainerConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = ProcessContainer::new(ContainerConfig::default());
        assert!(!container.is_enabled());
        assert_eq!(container.get_network_mode(), NetworkMode::Bridge);
    }

    #[test]
    fn test_container_enabled() {
        let config = ContainerConfig {
            enabled: true,
            ..Default::default()
        };
        let container = ProcessContainer::new(config);
        assert!(container.is_enabled());
    }

    #[test]
    fn test_container_pid_tracking() {
        let mut container = ProcessContainer::new(ContainerConfig::default());
        container.set_pid(12345);
        assert_eq!(container.get_pid(), Some(12345));
    }
}
