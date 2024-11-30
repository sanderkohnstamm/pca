use server::MyDetectorService;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use tonic::transport::Server;
use warp::Filter;
mod websocket_connection;
use websocket_connection::WebSocketConnection;

mod server;
use server::detector::detector_service_server::DetectorServiceServer;

mod detector_store;
use detector_store::DetectorStore;

#[tokio::main]
async fn main() {
    // Initialize the logger
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let detectors = Arc::new(Mutex::new(DetectorStore::new()));

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
        _ = start_ws_server(detectors.clone(),  tx.clone()) => {},
        _ = start_grpc_server(detectors.clone(), tx.clone()) => {},
        _ = frontend.wait_with_output() => {},
    }
}

async fn start_ws_server(
    detectors: Arc<Mutex<DetectorStore>>,
    tx: Arc<Mutex<broadcast::Sender<String>>>,
) {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_detectors(detectors.clone()))
        .map(move |ws: warp::ws::Ws, detectors| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| {
                let connection = WebSocketConnection::new(socket, tx, detectors);
                connection.handle_connection()
            })
        });

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

pub async fn start_grpc_server(
    detectors: Arc<Mutex<DetectorStore>>,
    broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let detector_service = MyDetectorService::new(detectors, broadcast_tx);

    println!("gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(DetectorServiceServer::new(detector_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn with_detectors(
    counter: Arc<Mutex<DetectorStore>>,
) -> impl Filter<Extract = (Arc<Mutex<DetectorStore>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || counter.clone())
}
