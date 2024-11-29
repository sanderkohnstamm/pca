use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tonic::{Request, Response, Status};

pub mod counter {
    tonic::include_proto!("counter"); // The string specified here must match the proto package name
}

use crate::counter_store::CounterStore;
use crate::text_store::TextStore;
use counter::counter_service_server::CounterService;
use counter::{Empty, ProtoCount, ProtoPing, ProtoText};

#[derive(Debug)]
pub struct MyCounterService {
    counters: Arc<Mutex<CounterStore>>,
    texts: Arc<Mutex<TextStore>>,
    broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
}

impl MyCounterService {
    pub fn new(
        counters: Arc<Mutex<CounterStore>>,
        texts: Arc<Mutex<TextStore>>,
        broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
    ) -> Self {
        Self {
            counters,
            texts,
            broadcast_tx,
        }
    }
}

#[tonic::async_trait]
impl CounterService for MyCounterService {
    async fn ping(&self, request: Request<ProtoPing>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!("Received Ping request: id={}", req.id);
        Ok(Response::new(Empty {}))
    }

    async fn update_counter_with(
        &self,
        request: Request<ProtoCount>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!(
            "Received UpdateCounterWith request: id={}, count={}",
            req.id, req.count
        );

        let mut counters = self.counters.lock().await;
        counters.increment_with(&req.id, req.count);
        self.broadcast_tx.lock().await.send("".to_string()).unwrap();
        Ok(Response::new(Empty {}))
    }

    async fn send_text(&self, request: Request<ProtoText>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!(
            "Received SendText request: id={}, text={}",
            req.id, req.text
        );
        let mut texts = self.texts.lock().await;
        texts.insert(req.id, req.text);
        self.broadcast_tx.lock().await.send("".to_string()).unwrap();
        Ok(Response::new(Empty {}))
    }
}
