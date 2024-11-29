use std::collections::HashMap;

#[derive(Debug)]
pub struct CounterStore {
    counters: HashMap<String, i32>,
}

impl CounterStore {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, count: i32) {
        self.counters.insert(id, count);
    }

    pub fn remove(&mut self, id: &str) {
        self.counters.remove(id);
    }

    pub fn increment_with(&mut self, id: &str, count: i32) {
        let counter = self.counters.entry(id.to_string()).or_insert(0);
        *counter += count;
    }

    pub fn set_to_zero(&mut self, id: &str) {
        self.counters.insert(id.to_string(), 0);
    }

    pub fn to_ws_message(&self) -> String {
        let data = serde_json::json!({
            "type": "update",
            "counters": self.counters.iter().map(|(id, count)| {
                serde_json::json!({ "id": id, "count": count })
            }).collect::<Vec<_>>()
        });
        serde_json::to_string(&data).unwrap()
    }
}
