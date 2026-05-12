use tokio::sync::broadcast;

use crate::api::ws::{ClientManager, WsMessage};
use crate::config::Config;
use crate::core::process_manager::ProcessManager;
use crate::core::rcon_client::RconClient;
use crate::monitor::SystemMonitor;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub process_manager: ProcessManager,
    pub rcon_client: RconClient,
    pub monitor: SystemMonitor,
    pub client_manager: ClientManager,
    pub log_broadcast_tx: broadcast::Sender<WsMessage>,
    pub metrics_broadcast_tx: broadcast::Sender<WsMessage>,
    pub status_broadcast_tx: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (log_broadcast_tx, _) = broadcast::channel(1000);
        let (metrics_broadcast_tx, _) = broadcast::channel(100);
        let (status_broadcast_tx, _) = broadcast::channel(100);

        let process_manager = ProcessManager::new(10000);
        let rcon_client = RconClient::new(
            &config.rcon.address(),
            &config.rcon.password,
        );
        let monitor = SystemMonitor::new(config.monitor.history_size);
        let client_manager = ClientManager::new(config.api.max_ws_connections);

        Self {
            config,
            process_manager,
            rcon_client,
            monitor,
            client_manager,
            log_broadcast_tx,
            metrics_broadcast_tx,
            status_broadcast_tx,
        }
    }
}
