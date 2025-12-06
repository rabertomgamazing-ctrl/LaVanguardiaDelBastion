//! Motor base scrimdark con protocolos de juego y coordinador IA híbrido.

pub mod ai;
pub mod dice;
pub mod events;
pub mod persistence;
pub mod protocol;
pub mod session;
pub mod world;

use protocol::{EngineMode, GameMode, ProtocolConfig};
use events::{EngineEventKind, EventEnvelope, EventSink};
use session::{NarrativeMode, ParagraphOutcome, SessionManager};
use ai::architect_mode_cards;

/// Contenedor de alto nivel que une protocolos, sesión y eventos.
pub struct BastionEngine {
    protocol: ProtocolConfig,
    session: SessionManager,
    event_sink: Option<Box<dyn EventSink>>, 
}

impl BastionEngine {
    /// Inicializa el motor cargando los protocolos desde el YAML central.
    pub fn from_core_yaml(path: impl AsRef<std::path::Path>) -> Result<Self, protocol::ProtocolError> {
        let protocol = ProtocolConfig::from_core_yaml(path)?;
        let session = SessionManager::new(protocol.engine_mode());
        Ok(Self { protocol, session, event_sink: None })
    }

    /// Permite adjuntar un sink de eventos (por ejemplo, hacia Tauri o pruebas).
    pub fn with_event_sink(mut self, sink: Box<dyn EventSink>) -> Self {
        self.event_sink = Some(sink);
        self
    }

    /// Emite el menú inicial de modos A/B/C para que la UI arranque en esa pantalla.
    pub fn present_architect_modes(&self) {
        let cards = architect_mode_cards();
        self.emit(EventEnvelope::new(EngineEventKind::ArchitectModesMenu { cards }));
    }

    /// Devuelve el modo de juego activo (Partida Estándar por defecto).
    pub fn game_mode(&self) -> GameMode {
        self.protocol.game_mode()
    }

    /// Devuelve el modo narrativo activo (por defecto Acción).
    pub fn narrative_mode(&self) -> NarrativeMode {
        self.session.narrative_mode()
    }

    /// Ajusta el modo de juego únicamente si se pidió explícitamente Relato Libre.
    pub fn request_game_mode_change(&mut self, target: GameMode, explicit: bool) {
        let previous = self.protocol.game_mode();
        self.protocol.request_game_mode_change(target, explicit);
        if self.protocol.game_mode() != previous {
            self.emit(EventEnvelope::new(EngineEventKind::GameModeChanged { game_mode: self.protocol.game_mode() }));
        }
    }

    /// Cambia el modo narrativo y emite el evento correspondiente.
    pub fn set_narrative_mode(&mut self, mode: NarrativeMode) {
        if self.session.narrative_mode() != mode {
            self.session.set_narrative_mode(mode);
            self.emit(EventEnvelope::new(EngineEventKind::NarrativeModeChanged { mode }));
        }
    }

    /// Cambia el motor a modo LITE (por ejemplo, si falla un proveedor IA).
    pub fn degrade_engine(&mut self) {
        self.protocol.set_engine_mode(EngineMode::Lite);
        self.session.update_engine_mode(EngineMode::Lite);
        self.emit(EventEnvelope::new(EngineEventKind::ModeChanged { mode: EngineMode::Lite }));
    }

    /// Ajusta fama/infamia y emite el evento para la capa audiovisual o UI.
    pub fn adjust_reputation(&mut self, fame_delta: i32, infamy_delta: i32) {
        let rep = self.session.adjust_reputation(fame_delta, infamy_delta);
        self.emit(EventEnvelope::new(EngineEventKind::ReputationChanged { fame: rep.fame, infamy: rep.infamy }));
    }

    /// Ejecuta el ciclo semanal (Corte y contrabando) y emite los eventos correspondientes.
    pub fn run_weekly_events(&mut self) {
        let outcome = self.session.advance_week();
        if outcome.court_event {
            self.emit(EventEnvelope::new(EngineEventKind::CourtSummoned { week: outcome.week }));
        }
        if outcome.contraband_event {
            self.emit(EventEnvelope::new(EngineEventKind::ContrabandReport { week: outcome.week }));
        }
    }

