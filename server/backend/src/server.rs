use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tonic::{Request, Response, Status};

pub mod detector {
    tonic::include_proto!("detector"); // The string specified here must match the proto package name
}

use crate::detector_store::Detection;
use crate::detector_store::DetectorStore;
use detector::{detector_service_server::DetectorService, Empty, ProtoDetections, ProtoPing};

#[derive(Debug)]
pub struct MyDetectorService {
    detectors: Arc<Mutex<DetectorStore>>,
    broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
}

impl MyDetectorService {
    pub fn new(
        detectors: Arc<Mutex<DetectorStore>>,
        broadcast_tx: Arc<Mutex<broadcast::Sender<String>>>,
    ) -> Self {
        Self {
            detectors,
            broadcast_tx,
        }
    }
}

#[tonic::async_trait]
impl DetectorService for MyDetectorService {
    async fn ping(&self, request: Request<ProtoPing>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let mut detectors = self.detectors.lock().await;
        if !detectors.is_registered(&req.id) {
            detectors.register_detector(req.id.clone(), req.ip.clone());
            log::info!("Registered detector: {}, with ip: {}", req.id, req.ip);
        }
        detectors.set_frame_rate(&req.id, req.frame_rate);

        let broadcast_tx = self.broadcast_tx.lock().await;
        broadcast_tx
            .send("registry".to_string())
            .map_err(|e| Status::internal(format!("Failed to send broadcast message: {}", e)))?;
        Ok(Response::new(Empty {}))
    }

    async fn send_detections(
        &self,
        request: Request<ProtoDetections>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let mut detectors = self.detectors.lock().await;
        let detections: Vec<Detection> = req
            .detections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(format!("Failed to convert detection: {}", e)))?;

        detectors.insert(req.id, detections);

        let broadcast_tx = self.broadcast_tx.lock().await;
        broadcast_tx
            .send("detections".to_string())
            .map_err(|e| Status::internal(format!("Failed to send broadcast message: {}", e)))?;

        Ok(Response::new(Empty {}))
    }
}
