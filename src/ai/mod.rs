pub mod providers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use providers::{Provider, ProviderPlan};
use crate::session::PARAGRAPHS_PER_THREAT;

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

/// Datos base para el rol A (Narrador) al arrancar una campaña.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModeAOnboarding {
    /// Mensaje de bienvenida en tono scrimdark.
    pub intro: String,
    /// Sello del Bastión (firma que el Narrador recita para anclar el canon).
    pub bastion_signature: String,
    /// Límite de párrafos narrativos por turno antes del Turno de Amenaza.
    pub paragraph_limit: u32,
    /// Misión de tutorial obligatoria para presentar reglas y ritmo.
    pub tutorial_mission: String,
}

impl ModeAOnboarding {
    pub fn default() -> Self {
        Self {
            intro: "Bienvenido al Último Bastión. La bruma huele a hierro mojado y las campanas recuerdan que no hay retorno.".to_string(),
            bastion_signature: "Firma del Bastión: Kaelen Barbagris mantiene el mando y ninguna victoria trivial compra el trono.".to_string(),
            paragraph_limit: PARAGRAPHS_PER_THREAT,
            tutorial_mission: "Tutorial: escoltar un convoy de suministros desde la Corte hasta la Puerta Roja, con tiradas visibles de 1d20+Rango y consecuencias en Relojes.".to_string(),
        }
    }

    /// Prompts para reforzar el rol A del Narrador con las restricciones clave.
    pub fn as_prompts(&self) -> Vec<IaPrompt> {
        vec![
            IaPrompt {
                role: IaRole::NarradorA,
                content: format!(
                    "{}\n{}\nRegla de tiempo: máximo {} párrafos narrativos antes del Turno de Amenaza. Ejecuta primero la misión de tutorial: {}",
                    self.intro, self.bastion_signature, self.paragraph_limit, self.tutorial_mission
                ),
            },
            IaPrompt {
                role: IaRole::JuezB,
                content: "Asegura que cada escena tenga tiradas o efectos mecánicos; evita derivar a novela pura sin dados.".to_string(),
            },
        ]
    }
}

/// Tarjetas para el selector inicial de modos A/B/C.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectModeCard {
    pub role: IaRole,
    pub headline: String,
    pub borrows_from: Vec<IaRole>,
    pub primary_mechanics: Vec<String>,
}

pub fn architect_mode_cards() -> Vec<ArchitectModeCard> {
    vec![
        ArchitectModeCard {
            role: IaRole::NarradorA,
            headline: "Modo A: Narrador".to_string(),
            borrows_from: vec![IaRole::JuezB, IaRole::SimuladorC],
            primary_mechanics: vec![
                "Tono scrimdark y 24 párrafos antes del Turno de Amenaza".to_string(),
                "Tiradas visibles y tutorial obligatorio".to_string(),
                "Activa FX de campanas/viñeta y respeta descansos".to_string(),
            ],
        },
        ArchitectModeCard {
            role: IaRole::JuezB,
            headline: "Modo B: Juez".to_string(),
            borrows_from: vec![IaRole::NarradorA, IaRole::SimuladorC],
            primary_mechanics: vec![
                "Valida reglas y aplica dados/rangos".to_string(),
                "Convoca eventos de Corte y contrabando semanal".to_string(),
                "Puede solicitar apoyo narrativo si faltan descripciones".to_string(),
            ],
        },
        ArchitectModeCard {
            role: IaRole::SimuladorC,
            headline: "Modo C: Simulador".to_string(),
            borrows_from: vec![IaRole::NarradorA, IaRole::JuezB],
            primary_mechanics: vec![
                "Propaga relojes/marcas y rutas de viaje".to_string(),
                "Escala fama/infamia según decisiones".to_string(),
                "Pide resolución de tiradas si necesita juicio".to_string(),
            ],
        },
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_a_onboarding_includes_paragraph_limit_and_tutorial() {
        let onboarding = ModeAOnboarding::default();
        assert_eq!(onboarding.paragraph_limit, PARAGRAPHS_PER_THREAT);
        let prompts = onboarding.as_prompts();
        assert!(!prompts.is_empty());
        let narrator_prompt = &prompts[0].content;
        assert!(narrator_prompt.contains("Turno de Amenaza"));
        assert!(narrator_prompt.contains("Tutorial"));
    }

    #[test]
    fn architect_cards_link_modes() {
        let cards = architect_mode_cards();
        assert_eq!(cards.len(), 3);
        assert!(cards.iter().any(|c| c.role == IaRole::NarradorA && c.borrows_from.contains(&IaRole::JuezB)));
        assert!(cards.iter().any(|c| c.role == IaRole::JuezB && c.primary_mechanics.iter().any(|m| m.contains("Corte"))));
    }
}
