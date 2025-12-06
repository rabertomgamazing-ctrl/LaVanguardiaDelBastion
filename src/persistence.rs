use crate::protocol::{EngineMode, GameMode};
use crate::session::{Reputation, SessionManager};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("No se pudo leer el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("No se pudo serializar/deserializar: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub label: String,
    pub engine_mode: EngineMode,
    pub game_mode: GameMode,
    pub session: SessionManager,
    pub bastion_favor: i32,
    pub reputation: Reputation,
    pub updated_at: DateTime<Utc>,
}

impl CampaignState {
    pub fn new(label: impl Into<String>, engine_mode: EngineMode, game_mode: GameMode) -> Self {
        Self {
            label: label.into(),
            engine_mode,
            game_mode,
            session: SessionManager::new(engine_mode),
            bastion_favor: 0,
            reputation: Reputation::default(),
            updated_at: Utc::now(),
        }
    }

    pub fn save_to_file(&mut self, path: impl AsRef<Path>) -> Result<(), PersistenceError> {
        self.updated_at = Utc::now();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let contents = fs::read_to_string(path)?;
        let mut state: CampaignState = serde_json::from_str(&contents)?;
        state.updated_at = Utc::now();
        Ok(state)
    }

    pub fn apply_clock_tick(&mut self, label: &str) {
        if let Some(clock) = self.session.clocks.iter_mut().find(|c| c.label == label) {
            clock.tick();
        }
    }

    pub fn apply_mark(&mut self, label: &str, amount: u8) {
        if let Some(mark) = self.session.marks.iter_mut().find(|m| m.label == label) {
            mark.apply(amount);
        }
    }

    pub fn adjust_reputation(&mut self, fame_delta: i32, infamy_delta: i32) {
        let rep = self.session.adjust_reputation(fame_delta, infamy_delta);
        self.reputation = rep;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn campaign_state_roundtrip() {
        let mut state = CampaignState::new("Slot 1", EngineMode::Full, GameMode::PartidaEstandar);
        state.apply_clock_tick("Reloj de Amenaza");
        state.apply_mark("Marcas del Bastión", 3);
        state.adjust_reputation(2, 1);

        let path = "./tmp_state.json";
        state.save_to_file(path).expect("write state");
        let loaded = CampaignState::load_from_file(path).expect("read state");
        assert_eq!(loaded.session.clocks[0].filled, 1);
        assert_eq!(loaded.session.marks[0].filled, 3);
        assert_eq!(loaded.reputation.fame, 2);
        let _ = File::create(path); // truncate for cleanup
    }
}
