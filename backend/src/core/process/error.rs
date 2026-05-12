use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Instance already exists: {0}")]
    InstanceAlreadyExists(String),

    #[error("Cluster error: {0}")]
    ClusterError(String),

    #[error("Container error: {0}")]
    ContainerError(String),

    #[error("Graceful shutdown timeout")]
    GracefulShutdownTimeout,

    #[error("Watchdog timeout for instance: {0}")]
    WatchdogTimeout(String),

    #[error("JVM tuning error: {0}")]
    JvmTuningError(String),

    #[error("Warmup timeout")]
    WarmupTimeout,

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Cgroup error: {0}")]
    CgroupError(String),

    #[error("Invalid start mode: {0}")]
    InvalidStartMode(String),

    #[error("Crash diagnostic error: {0}")]
    CrashDiagnosticError(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Process not running")]
    ProcessNotRunning,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
