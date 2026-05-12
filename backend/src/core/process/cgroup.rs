use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use super::error::ProcessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgroupConfig {
    pub enabled: bool,
    pub version: CgroupVersion,
    pub hierarchy_path: PathBuf,
    pub controllers: Vec<CgroupController>,
    pub limits: CgroupLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CgroupVersion {
    V1,
    V2,
}

impl Default for CgroupVersion {
    fn default() -> Self {
        Self::V2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgroupController {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgroupLimits {
    pub memory: MemoryLimit,
    pub cpu: CpuLimit,
    pub io: IoLimit,
    pub pids: PidsLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimit {
    pub enabled: bool,
    pub max_mb: u64,
    pub soft_limit_mb: Option<u64>,
    pub swap_max_mb: Option<u64>,
}

impl Default for MemoryLimit {
    fn default() -> Self {
        Self {
            enabled: true,
            max_mb: 4096,
            soft_limit_mb: None,
            swap_max_mb: Some(1024),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLimit {
    pub enabled: bool,
    pub max_cpus: u32,
    pub max_weight: Option<u64>,
    pub quota_us: Option<i64>,
    pub period_us: Option<u64>,
}

impl Default for CpuLimit {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cpus: 2,
            max_weight: None,
            quota_us: None,
            period_us: Some(100000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoLimit {
    pub enabled: bool,
    pub read_bps: Option<u64>,
    pub write_bps: Option<u64>,
    pub read_iops: Option<u64>,
    pub write_iops: Option<u64>,
}

impl Default for IoLimit {
    fn default() -> Self {
        Self {
            enabled: false,
            read_bps: None,
            write_bps: None,
            read_iops: None,
            write_iops: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsLimit {
    pub enabled: bool,
    pub max_pids: u64,
}

impl Default for PidsLimit {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pids: 4096,
        }
    }
}

impl Default for CgroupLimits {
    fn default() -> Self {
        Self {
            memory: MemoryLimit::default(),
            cpu: CpuLimit::default(),
            io: IoLimit::default(),
            pids: PidsLimit::default(),
        }
    }
}

impl Default for CgroupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            version: CgroupVersion::default(),
            hierarchy_path: PathBuf::from("/sys/fs/cgroup"),
            controllers: vec![
                CgroupController {
                    name: "memory".to_string(),
                    enabled: true,
                },
                CgroupController {
                    name: "cpu".to_string(),
                    enabled: true,
                },
                CgroupController {
                    name: "io".to_string(),
                    enabled: false,
                },
                CgroupController {
                    name: "pids".to_string(),
                    enabled: true,
                },
            ],
            limits: CgroupLimits::default(),
        }
    }
}

pub struct CgroupManager {
    config: CgroupConfig,
    instance_path: Option<PathBuf>,
}

impl CgroupManager {
    pub fn new(config: CgroupConfig) -> Self {
        Self {
            config,
            instance_path: None,
        }
    }

    pub fn is_available() -> bool {
        #[cfg(unix)]
        {
            std::path::Path::new("/sys/fs/cgroup").exists()
        }

        #[cfg(not(unix))]
        {
            false
        }
    }

    pub fn get_version() -> Option<CgroupVersion> {
        #[cfg(unix)]
        {
            if std::path::Path::new("/sys/fs/cgroup/unified").exists() {
                Some(CgroupVersion::V2)
            } else if std::path::Path::new("/sys/fs/cgroup/memory").exists() {
                Some(CgroupVersion::V1)
            } else {
                None
            }
        }

        #[cfg(not(unix))]
        {
            None
        }
    }

    pub fn create_cgroup(&mut self, instance_name: &str) -> Result<PathBuf, ProcessError> {
        if !self.config.enabled {
            return Err(ProcessError::CgroupError(
                "Cgroup support is disabled".to_string(),
            ));
        }

        if !Self::is_available() {
            return Err(ProcessError::CgroupError(
                "Cgroup is not available on this system".to_string(),
            ));
        }

        let hierarchy_path = match self.config.version {
            CgroupVersion::V1 => self.config.hierarchy_path.join("memory"),
            CgroupVersion::V2 => self.config.hierarchy_path.join("system.slice"),
        };

        let instance_path = hierarchy_path.join(format!("minecraft-{}.slice", instance_name));

        #[cfg(unix)]
        {
            use std::fs;

            fs::create_dir_all(&instance_path)
                .map_err(|e| ProcessError::CgroupError(format!("Failed to create cgroup: {}", e)))?;

            self.apply_limits(&instance_path)?;

            self.instance_path = Some(instance_path.clone());
            info!("Created cgroup at: {}", instance_path.display());

            Ok(instance_path)
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::CgroupError(
                "Cgroup is only supported on Unix systems".to_string(),
            ))
        }
    }

    fn apply_limits(&self, path: &PathBuf) -> Result<(), ProcessError> {
        #[cfg(unix)]
        {
            use std::fs;

            if self.config.limits.memory.enabled {
                let max_memory = format!("{}M", self.config.limits.memory.max_mb);
                let memory_path = path.join("memory.max");
                fs::write(&memory_path, &max_memory)
                    .map_err(|e| ProcessError::CgroupError(format!("Failed to set memory limit: {}", e)))?;

                if let Some(swap_max) = self.config.limits.memory.swap_max_mb {
                    let swap_path = path.join("memory.swap.max");
                    fs::write(&swap_path, format!("{}M", swap_max))
                        .map_err(|e| ProcessError::CgroupError(format!("Failed to set swap limit: {}", e)))?;
                }
            }

            if self.config.limits.cpu.enabled {
                let cpu_max = self.config.limits.cpu.max_cpus.to_string();
                let cpu_path = path.join("cpu.max");
                let period = self.config.limits.cpu.period_us.unwrap_or(100000);
                fs::write(&cpu_path, format!("max {} 100000", period))
                    .map_err(|e| ProcessError::CgroupError(format!("Failed to set CPU limit: {}", e)))?;
            }

            if self.config.limits.pids.enabled {
                let pids_path = path.join("pids.max");
                fs::write(&pids_path, self.config.limits.pids.max_pids.to_string())
                    .map_err(|e| ProcessError::CgroupError(format!("Failed to set PIDs limit: {}", e)))?;
            }

            Ok(())
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::CgroupError(
                "Cgroup limits can only be applied on Unix systems".to_string(),
            ))
        }
    }

    pub fn add_process(&self, pid: u32) -> Result<(), ProcessError> {
        let path = self.instance_path.as_ref()
            .ok_or_else(|| ProcessError::CgroupError("No cgroup created yet".to_string()))?;

        #[cfg(unix)]
        {
            use std::fs;

            let tasks_path = path.join("cgroup.procs");
            fs::write(&tasks_path, pid.to_string())
                .map_err(|e| ProcessError::CgroupError(format!("Failed to add process to cgroup: {}", e)))?;

            info!("Added process {} to cgroup {}", pid, path.display());
            Ok(())
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::CgroupError(
                "Cgroup is only supported on Unix systems".to_string(),
            ))
        }
    }

    pub fn remove_cgroup(&self) -> Result<(), ProcessError> {
        let path = self.instance_path.as_ref()
            .ok_or_else(|| ProcessError::CgroupError("No cgroup created yet".to_string()))?;

        #[cfg(unix)]
        {
            use std::fs;

            if path.exists() {
                fs::remove_dir(path)
                    .map_err(|e| ProcessError::CgroupError(format!("Failed to remove cgroup: {}", e)))?;
                info!("Removed cgroup at: {}", path.display());
            }

            Ok(())
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::CgroupError(
                "Cgroup is only supported on Unix systems".to_string(),
            ))
        }
    }

    pub fn get_usage(&self) -> Result<CgroupUsage, ProcessError> {
        let path = self.instance_path.as_ref()
            .ok_or_else(|| ProcessError::CgroupError("No cgroup created yet".to_string()))?;

        #[cfg(unix)]
        {
            use std::fs;

            let mut usage = CgroupUsage::default();

            if self.config.limits.memory.enabled {
                let memory_current = path.join("memory.current");
                if let Ok(content) = fs::read_to_string(&memory_current) {
                    if let Ok(val) = content.trim().parse::<u64>() {
                        usage.memory_usage_bytes = val;
                    }
                }
            }

            if self.config.limits.cpu.enabled {
                let cpu_stat = path.join("cpu.stat");
                if let Ok(content) = fs::read_to_string(&cpu_stat) {
                    for line in content.lines() {
                        if line.starts_with("usage_usec") {
                            if let Some(val) = line.split_whitespace().nth(1) {
                                if let Ok(usec) = val.parse::<u64>() {
                                    usage.cpu_usage_usec = usec;
                                }
                            }
                        }
                    }
                }
            }

            let pids_current = path.join("pids.current");
            if let Ok(content) = fs::read_to_string(&pids_current) {
                if let Ok(val) = content.trim().parse::<u64>() {
                    usage.pids_current = val;
                }
            }

            Ok(usage)
        }

        #[cfg(not(unix))]
        {
            Err(ProcessError::CgroupError(
                "Cgroup usage can only be read on Unix systems".to_string(),
            ))
        }
    }

    pub fn update_limits(&mut self, limits: CgroupLimits) -> Result<(), ProcessError> {
        self.config.limits = limits;

        if let Some(path) = &self.instance_path {
            self.apply_limits(path)?;
        }

        Ok(())
    }

    pub fn get_config(&self) -> CgroupConfig {
        self.config.clone()
    }

    pub fn get_instance_path(&self) -> Option<&PathBuf> {
        self.instance_path.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CgroupUsage {
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub cpu_usage_usec: u64,
    pub pids_current: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_manager_creation() {
        let config = CgroupConfig::default();
        let manager = CgroupManager::new(config);
        assert!(!manager.get_config().enabled);
    }

    #[test]
    fn test_cgroup_config_defaults() {
        let config = CgroupConfig::default();
        assert_eq!(config.limits.memory.max_mb, 4096);
        assert_eq!(config.limits.cpu.max_cpus, 2);
        assert_eq!(config.limits.pids.max_pids, 4096);
    }

    #[test]
    fn test_cgroup_version_detection() {
        let version = CgroupManager::get_version();
        if CgroupManager::is_available() {
            assert!(version.is_some());
        }
    }

    #[test]
    fn test_memory_limit_serialization() {
        let limit = MemoryLimit {
            enabled: true,
            max_mb: 8192,
            soft_limit_mb: Some(4096),
            swap_max_mb: Some(2048),
        };

        let json = serde_json::to_string(&limit).unwrap();
        let deserialized: MemoryLimit = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_mb, 8192);
        assert_eq!(deserialized.soft_limit_mb, Some(4096));
    }

    #[test]
    fn test_cpu_limit_serialization() {
        let limit = CpuLimit {
            enabled: true,
            max_cpus: 4,
            max_weight: Some(1024),
            quota_us: Some(50000),
            period_us: Some(100000),
        };

        let json = serde_json::to_string(&limit).unwrap();
        let deserialized: CpuLimit = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_cpus, 4);
        assert_eq!(deserialized.quota_us, Some(50000));
    }
}
