use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngineMode {
    Full,
    Lite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameMode {
    PartidaEstandar,
    RelatoLibre,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CoreYaml {
    #[serde(default = "EngineMode::default_full")]
    pub engine_mode: EngineMode,
    #[serde(default = "GameMode::default_standard")]
    pub game_mode: GameMode,
}

impl EngineMode {
    pub fn default_full() -> Self {
        EngineMode::Full
    }
}

impl Default for EngineMode {
    fn default() -> Self {
        Self::Full
    }
}

impl GameMode {
    pub fn default_standard() -> Self {
        GameMode::PartidaEstandar
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self::PartidaEstandar
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("No se pudo leer el YAML base: {0}")]
    Io(#[from] std::io::Error),
    #[error("No se pudo parsear el YAML base: {0}")]
    Parse(#[from] serde_yaml::Error),
}

/// Configuración de protocolos cargada desde el YAML raíz.
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    engine_mode: EngineMode,
    game_mode: GameMode,
    source_path: String,
}

impl ProtocolConfig {
    pub fn from_core_yaml(path: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let path_ref = path.as_ref();
        let contents = fs::read_to_string(path_ref)?;
        let core: CoreYaml = serde_yaml::from_str(&contents).unwrap_or_default();
        Ok(Self {
            engine_mode: core.engine_mode,
            game_mode: core.game_mode,
            source_path: path_ref.display().to_string(),
        })
    }

    pub fn engine_mode(&self) -> EngineMode {
        self.engine_mode
    }

    pub fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    /// Solo permite activar Relato Libre si es una solicitud explícita.
    pub fn request_game_mode_change(&mut self, target: GameMode, explicit: bool) {
        if target == GameMode::RelatoLibre && !explicit {
            return;
        }
        self.game_mode = target;
    }

    /// Permite cambiar el modo de motor en caliente (p.ej. fallos de IA).
    pub fn set_engine_mode(&mut self, mode: EngineMode) {
        self.engine_mode = mode;
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_modes_fallback() {
        let config = ProtocolConfig::from_core_yaml("BastionLAN_Core.yaml").expect("yaml readable");
        assert!(matches!(config.engine_mode(), EngineMode::Full));
        assert!(matches!(config.game_mode(), GameMode::PartidaEstandar));
    }

    #[test]
    fn explicit_freeform_switch_required() {
        let mut config = ProtocolConfig::from_core_yaml("BastionLAN_Core.yaml").expect("yaml readable");
        config.request_game_mode_change(GameMode::RelatoLibre, false);
        assert_eq!(config.game_mode(), GameMode::PartidaEstandar);
        config.request_game_mode_change(GameMode::RelatoLibre, true);
        assert_eq!(config.game_mode(), GameMode::RelatoLibre);
    }
}
