use axum::{
    extract::{
        ws::{WebSocket, Message, WebSocketUpgrade},
        State,
    },
    response::{Response, IntoResponse},
    http::StatusCode,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::api::ws::client::WsClient;
use crate::api::ws::message::{WsClientMessage, WsMessage};
use crate::state::AppState;

const HEARTBEAT_INTERVAL_SECS: u64 = 30;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    if state.client_manager.client_count() >= state.config.api.max_ws_connections {
        tracing::warn!("Max connections reached, rejecting WebSocket");
        return (StatusCode::SERVICE_UNAVAILABLE, "Max connections reached").into_response();
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let client_id = Uuid::new_v4();

    let client = WsClient::new(client_id);
    if state.client_manager.add_client(client).is_none() {
            let mut sender = sender.lock().await;
            let _ = sender.send(Message::Close(Some(axum::extract::ws::CloseFrame {
                code: axum::extract::ws::close_code::AWAY,
                reason: "Too many connections".into(),
            }))).await;
            return;
        }

    let client_id_for_tasks = client_id.clone();
    let client_manager = state.client_manager.clone();
    let sender_for_recv = sender.clone();

    let (log_tx, log_rx) = mpsc::channel::<WsMessage>(100);
    let (metrics_tx, metrics_rx) = mpsc::channel::<WsMessage>(100);
    let (status_tx, status_rx) = mpsc::channel::<WsMessage>(100);

    let log_broadcast_rx = state.log_broadcast_tx.subscribe();
    let metrics_broadcast_rx = state.metrics_broadcast_tx.subscribe();
    let status_broadcast_rx = state.status_broadcast_tx.subscribe();

    let log_sender = log_tx.clone();
    tokio::spawn(async move {
        let mut rx = log_broadcast_rx;
        while let Ok(msg) = rx.recv().await {
            let _ = log_sender.send(msg).await;
        }
    });

    let metrics_sender = metrics_tx.clone();
    tokio::spawn(async move {
        let mut rx = metrics_broadcast_rx;
        while let Ok(msg) = rx.recv().await {
            let _ = metrics_sender.send(msg).await;
        }
    });

    let status_sender = status_tx.clone();
    tokio::spawn(async move {
        let mut rx = status_broadcast_rx;
        while let Ok(msg) = rx.recv().await {
            let _ = status_sender.send(msg).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        let sender = sender_for_recv;
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                        client_manager.handle_message(&client_id_for_tasks, client_msg);
                    }
                }
                Ok(Message::Ping(data)) => {
                    let mut sender = sender.lock().await;
                    let _ = sender.send(Message::Pong(data)).await;
                }
                Ok(Message::Pong(_)) => {
                    if let Some(mut c) = client_manager.get_client(&client_id_for_tasks) {
                        c.update_ping();
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    let client_manager_clone = state.client_manager.clone();
    let client_id_clone = client_id.clone();
    let log_rx_clone = log_rx;
    let metrics_rx_clone = metrics_rx;
    let status_rx_clone = status_rx;

    let send_task = tokio::spawn(async move {
        let sender = sender.clone();
        let mut log_rx = log_rx_clone;
        let mut metrics_rx = metrics_rx_clone;
        let mut status_rx = status_rx_clone;
        let mut heartbeat = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    let mut sender = sender.lock().await;
                    let _ = sender.send(Message::Ping(vec![0; 4])).await;
                }

                Some(msg) = log_rx.recv() => {
                    let mut sender = sender.lock().await;
                    if sender.send(Message::Text(msg.to_json().into())).await.is_err() {
                        break;
                    }
                }

                Some(msg) = metrics_rx.recv() => {
                    let mut sender = sender.lock().await;
                    if sender.send(Message::Text(msg.to_json().into())).await.is_err() {
                        break;
                    }
                }

                Some(msg) = status_rx.recv() => {
                    let mut sender = sender.lock().await;
                    if sender.send(Message::Text(msg.to_json().into())).await.is_err() {
                        break;
                    }
                }
            }
        }

        client_manager_clone.remove_client(&client_id_clone);
    });

    tokio::select! {
        _ = recv_task => {}
        _ = send_task => {}
    }

    tracing::debug!("WebSocket session ended for client {}", client_id);
}

pub fn start_broadcast_tasks(state: Arc<AppState>) {
    let log_broadcast_tx = state.log_broadcast_tx.clone();
    let metrics_broadcast_tx = state.metrics_broadcast_tx.clone();
    let status_broadcast_tx = state.status_broadcast_tx.clone();
    let monitor = state.monitor.clone();
    let process_manager = state.process_manager.clone();
    let rcon_client = state.rcon_client.clone();
    let interval_secs = state.config.monitor.interval_secs;

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_secs));

        loop {
            ticker.tick().await;

            let server_pid = process_manager.get_pid().await;
            let is_running = process_manager.is_running().await;

            let metrics = monitor.collect(server_pid).await;
            let msg = WsMessage::metrics(
                metrics.system.cpu_usage,
                metrics.system.memory_percent,
                metrics.system.memory_used,
                metrics.system.memory_total,
            );
            let _ = metrics_broadcast_tx.send(msg);

            let (players_online, tps) = if is_running && rcon_client.is_connected().await {
                let stats = rcon_client.get_cached_stats().await;
                (stats.online_players.len() as u32, stats.tps)
            } else {
                (0, None)
            };

            let status_msg = WsMessage::status(is_running, server_pid, players_online, tps);
            let _ = status_broadcast_tx.send(status_msg);
        }
    });

    let process_manager_for_logs = state.process_manager.clone();
    tokio::spawn(async move {
        let mut log_rx = process_manager_for_logs.log_broadcast_rx();
        while let Ok(entry) = log_rx.recv().await {
            let msg = WsMessage::log(
                &entry.content,
                &entry.level,
                &format!("{:?}", entry.source),
            );
            let _ = log_broadcast_tx.send(msg);
        }
    });
}
