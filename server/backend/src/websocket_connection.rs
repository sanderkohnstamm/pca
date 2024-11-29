use crate::counter_store::CounterStore;
use crate::text_store::TextStore;
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
    counters: Arc<Mutex<CounterStore>>,
    texts: Arc<Mutex<TextStore>>,
}

impl WebSocketConnection {
    pub fn new(
        ws: WebSocket,
        tx: Arc<Mutex<broadcast::Sender<String>>>,
        counters: Arc<Mutex<CounterStore>>,
        texts: Arc<Mutex<TextStore>>,
    ) -> Self {
        Self {
            ws,
            tx,
            counters,
            texts,
        }
    }

    pub async fn handle_connection(self) {
        let (mut ws_tx, mut ws_rx) = self.ws.split();
        let mut rx = {
            let tx = self.tx.lock().await;
            tx.subscribe()
        };

        // Send the current counter value upon connection
        {
            let counter = self.counters.lock().await;
            let msg = counter.to_ws_message();
            if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                error!("Error sending initial counter message: {}", e);
            }
            let text = self.texts.lock().await;
            let msg = text.to_ws_message();
            if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                error!("Error sending initial text message: {}", e);
            }
        }

        // Spawn a task to listen for broadcast messages and send them to the client
        let counter_store_clone = self.counters.clone();
        let text_store_clone = self.texts.clone();
        tokio::spawn(async move {
            while let Ok(_) = rx.recv().await {
                let counter = counter_store_clone.lock().await;
                let msg = counter.to_ws_message();
                if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                    error!("Error sending counter message: {}", e);
                }
                let text = text_store_clone.lock().await;
                let msg = text.to_ws_message();
                if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                    error!("Error sending initial text message: {}", e);
                }
            }
        });

        while let Some(result) = ws_rx.next().await {
            let tx = self.tx.clone();
            match result {
                Ok(msg) => {
                    let counters: tokio::sync::MutexGuard<'_, CounterStore> =
                        self.counters.lock().await;
                    let texts: tokio::sync::MutexGuard<'_, TextStore> = self.texts.lock().await;
                    info!("Received message from client: {:?}", msg);
                    handle_message(msg, counters, texts, tx).await;
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
    mut counters: tokio::sync::MutexGuard<'_, CounterStore>,
    mut texts: tokio::sync::MutexGuard<'_, TextStore>,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
) {
    if let Ok(text) = msg.to_str() {
        let data: Value = serde_json::from_str(text).unwrap();
        if let Some(action) = data["action"].as_str() {
            match action {
                "increment" => {
                    if let Some(id) = data["id"].as_str() {
                        counters.increment_with(id, 1);
                    }
                }
                "decrement" => {
                    if let Some(id) = data["id"].as_str() {
                        counters.increment_with(id, -1);
                    }
                }
                "set" => {
                    if let Some(id) = data["id"].as_str() {
                        if let Some(count) = data["count"].as_i64() {
                            counters.insert(id.to_string(), count as i32);
                        }
                    }
                }
                "remove" => {
                    if let Some(id) = data["id"].as_str() {
                        counters.remove(id);
                    }
                }
                "set_to_empty" => {
                    if let Some(id) = data["id"].as_str() {
                        texts.set_to_empty(id);
                    }
                }
                _ => {}
            }
        }
    }
    tx.lock().await.send("".to_string()).unwrap();
}
