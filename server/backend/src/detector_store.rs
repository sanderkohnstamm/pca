use crate::server::detector::ProtoDetection;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DetectorStore {
    detections: HashMap<String, Vec<Detection>>,
    ips: HashMap<String, String>,
    frame_rates: HashMap<String, f32>,
}

impl DetectorStore {
    pub fn new() -> Self {
        Self {
            detections: HashMap::new(),
            ips: HashMap::new(),
            frame_rates: HashMap::new(),
        }
    }

    pub fn register_detector(&mut self, id: String, ip: String) {
        self.ips.insert(id, ip);
    }

    pub fn get_ip(&self, id: &str) -> Option<&String> {
        self.ips.get(id)
    }

    pub fn is_registered(&self, id: &str) -> bool {
        self.ips.contains_key(id)
    }

    pub fn set_frame_rate(&mut self, id: &str, frame_rate: f32) {
        self.frame_rates.insert(id.to_string(), frame_rate);
    }

    pub fn insert(&mut self, id: String, detections: Vec<Detection>) {
        self.detections.insert(id, detections);
    }

    pub fn remove(&mut self, id: &str) {
        self.detections.remove(id);
    }

    pub fn set_empty(&mut self, id: &str) {
        self.detections.insert(id.to_string(), Vec::new());
    }

    pub fn to_ws_message(&self) -> String {
        let mut messages: Vec<DetectorMessage> = Vec::new();

        for (id, detections) in &self.detections {
            let Some(ip) = self.ips.get(id) else {
                continue;
            };

            let Some(frame_rate) = self.frame_rates.get(id) else {
                continue;
            };

            messages.push(DetectorMessage {
                id,
                detections,
                ip,
                frame_rate,
            });
        }

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
    ip: &'a str,
    frame_rate: &'a f32,
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
