use crate::dice::RollResult;
use crate::protocol::{EngineMode, GameMode};
use crate::session::{Clock, MarkTrack};
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FxPreset {
    BloodVignette,
    RunePulse,
    Fog,
    Glitch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioPreset {
    LowDrone,
    BellStrike,
    DiceHit,
    ThreatPulse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineEventKind {
    ModeChanged { mode: EngineMode },
    GameModeChanged { game_mode: GameMode },
    ParagraphRegistered { count: u32 },
    ThreatTurn { total: u32 },
    RollResolved { roll: RollResult },
    ClockAdvanced { clock: Clock },
    MarkApplied { mark: MarkTrack },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub at: DateTime<Utc>,
    pub kind: EngineEventKind,
    pub fx: Vec<FxPreset>,
    pub audio: Vec<AudioPreset>,
}

impl EventEnvelope {
    pub fn new(kind: EngineEventKind) -> Self {
        let (fx, audio) = Self::presets_for(&kind);
        Self {
            at: Utc::now(),
            kind,
            fx,
            audio,
        }
    }

    fn presets_for(kind: &EngineEventKind) -> (Vec<FxPreset>, Vec<AudioPreset>) {
        match kind {
            EngineEventKind::ModeChanged { mode } => match mode {
                EngineMode::Full => (vec![FxPreset::RunePulse], vec![AudioPreset::LowDrone]),
                EngineMode::Lite => (vec![FxPreset::Glitch], vec![AudioPreset::BellStrike]),
            },
            EngineEventKind::GameModeChanged { .. } => (vec![FxPreset::Fog], vec![AudioPreset::LowDrone]),
            EngineEventKind::ParagraphRegistered { .. } => (vec![], vec![]),
            EngineEventKind::ThreatTurn { .. } => (vec![FxPreset::BloodVignette], vec![AudioPreset::ThreatPulse]),
            EngineEventKind::RollResolved { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::DiceHit]),
            EngineEventKind::ClockAdvanced { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::BellStrike]),
            EngineEventKind::MarkApplied { .. } => (vec![FxPreset::BloodVignette], vec![AudioPreset::LowDrone]),
        }
    }
}

/// Sink de eventos para integraciones (p.ej. comandos Tauri o pruebas).
pub trait EventSink: Send + Sync {
    fn emit(&self, event: &EventEnvelope);
}

/// Sink en memoria que almacena los eventos para inspección o tests.
#[derive(Clone, Default)]
pub struct MemoryEventSink {
    events: Arc<Mutex<Vec<EventEnvelope>>>,
}

impl MemoryEventSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<EventEnvelope> {
        let guard = self.events.lock().expect("lock poisoned");
        guard.clone()
    }
}

impl EventSink for MemoryEventSink {
    fn emit(&self, event: &EventEnvelope) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(event.clone());
        }
    }
}
