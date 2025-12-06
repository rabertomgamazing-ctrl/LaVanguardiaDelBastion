pub mod providers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use providers::{Provider, ProviderPlan};

/// Rol del Arquitecto IA.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IaRole {
    NarradorA,
    JuezB,
    SimuladorC,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IaPrompt {
    pub role: IaRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IaResponse {
    pub provider: Provider,
    pub role: IaRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoordinatedTurn {
    pub plan: ProviderPlan,
    pub narration: IaResponse,
    pub validation: IaResponse,
    pub world_state: IaResponse,
}

/// Orquesta proveedores para producir una respuesta compuesta.
#[derive(Default)]
pub struct AiCoordinator;

impl AiCoordinator {
    pub fn run_turn(&self, prompts: &[IaPrompt]) -> Option<CoordinatedTurn> {
        let plan = providers::plan_with_gemini(prompts)?;
        let narration = providers::narrate_with_ollama(prompts)?;
        let validation = providers::validate_with_gemini(prompts)?;
        let world_state = providers::simulate_with_external(prompts)?;
        Some(CoordinatedTurn {
            plan,
            narration,
            validation,
            world_state,
        })
    }
}
