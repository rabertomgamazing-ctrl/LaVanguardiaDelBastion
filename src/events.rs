use crate::dice::RollResult;
use crate::protocol::{EngineMode, GameMode};
use crate::session::{Clock, MarkTrack, NarrativeMode};
use crate::ai::ArchitectModeCard;
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
    CourtChant,
    ContrabandWhisper,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AntiNoiseProfile {
    None,
    LowPass,
    Gate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineEventKind {
    ModeChanged { mode: EngineMode },
    GameModeChanged { game_mode: GameMode },
    NarrativeModeChanged { mode: NarrativeMode },
    ArchitectModesMenu { cards: Vec<ArchitectModeCard> },
    ParagraphRegistered { count: u32, rest: u32, mode: NarrativeMode, counted_for_threat: bool },
    ThreatTurn { total: u32 },
    RollResolved { roll: RollResult },
    ClockAdvanced { clock: Clock },
    MarkApplied { mark: MarkTrack },
    ReputationChanged { fame: i32, infamy: i32 },
    CourtSummoned { week: u32 },
    ContrabandReport { week: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub at: DateTime<Utc>,
    pub kind: EngineEventKind,
    pub fx: Vec<FxPreset>,
    pub audio: Vec<AudioPreset>,
    pub anti_noise: AntiNoiseProfile,
}

impl EventEnvelope {
    pub fn new(kind: EngineEventKind) -> Self {
        let (fx, audio) = Self::presets_for(&kind);
        let anti_noise = Self::anti_noise_for(&kind);
        Self {
            at: Utc::now(),
            kind,
            fx,
            audio,
            anti_noise,
        }
    }

    fn presets_for(kind: &EngineEventKind) -> (Vec<FxPreset>, Vec<AudioPreset>) {
        match kind {
            EngineEventKind::ModeChanged { mode } => match mode {
                EngineMode::Full => (vec![FxPreset::RunePulse], vec![AudioPreset::LowDrone]),
                EngineMode::Lite => (vec![FxPreset::Glitch], vec![AudioPreset::BellStrike]),
            },
            EngineEventKind::GameModeChanged { .. } => (vec![FxPreset::Fog], vec![AudioPreset::LowDrone]),
            EngineEventKind::NarrativeModeChanged { mode } => match mode {
                NarrativeMode::Accion => (vec![FxPreset::RunePulse], vec![AudioPreset::LowDrone]),
                NarrativeMode::Descanso => (vec![FxPreset::Fog], vec![AudioPreset::LowDrone]),
                NarrativeMode::Viaje => (vec![FxPreset::RunePulse], vec![AudioPreset::BellStrike]),
                NarrativeMode::Libre => (vec![FxPreset::Fog], vec![AudioPreset::DiceHit]),
            },
            EngineEventKind::ArchitectModesMenu { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::LowDrone]),
            EngineEventKind::ParagraphRegistered { .. } => (vec![], vec![]),
            EngineEventKind::ThreatTurn { .. } => (vec![FxPreset::BloodVignette], vec![AudioPreset::ThreatPulse]),
            EngineEventKind::RollResolved { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::DiceHit]),
            EngineEventKind::ClockAdvanced { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::BellStrike]),
            EngineEventKind::MarkApplied { .. } => (vec![FxPreset::BloodVignette], vec![AudioPreset::LowDrone]),
            EngineEventKind::ReputationChanged { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::LowDrone]),
            EngineEventKind::CourtSummoned { .. } => (vec![FxPreset::RunePulse], vec![AudioPreset::CourtChant]),
            EngineEventKind::ContrabandReport { .. } => (vec![FxPreset::Glitch], vec![AudioPreset::ContrabandWhisper]),
        }
    }

    fn anti_noise_for(kind: &EngineEventKind) -> AntiNoiseProfile {
        match kind {
            EngineEventKind::ThreatTurn { .. }
            | EngineEventKind::ClockAdvanced { .. }
            | EngineEventKind::CourtSummoned { .. } => AntiNoiseProfile::LowPass,
            EngineEventKind::ContrabandReport { .. } | EngineEventKind::ModeChanged { .. } => AntiNoiseProfile::Gate,
            _ => AntiNoiseProfile::None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_sink_stores_events() {
        let sink = MemoryEventSink::new();
        let event = EventEnvelope::new(EngineEventKind::ModeChanged { mode: EngineMode::Full });
        sink.emit(&event);

        let events = sink.events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].kind, EngineEventKind::ModeChanged { .. }));
    }

    #[test]
    fn anti_noise_profiles_are_applied() {
        let threat = EventEnvelope::new(EngineEventKind::ThreatTurn { total: 1 });
        assert_eq!(threat.anti_noise, AntiNoiseProfile::LowPass);

        let contraband = EventEnvelope::new(EngineEventKind::ContrabandReport { week: 1 });
        assert_eq!(contraband.anti_noise, AntiNoiseProfile::Gate);
    }
}
