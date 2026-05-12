use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, watch, OwnedSemaphorePermit, Semaphore, Mutex as TokioMutex, RwLock as TokioRwLock};
use tracing::{debug, error, info, warn};

use crate::config::ServerConfig;
use crate::error::AppError;

const MAX_LOG_BUFFER_SIZE: usize = 10000;

#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
}

impl ProcessHandle {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            started_at: Utc::now(),
        }
    }
}

pub struct ManagedProcess {
    stdin: Arc<TokioMutex<Option<tokio::process::ChildStdin>>>,
    stop_tx: watch::Sender<bool>,
    handle: ProcessHandle,
}

impl ManagedProcess {
    pub fn handle(&self) -> &ProcessHandle {
        &self.handle
    }

    pub async fn send_command(&self, command: &str) -> std::io::Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            stdin.write_all(format!("{}\n", command).as_bytes()).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "stdin not available",
            ))
        }
    }

    pub async fn is_running(&self) -> bool {
        true
    }

    pub async fn kill(&self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Clone for ManagedProcess {
    fn clone(&self) -> Self {
        Self {
            stdin: self.stdin.clone(),
            stop_tx: self.stop_tx.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(true);
    }
}

#[derive(Clone)]
pub struct ProcessManager {
    process: Arc<TokioRwLock<Option<ManagedProcess>>>,
    log_buffer: Arc<TokioRwLock<VecDeque<LogEntry>>>,
    log_broadcast_tx: Arc<broadcast::Sender<LogEntry>>,
    permit: Arc<Semaphore>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub content: String,
    pub source: LogSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Stdout,
    Stderr,
    System,
}

impl ProcessManager {
    pub fn new(max_log_size: usize) -> Self {
        let (log_broadcast_tx, _) = broadcast::channel(max_log_size);

        Self {
            process: Arc::new(TokioRwLock::new(None)),
            log_buffer: Arc::new(TokioRwLock::new(VecDeque::with_capacity(max_log_size))),
            log_broadcast_tx: Arc::new(log_broadcast_tx),
            permit: Arc::new(Semaphore::new(1)),
        }
    }

    pub async fn start(
        &self,
        config: &ServerConfig,
        permit: OwnedSemaphorePermit,
    ) -> std::result::Result<ProcessHandle, AppError> {
        let permit = Arc::new(permit);

        let is_running = {
            let guard = self.process.read().await;
            guard.is_some()
        };
        if is_running {
            return Err(AppError::ServerAlreadyRunning);
        }

        let mut cmd = tokio::process::Command::new("java");

        for arg in &config.jvm_args {
            if arg == "-jar" {
                continue;
            }
            cmd.arg(arg);
        }
        cmd.arg("-jar");
        cmd.arg(&config.jar_path);

        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        if let Some(parent) = config.jar_path.parent() {
            cmd.current_dir(parent);
        }

        info!("Starting Minecraft server: java {}", config.jvm_args.join(" "));

        let mut child = cmd.spawn().map_err(AppError::ProcessError)?;

        let pid = child.id().ok_or_else(|| {
            AppError::Internal("Failed to get process ID".to_string())
        })?;
        let handle = ProcessHandle::new(pid);

        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::Internal("Failed to capture stdout".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            AppError::Internal("Failed to capture stderr".to_string())
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::Internal("Failed to capture stdin".to_string())
        })?;

        let (stop_tx, stop_rx) = watch::channel(false);

        let log_broadcast_tx = self.log_broadcast_tx.clone();
        let log_buffer = self.log_buffer.clone();
        let permit_clone = permit.clone();

        tokio::spawn(async move {
            let mut stdout_stream = BufReader::new(stdout).lines();
            let mut stderr_stream = BufReader::new(stderr).lines();
            let mut stop_rx = stop_rx;

            loop {
                tokio::select! {
                    biased;

                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            info!("Received stop signal");
                            break;
                        }
                    }

                    line_result = stdout_stream.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                let entry = LogEntry {
                                    timestamp: Utc::now(),
                                    level: detect_log_level(&line),
                                    content: line.clone(),
                                    source: LogSource::Stdout,
                                };
                                let _ = log_broadcast_tx.send(entry.clone());
                                add_to_buffer(&log_buffer, entry, MAX_LOG_BUFFER_SIZE);
                            }
                            Ok(None) => {
                                debug!("stdout stream ended");
                                break;
                            }
                            Err(e) => {
                                error!("stdout error: {}", e);
                                break;
                            }
                        }
                    }

                    line_result = stderr_stream.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                let entry = LogEntry {
                                    timestamp: Utc::now(),
                                    level: "error".to_string(),
                                    content: line.clone(),
                                    source: LogSource::Stderr,
                                };
                                let _ = log_broadcast_tx.send(entry.clone());
                                add_to_buffer(&log_buffer, entry, MAX_LOG_BUFFER_SIZE);
                            }
                            Ok(None) => {
                                debug!("stderr stream ended");
                            }
                            Err(e) => {
                                error!("stderr error: {}", e);
                            }
                        }
                    }
                }
            }

            drop(permit_clone);
        });

        let managed = ManagedProcess { stdin: Arc::new(TokioMutex::new(Some(stdin))), stop_tx, handle: handle.clone() };
        {
            let mut guard = self.process.write().await;
            *guard = Some(managed);
        }

        info!("Minecraft server started with PID: {}", pid);
        Ok(handle)
    }

    pub async fn stop(&self) -> std::result::Result<(), AppError> {
        let managed = {
            let mut guard = self.process.write().await;
            guard.take().ok_or(AppError::ServerNotRunning)?
        };

        info!("Stopping Minecraft server PID: {}", managed.handle.pid);

        if let Err(e) = managed.send_command("stop").await {
            warn!("Failed to send stop command: {}", e);
        }

        let mut attempts = 0;
        while managed.is_running().await && attempts < 300 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        if managed.is_running().await {
            warn!("Graceful stop timeout, forcing kill");
            let _ = managed.kill().await;
        }

        info!("Minecraft server stopped");
        Ok(())
    }

    pub async fn restart(&self, config: &ServerConfig) -> std::result::Result<ProcessHandle, AppError> {
        info!("Restarting Minecraft server");
        self.stop().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let permit = self.acquire_permit().await?;
        self.start(config, permit).await
    }

    pub async fn is_running(&self) -> bool {
        let guard = self.process.read().await;
        match &*guard {
            Some(p) => p.is_running().await,
            None => false,
        }
    }

    pub async fn send_command(&self, command: &str) -> std::result::Result<(), AppError> {
        let cmd = self.get_managed_process().await?;
        cmd.send_command(command).await.map_err(AppError::ProcessError)?;

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "info".to_string(),
            content: format!("> {}", command),
            source: LogSource::System,
        };
        let _ = self.log_broadcast_tx.send(entry.clone());
        add_to_buffer(&self.log_buffer, entry, MAX_LOG_BUFFER_SIZE);

        Ok(())
    }

    async fn get_managed_process(&self) -> std::result::Result<ManagedProcess, AppError> {
        let guard = self.process.read().await;
        guard.clone().ok_or(AppError::ServerNotRunning)
    }

    pub async fn get_logs(&self, offset: usize) -> Vec<LogEntry> {
        let buffer = self.log_buffer.read().await;
        buffer.iter().skip(offset).cloned().collect()
    }

    pub async fn get_pid(&self) -> Option<u32> {
        let guard = self.process.read().await;
        guard.as_ref().map(|p| p.handle.pid)
    }

    pub async fn acquire_permit(&self) -> std::result::Result<OwnedSemaphorePermit, AppError> {
        self.permit.clone().acquire_owned()
            .await
            .map_err(|_| AppError::Internal("Failed to acquire semaphore".to_string()))
    }

    pub fn log_broadcast_rx(&self) -> broadcast::Receiver<LogEntry> {
        self.log_broadcast_tx.subscribe()
    }
}

fn detect_log_level(line: &str) -> String {
    let lower = line.to_lowercase();
    if lower.contains("error") || lower.contains("fatal") || lower.contains("exception") {
        "error".to_string()
    } else if lower.contains("warn") || lower.contains("warning") {
        "warn".to_string()
    } else if lower.contains("debug") {
        "debug".to_string()
    } else {
        "info".to_string()
    }
}

fn add_to_buffer(buffer: &Arc<TokioRwLock<VecDeque<LogEntry>>>, entry: LogEntry, max_size: usize) {
    let mut buf = tokio::runtime::Handle::current().block_on(async { buffer.write().await });
    if buf.len() >= max_size {
        buf.pop_front();
    }
    buf.push_back(entry);
}
