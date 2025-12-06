//! Motor base scrimdark con protocolos de juego y coordinador IA híbrido.

pub mod ai;
pub mod dice;
pub mod events;
pub mod persistence;
pub mod protocol;
pub mod session;

use protocol::{EngineMode, GameMode, ProtocolConfig};
use session::SessionManager;

/// Contenedor de alto nivel que une protocolos, sesión y eventos.
pub struct BastionEngine {
    protocol: ProtocolConfig,
    session: SessionManager,
}

impl BastionEngine {
    /// Inicializa el motor cargando los protocolos desde el YAML central.
    pub fn from_core_yaml(path: impl AsRef<std::path::Path>) -> Result<Self, protocol::ProtocolError> {
        let protocol = ProtocolConfig::from_core_yaml(path)?;
        let session = SessionManager::new(protocol.engine_mode());
        Ok(Self { protocol, session })
    }

    /// Devuelve el modo de juego activo (Partida Estándar por defecto).
    pub fn game_mode(&self) -> GameMode {
        self.protocol.game_mode()
    }

    /// Ajusta el modo de juego únicamente si se pidió explícitamente Relato Libre.
    pub fn request_game_mode_change(&mut self, target: GameMode, explicit: bool) {
        self.protocol.request_game_mode_change(target, explicit);
    }

    /// Cambia el motor a modo LITE (por ejemplo, si falla un proveedor IA).
    pub fn degrade_engine(&mut self) {
        self.protocol.set_engine_mode(EngineMode::Lite);
        self.session.update_engine_mode(EngineMode::Lite);
    }

    /// Registra un párrafo narrativo y devuelve si se activa un turno de amenaza.
    pub fn register_paragraph(&mut self) -> bool {
        self.session.register_paragraph()
    }

    /// Lee el contador de reloj de amenaza.
    pub fn threat_turns_triggered(&self) -> u32 {
        self.session.threat_turns_triggered()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_defaults_to_standard_full() {
        let mut engine = BastionEngine::from_core_yaml("BastionLAN_Core.yaml").expect("core YAML readable");
        assert_eq!(engine.game_mode(), GameMode::PartidaEstandar);
        assert!(matches!(engine.protocol.engine_mode(), EngineMode::Full));

        // No implicit freeform switches
        engine.request_game_mode_change(GameMode::RelatoLibre, false);
        assert_eq!(engine.game_mode(), GameMode::PartidaEstandar);

        engine.request_game_mode_change(GameMode::RelatoLibre, true);
        assert_eq!(engine.game_mode(), GameMode::RelatoLibre);
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
}
