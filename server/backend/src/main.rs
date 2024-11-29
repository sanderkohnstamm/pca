use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use tonic::transport::Server;
use warp::Filter;
mod websocket_connection;
use websocket_connection::WebSocketConnection;

mod server;
use server::{counter::counter_service_server::CounterServiceServer, MyCounterService};

mod counter_store;
use counter_store::CounterStore;

mod text_store;
use text_store::TextStore;

#[tokio::main]
async fn main() {
    // Initialize the logger
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let counters = Arc::new(Mutex::new(CounterStore::new()));
    let texts = Arc::new(Mutex::new(TextStore::new()));

    // Start the frontend
    let frontend = Command::new("npm")
        .arg("run")
        .arg("dev")
        .current_dir("../frontend") // Set the working directory
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start frontend");

    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));

    tokio::select! {
        _ = start_ws_server(counters.clone(), texts.clone(), tx.clone()) => {},
        _ = start_grpc_server(counters.clone(),texts.clone(), tx.clone()) => {},
        _ = frontend.wait_with_output() => {},
    }
}

async fn start_ws_server(
    counters: Arc<Mutex<CounterStore>>,
    texts: Arc<Mutex<TextStore>>,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
) {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_counter(counters.clone()))
        .and(with_text(texts.clone()))
        .map(move |ws: warp::ws::Ws, counter, text| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| {
                let connection = WebSocketConnection::new(socket, tx, counter, text);
                connection.handle_connection()
            })
        });

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

pub async fn start_grpc_server(
    counters: Arc<Mutex<CounterStore>>,
    texts: Arc<Mutex<TextStore>>,
    broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let counter_service = MyCounterService::new(counters, texts, broadcast_tx);

    println!("gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(CounterServiceServer::new(counter_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn with_counter(
    counter: Arc<Mutex<CounterStore>>,
) -> impl Filter<Extract = (Arc<Mutex<CounterStore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || counter.clone())
}

fn with_text(
    text: Arc<Mutex<TextStore>>,
) -> impl Filter<Extract = (Arc<Mutex<TextStore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || text.clone())
}
