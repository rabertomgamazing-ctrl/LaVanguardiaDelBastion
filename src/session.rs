use crate::protocol::EngineMode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PARAGRAPHS_PER_THREAT: u32 = 24;

/// Submodos narrativos que afectan el conteo de párrafos y el ritmo de amenaza.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum NarrativeMode {
    /// Escena de misión o relato activo: cuenta para el Turno de Amenaza.
    Accion,
    /// Tiempo de respiro en el Bastión: no avanza el reloj de párrafos.
    Descanso,
    /// Viaje entre nodos rúnicos o coronas de distancia.
    Viaje,
    /// Escena libre solicitada explícitamente por jugadores.
    Libre,
}

impl NarrativeMode {
    pub fn counts_for_threat(&self) -> bool {
        matches!(self, NarrativeMode::Accion | NarrativeMode::Viaje)
    }
}

/// Fama/Infamia acumulada por la campaña.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Default)]
pub struct Reputation {
    pub fame: i32,
    pub infamy: i32,
}

impl Reputation {
    pub fn adjust(&mut self, fame_delta: i32, infamy_delta: i32) {
        self.fame = (self.fame + fame_delta).max(0);
        self.infamy = (self.infamy + infamy_delta).max(0);
    }
}

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
    rest_paragraphs: u32,
    threat_triggers: u32,
    last_threat_at: Option<DateTime<Utc>>,
    narrative_mode: NarrativeMode,
    weeks_elapsed: u32,
    reputation: Reputation,
    pub clocks: Vec<Clock>,
    pub marks: Vec<MarkTrack>,
}

/// Resultado de registrar un párrafo narrativo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParagraphOutcome {
    pub total_paragraphs: u32,
    pub rest_paragraphs: u32,
    pub mode: NarrativeMode,
    pub counted_for_threat: bool,
    pub threat_triggered: bool,
    pub clocks_advanced: Vec<Clock>,
}

impl SessionManager {
    pub fn new(engine_mode: EngineMode) -> Self {
        Self {
            engine_mode,
            paragraphs: 0,
            rest_paragraphs: 0,
            threat_triggers: 0,
            last_threat_at: None,
            narrative_mode: NarrativeMode::Accion,
            weeks_elapsed: 0,
            reputation: Reputation::default(),
            clocks: vec![Clock::new("Reloj de Amenaza", 8)],
            marks: vec![MarkTrack::new("Marcas del Bastión", 24)],
        }
    }

    pub fn update_engine_mode(&mut self, mode: EngineMode) {
        self.engine_mode = mode;
    }

    /// Registra un párrafo; si alcanza 24 activa Turno de Amenaza.
    pub fn register_paragraph(&mut self) -> ParagraphOutcome {
        let counted_for_threat = self.narrative_mode.counts_for_threat();
        if counted_for_threat {
            self.paragraphs += 1;
        } else {
            self.rest_paragraphs += 1;
        }
        let mut threat_triggered = false;
        let mut clocks_advanced = Vec::new();

        if counted_for_threat && self.paragraphs % PARAGRAPHS_PER_THREAT == 0 {
            threat_triggered = true;
            self.threat_triggers += 1;
            self.last_threat_at = Some(Utc::now());
            for clock in self.clocks.iter_mut() {
                clock.tick();
                clocks_advanced.push(clock.clone());
            }
        }

        ParagraphOutcome {
            total_paragraphs: self.paragraphs,
            rest_paragraphs: self.rest_paragraphs,
            mode: self.narrative_mode,
            counted_for_threat,
            threat_triggered,
            clocks_advanced,
        }
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

    pub fn narrative_mode(&self) -> NarrativeMode {
        self.narrative_mode
    }

    pub fn reputation(&self) -> Reputation {
        self.reputation
    }

    pub fn adjust_reputation(&mut self, fame_delta: i32, infamy_delta: i32) -> Reputation {
        self.reputation.adjust(fame_delta, infamy_delta);
        self.reputation
    }

    pub fn advance_week(&mut self) -> WeeklyOutcome {
        self.weeks_elapsed += 1;
        WeeklyOutcome {
            week: self.weeks_elapsed,
            court_event: true,
            contraband_event: true,
        }
    }

    /// Permite cambiar el modo narrativo (Acción, Descanso, Viaje o Libre).
    pub fn set_narrative_mode(&mut self, mode: NarrativeMode) {
        self.narrative_mode = mode;
    }

    /// Avanza una marca específica y devuelve su estado para emitir eventos.
    pub fn apply_mark(&mut self, index: usize, amount: u8) -> Option<MarkTrack> {
        let track = self.marks.get_mut(index)?;
        track.apply(amount);
        Some(track.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeeklyOutcome {
    pub week: u32,
    pub court_event: bool,
    pub contraband_event: bool,
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

    #[test]
    fn outcome_includes_clocks() {
        let mut session = SessionManager::new(EngineMode::Full);
        for _ in 0..(PARAGRAPHS_PER_THREAT - 1) {
            let outcome = session.register_paragraph();
            assert!(!outcome.threat_triggered);
        }

        let outcome = session.register_paragraph();
        assert!(outcome.threat_triggered);
        assert_eq!(outcome.total_paragraphs, PARAGRAPHS_PER_THREAT);
        assert_eq!(outcome.clocks_advanced.len(), session.clocks.len());
    }

    #[test]
    fn apply_mark_returns_updated_track() {
        let mut session = SessionManager::new(EngineMode::Full);
        let updated = session.apply_mark(0, 3).expect("mark exists");
        assert_eq!(updated.filled, 3);
        assert_eq!(session.marks[0].filled, 3);
    }

    #[test]
    fn rest_mode_pauses_threat_counter() {
        let mut session = SessionManager::new(EngineMode::Full);
        session.set_narrative_mode(NarrativeMode::Descanso);
        for _ in 0..PARAGRAPHS_PER_THREAT {
            let outcome = session.register_paragraph();
            assert_eq!(outcome.mode, NarrativeMode::Descanso);
            assert!(!outcome.counted_for_threat);
            assert!(!outcome.threat_triggered);
        }
        assert_eq!(session.threat_turns_triggered(), 0);
        assert_eq!(session.paragraphs(), 0);
        assert_eq!(session.rest_paragraphs, PARAGRAPHS_PER_THREAT);
    }

    #[test]
    fn reputation_and_weekly_events_progress() {
        let mut session = SessionManager::new(EngineMode::Full);
        let rep = session.adjust_reputation(3, 1);
        assert_eq!(rep.fame, 3);
        assert_eq!(rep.infamy, 1);

        let week = session.advance_week();
        assert_eq!(week.week, 1);
        assert!(week.court_event);
        assert!(week.contraband_event);
    }
}
