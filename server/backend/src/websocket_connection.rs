use crate::detector_store::DetectorStore;
use futures_util::SinkExt;
use futures_util::StreamExt;
use log::{error, info};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use warp::filters::ws::Message;
use warp::ws::WebSocket;

pub struct WebSocketConnection {
    ws: WebSocket,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    detectors: Arc<Mutex<DetectorStore>>,
}

impl WebSocketConnection {
    pub fn new(
        ws: WebSocket,
        tx: Arc<Mutex<broadcast::Sender<String>>>,
        detectors: Arc<Mutex<DetectorStore>>,
    ) -> Self {
        Self { ws, tx, detectors }
    }

    pub async fn handle_connection(self) {
        let (mut ws_tx, mut ws_rx) = self.ws.split();
        let mut rx = {
            let tx = self.tx.lock().await;
            tx.subscribe()
        };

        // Send the current counter value upon connection
        {
            let detectors = self.detectors.lock().await;
            let msg = detectors.to_ws_message();
            if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                error!("Error sending initial counter message: {}", e);
            }
        }

        // Spawn a task to listen for broadcast messages and send them to the client
        let counter_store_clone = self.detectors.clone();
        tokio::spawn(async move {
            while let Ok(_) = rx.recv().await {
                let counter = counter_store_clone.lock().await;
                let msg = counter.to_ws_message();
                if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                    error!("Error sending counter message: {}", e);
                }
            }
        });

        while let Some(result) = ws_rx.next().await {
            let tx = self.tx.clone();
            match result {
                Ok(msg) => {
                    let detectors: tokio::sync::MutexGuard<'_, DetectorStore> =
                        self.detectors.lock().await;
                    info!("Received message from client: {:?}", msg);
                    handle_message(msg, detectors, tx).await;
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                }
            }
        }

        info!("WebSocket connection closed");
    }
}

async fn handle_message(
    msg: Message,
    mut detectors: tokio::sync::MutexGuard<'_, DetectorStore>,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
) {
    if let Ok(text) = msg.to_str() {
        let data: Value = serde_json::from_str(text).unwrap();
        if let Some(action) = data["action"].as_str() {
            match action {
                "remove" => {
                    if let Some(id) = data["id"].as_str() {
                        detectors.remove(id);
                    }
                }
                "set_to_empty" => {
                    if let Some(id) = data["id"].as_str() {
                        detectors.set_empty(id);
                    }
                }
                _ => {}
            }
        }
    }
    tx.lock().await.send("".to_string()).unwrap();
}
