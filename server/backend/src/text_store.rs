use std::collections::HashMap;

#[derive(Debug)]
pub struct TextStore {
    texts: HashMap<String, String>,
}

impl TextStore {
    pub fn new() -> Self {
        Self {
            texts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, text: String) {
        self.texts.insert(id, text);
    }

    pub fn remove(&mut self, id: &str) {
        self.texts.remove(id);
    }

    pub fn set_to_empty(&mut self, id: &str) {
        self.texts.insert(id.to_string(), "".to_string());
    }

    pub fn to_ws_message(&self) -> String {
        let data = serde_json::json!({
            "type": "update",
            "texts": self.texts.iter().map(|(id, text)| {
                serde_json::json!({ "id": id, "text": text })
            }).collect::<Vec<_>>()
        });
        serde_json::to_string(&data).unwrap()
    }
}
