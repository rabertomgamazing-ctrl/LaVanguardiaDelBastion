use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{IaPrompt, IaResponse, IaRole};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provider {
    Ollama,
    Gemini,
    ExternalApi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderPlan {
    pub summary: String,
    pub steps: Vec<String>,
}

fn prompt_for_role(prompts: &[IaPrompt], role: IaRole) -> String {
    prompts
        .iter()
        .find(|p| p.role == role)
        .map(|p| p.content.clone())
        .unwrap_or_else(|| "".to_string())
}

pub fn plan_with_gemini(prompts: &[IaPrompt]) -> Option<ProviderPlan> {
    let content = prompt_for_role(prompts, IaRole::JuezB);
    if content.is_empty() {
        return None;
    }
    Some(ProviderPlan {
        summary: "Plan tático scrimdark consolidado".to_string(),
        steps: vec![
            "Validar reglas con Códice".to_string(),
            format!("Aplicar entrada del juez: {content}"),
            "Emitir triggers para audio/FX".to_string(),
        ],
    })
}

pub fn narrate_with_ollama(prompts: &[IaPrompt]) -> Option<IaResponse> {
    let content = prompt_for_role(prompts, IaRole::NarradorA);
    if content.is_empty() {
        return None;
    }
    Some(IaResponse {
        provider: Provider::Ollama,
        role: IaRole::NarradorA,
        content: format!(
            "[Narrador scrimdark] {content}\nTono: campanas, penumbra, runas que palpitan."
        ),
        timestamp: Utc::now(),
    })
}

pub fn validate_with_gemini(prompts: &[IaPrompt]) -> Option<IaResponse> {
    let content = prompt_for_role(prompts, IaRole::JuezB);
    if content.is_empty() {
        return None;
    }
    Some(IaResponse {
        provider: Provider::Gemini,
        role: IaRole::JuezB,
        content: format!("[Validación de reglas] {content}"),
        timestamp: Utc::now(),
    })
}

pub fn simulate_with_external(prompts: &[IaPrompt]) -> Option<IaResponse> {
    let content = prompt_for_role(prompts, IaRole::SimuladorC);
    if content.is_empty() {
        return None;
    }
    Some(IaResponse {
        provider: Provider::ExternalApi,
        role: IaRole::SimuladorC,
        content: format!("[Mundo rúnico] {content}"),
        timestamp: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_requires_judge_prompt() {
        let prompts = vec![IaPrompt {
            role: IaRole::JuezB,
            content: "Resolver tirada".to_string(),
        }];
        let plan = plan_with_gemini(&prompts).unwrap();
        assert!(plan.summary.contains("Plan tático"));
    }
}
