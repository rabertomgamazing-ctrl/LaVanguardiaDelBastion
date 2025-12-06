use crate::protocol::EngineMode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PARAGRAPHS_PER_THREAT: u32 = 24;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Clock {
    pub label: String,
    pub steps: u8,
    pub filled: u8,
}

impl Clock {
    pub fn new(label: impl Into<String>, steps: u8) -> Self {
        Self {
            label: label.into(),
            steps,
            filled: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.filled < self.steps {
            self.filled += 1;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.filled >= self.steps
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkTrack {
    pub label: String,
    pub segments: u8,
    pub filled: u8,
}

impl MarkTrack {
    pub fn new(label: impl Into<String>, segments: u8) -> Self {
        Self {
            label: label.into(),
            segments,
            filled: 0,
        }
    }

    pub fn apply(&mut self, amount: u8) {
        self.filled = (self.filled + amount).min(self.segments);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManager {
    engine_mode: EngineMode,
    paragraphs: u32,
    threat_triggers: u32,
    last_threat_at: Option<DateTime<Utc>>,
    pub clocks: Vec<Clock>,
    pub marks: Vec<MarkTrack>,
}

impl SessionManager {
    pub fn new(engine_mode: EngineMode) -> Self {
        Self {
            engine_mode,
            paragraphs: 0,
            threat_triggers: 0,
            last_threat_at: None,
            clocks: vec![Clock::new("Reloj de Amenaza", 8)],
            marks: vec![MarkTrack::new("Marcas del Bastión", 24)],
        }
    }

    pub fn update_engine_mode(&mut self, mode: EngineMode) {
        self.engine_mode = mode;
    }

    /// Registra un párrafo; si alcanza 24 activa Turno de Amenaza.
    pub fn register_paragraph(&mut self) -> bool {
        self.paragraphs += 1;
        if self.paragraphs % PARAGRAPHS_PER_THREAT == 0 {
            self.threat_triggers += 1;
            self.last_threat_at = Some(Utc::now());
            self.clocks.iter_mut().for_each(Clock::tick);
            return true;
        }
        false
    }

    pub fn paragraphs(&self) -> u32 {
        self.paragraphs
    }

    pub fn threat_turns_triggered(&self) -> u32 {
        self.threat_triggers
    }

    pub fn last_threat_at(&self) -> Option<DateTime<Utc>> {
        self.last_threat_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_ticks_on_threat_turn() {
        let mut session = SessionManager::new(EngineMode::Full);
        for _ in 0..PARAGRAPHS_PER_THREAT {
            session.register_paragraph();
        }
        assert_eq!(session.clocks[0].filled, 1);
        assert_eq!(session.threat_triggers, 1);
    }
}