    /// Registra un párrafo narrativo y devuelve si se activa un turno de amenaza.
    pub fn register_paragraph(&mut self) -> bool {
        let ParagraphOutcome { total_paragraphs, rest_paragraphs, mode, counted_for_threat, threat_triggered, clocks_advanced } =
            self.session.register_paragraph();
        self.emit(EventEnvelope::new(EngineEventKind::ParagraphRegistered {
            count: total_paragraphs,
            rest: rest_paragraphs,
            mode,
            counted_for_threat,
        }));
        for clock in clocks_advanced {
            self.emit(EventEnvelope::new(EngineEventKind::ClockAdvanced { clock }));
        }
        if threat_triggered {
            self.emit(EventEnvelope::new(EngineEventKind::ThreatTurn { total: self.session.threat_turns_triggered() }));
        }
        threat_triggered
    }

    /// Lee el contador de reloj de amenaza.
    pub fn threat_turns_triggered(&self) -> u32 {
        self.session.threat_turns_triggered()
    }

    /// Aplica daño/avance a una marca y emite el evento correspondiente.
    pub fn apply_mark(&mut self, index: usize, amount: u8) {
        if let Some(mark) = self.session.apply_mark(index, amount) {
            self.emit(EventEnvelope::new(EngineEventKind::MarkApplied { mark }));
        }
    }

    fn emit(&self, event: EventEnvelope) {
        if let Some(sink) = &self.event_sink {
            sink.emit(&event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MemoryEventSink;

    #[test]
    fn engine_defaults_to_standard_full() {
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml").expect("core YAML readable");
        assert_eq!(engine.game_mode(), GameMode::PartidaEstandar);
        assert!(matches!(engine.protocol.engine_mode(), EngineMode::Full));

        assert_eq!(engine.narrative_mode(), NarrativeMode::Accion);

        // No implicit freeform switches
        engine.request_game_mode_change(GameMode::RelatoLibre, false);
        assert_eq!(engine.game_mode(), GameMode::PartidaEstandar);

        engine.request_game_mode_change(GameMode::RelatoLibre, true);
        assert_eq!(engine.game_mode(), GameMode::RelatoLibre);
    }

    #[test]
    fn rest_mode_pauses_threat_turns() {
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml").expect("core YAML readable");
        engine.set_narrative_mode(NarrativeMode::Descanso);
        for _ in 0..session::PARAGRAPHS_PER_THREAT {
            assert!(!engine.register_paragraph());
        }
        assert_eq!(engine.threat_turns_triggered(), 0);

        engine.set_narrative_mode(NarrativeMode::Accion);
        for _ in 0..session::PARAGRAPHS_PER_THREAT {
            engine.register_paragraph();
        }
        assert_eq!(engine.threat_turns_triggered(), 1);
    }

    #[test]
    fn threat_turn_triggers_every_24_paragraphs() {
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml").expect("core YAML readable");
        let mut triggers = 0;
        for _ in 0..48 {
            if engine.register_paragraph() {
                triggers += 1;
            }
        }
        assert_eq!(triggers, 2);
        assert_eq!(engine.threat_turns_triggered(), 2);
    }

    #[test]
    fn event_sink_captures_sequence() {
        let sink = MemoryEventSink::new();
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml")
            .expect("core YAML readable")
            .with_event_sink(Box::new(sink.clone()));

        engine.set_narrative_mode(NarrativeMode::Viaje);
        engine.register_paragraph();
        engine.apply_mark(0, 2);
        engine.degrade_engine();

        let events = sink.events();
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::ParagraphRegistered { .. })));
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::NarrativeModeChanged { mode: NarrativeMode::Viaje })));
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::MarkApplied { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e.kind, EngineEventKind::ModeChanged { mode: EngineMode::Lite })));
    }

    #[test]
    fn architect_menu_and_reputation_events_emit() {
        let sink = MemoryEventSink::new();
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml")
            .expect("core YAML readable")
            .with_event_sink(Box::new(sink.clone()));

        engine.present_architect_modes();
        engine.adjust_reputation(5, 2);
        engine.run_weekly_events();

        let events = sink.events();
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::ArchitectModesMenu { .. })));
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::ReputationChanged { fame: 5, infamy: 2 })));
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::CourtSummoned { week: 1 })));
        assert!(events.iter().any(|e| matches!(e.kind, EngineEventKind::ContrabandReport { week: 1 })));
    }
}
