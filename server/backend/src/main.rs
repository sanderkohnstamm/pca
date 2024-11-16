use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use warp::Filter;

mod websocket_connection;
use websocket_connection::WebSocketConnection;

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
            ws.on_upgrade(move |socket| {
                let connection = WebSocketConnection::new(socket, tx, counter);
                connection.handle_connection()
            })
        });

    let routes = ws_route;

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_counter(
    counter: Arc<Mutex<i32>>,
) -> impl Filter<Extract = (Arc<Mutex<i32>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || counter.clone())
}
