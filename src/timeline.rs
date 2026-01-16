use eframe::epaint::Color32;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

#[cfg(target_arch = "wasm32")]
use web_time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEvent {
    pub title: String,
    pub description: String,
    pub day: u8,
    pub month: u8,
    pub year: i32,
    pub image_path: Option<String>,
    pub color: [u8; 4], // [r, g, b, a]
}

#[derive(Debug, Clone)]
pub struct Event {
    pub title: String,
    pub description: String,
    pub timestamp: SystemTime,
    pub day: u8,
    pub month: u8,
    pub year: i32,
    pub image_path: Option<String>,
    pub color: Color32,
}

impl Event {
    pub fn to_serializable(&self) -> SerializableEvent {
        SerializableEvent {
            title: self.title.clone(),
            description: self.description.clone(),
            day: self.day,
            month: self.month,
            year: self.year,
            image_path: self.image_path.clone(),
            color: self.color.to_array(),
        }
    }

    pub fn from_serializable(s: SerializableEvent) -> Self {
        Self::new(s.title, s.description, s.day, s.month, s.year, s.image_path)
    }
}

impl Event {
    pub fn new(
        title: String,
        description: String,
        day: u8,
        month: u8,
        year: i32,
        image_path: Option<String>,
    ) -> Self {
        // Convert date to timestamp (approximate)
        let now = SystemTime::now();
        let current_year = 2026;
        let years_ago = current_year - year;

        let timestamp = if years_ago >= 0 {
            now - std::time::Duration::from_secs((years_ago as u64) * 365 * 24 * 60 * 60)
        } else {
            now + std::time::Duration::from_secs(((-years_ago) as u64) * 365 * 24 * 60 * 60)
        };

        Self {
            title,
            description,
            timestamp,
            day,
            month,
            year,
            image_path,
            color: Color32::from_rgb(100, 150, 255),
        }
    }

    pub fn today(title: String, description: String) -> Self {
        Self::new(title, description, 16, 1, 2026, None)
    }
}

pub struct Timeline {
    events: Vec<Event>,
}

impl Timeline {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.events.sort_by_key(|e| e.timestamp);
    }

    pub fn remove_event(&mut self, index: usize) {
        if index < self.events.len() {
            self.events.remove(index);
        }
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let serializable: Vec<SerializableEvent> =
            self.events.iter().map(|e| e.to_serializable()).collect();
        serde_json::to_string(&serializable)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let serializable: Vec<SerializableEvent> = serde_json::from_str(json)?;
        let mut timeline = Timeline::new();
        for s in serializable {
            let mut event = Event::from_serializable(s.clone());
            event.color =
                Color32::from_rgba_unmultiplied(s.color[0], s.color[1], s.color[2], s.color[3]);
            timeline.add_event(event);
        }
        Ok(timeline)
    }
}
