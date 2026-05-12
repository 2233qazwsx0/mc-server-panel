use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch, OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::{info, warn};

use super::error::ProcessError;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    Initiated,
    SignalSent,
    WaitingForStop,
    ForceKill,
    Completed,
}

pub struct GracefulShutdown {
    stop_tx: watch::Sender<bool>,
    phase: Arc<std::sync::RwLock<ShutdownPhase>>,
    timeout_duration: Duration,
}

impl GracefulShutdown {
    pub fn new(timeout_duration: Duration) -> Self {
        let (stop_tx, _) = watch::channel(false);

        Self {
            stop_tx,
            phase: Arc::new(std::sync::RwLock::new(ShutdownPhase::Initiated)),
            timeout_duration,
        }
    }

    pub fn get_stop_rx(&self) -> watch::Receiver<bool> {
        self.stop_tx.subscribe()
    }

    pub async fn initiate(&self) -> Result<(), ProcessError> {
        {
            let mut phase = self.phase.write().unwrap();
            *phase = ShutdownPhase::SignalSent;
        }

        self.stop_tx.send(true)
            .map_err(|_| ProcessError::GracefulShutdownTimeout)?;

        {
            let mut phase = self.phase.write().unwrap();
            *phase = ShutdownPhase::WaitingForStop;
        }

        info!("Graceful shutdown initiated");
        Ok(())
    }

    pub fn get_phase(&self) -> ShutdownPhase {
        *self.phase.read().unwrap()
    }

    pub async fn wait_for_completion<F, Fut>(&self, check_running: F) -> Result<ShutdownPhase, ProcessError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let mut attempts = 0;
        let max_attempts = (self.timeout_duration.as_secs() * 10) as usize;

        loop {
            if !check_running().await {
                {
                    let mut phase = self.phase.write().unwrap();
                    *phase = ShutdownPhase::Completed;
                }
                info!("Process stopped gracefully");
                return Ok(ShutdownPhase::Completed);
            }

            if attempts >= max_attempts {
                warn!("Graceful shutdown timeout, forcing kill");
                {
                    let mut phase = self.phase.write().unwrap();
                    *phase = ShutdownPhase::ForceKill;
                }
                return Err(ProcessError::GracefulShutdownTimeout);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }
    }

    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    pub fn get_timeout(&self) -> Duration {
        self.timeout_duration
    }
}

pub struct ShutdownManager {
    graceful: Arc<GracefulShutdown>,
    force_kill_tx: broadcast::Sender<()>,
}

impl ShutdownManager {
    pub fn new(timeout_duration: Duration) -> Self {
        let (force_kill_tx, _) = broadcast::channel(1);

        Self {
            graceful: Arc::new(GracefulShutdown::new(timeout_duration)),
            force_kill_tx,
        }
    }

    pub fn graceful(&self) -> &GracefulShutdown {
        &self.graceful
    }

    pub fn request_force_kill(&self) -> Result<(), ProcessError> {
        self.force_kill_tx
            .send(())
            .map_err(|_| ProcessError::GracefulShutdownTimeout)?;
        info!("Force kill requested");
        Ok(())
    }

    pub fn force_kill_rx(&self) -> broadcast::Receiver<()> {
        self.force_kill_tx.subscribe()
    }
}

impl Clone for GracefulShutdown {
    fn clone(&self) -> Self {
        Self {
            stop_tx: self.stop_tx.clone(),
            phase: self.phase.clone(),
            timeout_duration: self.timeout_duration,
        }
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_graceful_shutdown_initiation() {
        let shutdown = GracefulShutdown::default();
        assert_eq!(shutdown.get_phase(), ShutdownPhase::Initiated);

        shutdown.initiate().await.unwrap();
        assert_eq!(shutdown.get_phase(), ShutdownPhase::SignalSent);
    }

    #[tokio::test]
    async fn test_shutdown_wait_for_completion() {
        let shutdown = GracefulShutdown::new(Duration::from_millis(100));
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            running_clone.store(false, Ordering::SeqCst);
        });

        let result = shutdown.wait_for_completion(|| async { running.load(Ordering::SeqCst) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ShutdownPhase::Completed);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let shutdown = GracefulShutdown::new(Duration::from_millis(50));

        let result = shutdown.wait_for_completion(|| async { true }).await;
        assert!(result.is_err());
        assert_eq!(shutdown.get_phase(), ShutdownPhase::ForceKill);
    }

    #[tokio::test]
    async fn test_shutdown_manager_force_kill() {
        let manager = ShutdownManager::new(Duration::from_secs(30));
        assert!(manager.request_force_kill().is_ok());

        let mut rx = manager.force_kill_rx();
        let result = rx.recv().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_shutdown_timeout_configuration() {
        let mut shutdown = GracefulShutdown::new(Duration::from_secs(10));
        assert_eq!(shutdown.get_timeout(), Duration::from_secs(10));

        shutdown.set_timeout(Duration::from_secs(60));
        assert_eq!(shutdown.get_timeout(), Duration::from_secs(60));
    }
}
