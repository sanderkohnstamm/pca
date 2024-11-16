use futures_util::{stream::StreamExt as StreamExtTrait, SinkExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use warp::ws::WebSocket;

pub struct WebSocketConnection {
    ws: WebSocket,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    counter: Arc<Mutex<i32>>,
}

impl WebSocketConnection {
    pub fn new(
        ws: WebSocket,
        tx: Arc<Mutex<broadcast::Sender<String>>>,
        counter: Arc<Mutex<i32>>,
    ) -> Self {
        Self { ws, tx, counter }
    }

    pub async fn handle_connection(self) {
        let (mut ws_tx, mut ws_rx) = self.ws.split();
        let mut rx = {
            let tx = self.tx.lock().await;
            tx.subscribe()
        };

        // Send the current counter value upon connection
        {
            let counter = self.counter.lock().await;
            let msg = format!("Counter: {}", *counter);
            if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                error!("Error sending initial counter message: {}", e);
            }
        }

        // Spawn a task to listen for broadcast messages and send them to the client
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                    error!("Error sending counter message: {}", e);
                }
            }
        });

        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    info!("Received message from client: {:?}", msg);
                    if msg.is_text() {
                        let text = msg.to_str().unwrap();
                        if text == "{\"action\":\"increment\"}" {
                            let mut counter = self.counter.lock().await;
                            *counter += 1;
                            let msg = format!("Counter: {}", *counter);
                            info!("Broadcasting counter message: {}", msg);
                            let tx = self.tx.lock().await;
                            if let Err(e) = tx.send(msg) {
                                error!("Error broadcasting counter message: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                }
            }
        }

        info!("WebSocket connection closed");
    }
}
