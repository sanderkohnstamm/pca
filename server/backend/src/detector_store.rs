use crate::server::detector::ProtoDetection;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DetectorStore {
    detectors: HashMap<String, Vec<Detection>>,
}

impl DetectorStore {
    pub fn new() -> Self {
        Self {
            detectors: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, detections: Vec<Detection>) {
        self.detectors.insert(id, detections);
    }

    pub fn remove(&mut self, id: &str) {
        self.detectors.remove(id);
    }

    pub fn set_empty(&mut self, id: &str) {
        self.detectors.insert(id.to_string(), Vec::new());
    }

    pub fn to_ws_message(&self) -> String {
        let messages: Vec<DetectorMessage> = self
            .detectors
            .iter()
            .map(|(id, detections)| DetectorMessage { id, detections })
            .collect();

        serde_json::to_string(&messages).unwrap()
    }
}

impl TryFrom<ProtoDetection> for Detection {
    type Error = String;
    fn try_from(proto: ProtoDetection) -> Result<Self, String> {
        let class = proto.class_name;
        let score = proto.score;
        let Some(bounding_box) = proto.bounding_box else {
            return Err("Bounding box not found".to_string());
        };

        let bounding_box = BoundingBox {
            center_x: bounding_box.center_x,
            center_y: bounding_box.center_y,
            width: bounding_box.width,
            height: bounding_box.height,
        };
        Ok(Self {
            class,
            score,
            bounding_box,
        })
    }
}

#[derive(Serialize)]
struct DetectorMessage<'a> {
    id: &'a str,
    detections: &'a [Detection],
}

#[derive(Debug, Serialize)]
pub struct Detection {
    class: String,
    score: f32,
    bounding_box: BoundingBox,
}

#[derive(Debug, Serialize)]
pub struct BoundingBox {
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
}
