use futures_util::{stream::StreamExt as StreamExtTrait, SinkExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize the logger
    std::env::set_var("RUST_LOG", "info");

    env_logger::init();

    let counter = Arc::new(Mutex::new(0));

    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_counter(counter.clone()))
        .map(move |ws: warp::ws::Ws, counter| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| handle_connection(socket, tx, counter))
        });

    let routes = ws_route;

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_counter(
    counter: Arc<Mutex<i32>>,
) -> impl Filter<Extract = (Arc<Mutex<i32>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || counter.clone())
}

async fn handle_connection(
    ws: warp::ws::WebSocket,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
    counter: Arc<Mutex<i32>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = {
        let tx = tx.lock().await;
        tx.subscribe()
    };

    // Send the current counter value upon connection
    {
        let counter = counter.lock().await;
        let msg = format!("Counter: {}", *counter);
        if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
            error!("Error sending initial counter message: {}", e);
        }
    }

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                info!("Received message from client: {:?}", msg);
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    if text == "{\"action\":\"increment\"}" {
                        let mut counter = counter.lock().await;
                        *counter += 1;
                        let msg = format!("Counter: {}", *counter);
                        info!("Sending counter message to client: {}", msg);
                        if let Err(e) = ws_tx.send(warp::ws::Message::text(msg)).await {
                            error!("Error sending counter message: {}", e);
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
