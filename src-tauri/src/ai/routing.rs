use serde::Serialize;

use super::provider;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedRoute {
    pub provider: String,
    pub model: String,
    pub routing_mode: String,
    pub reason: String,
}

pub fn select_route(
    preferred_provider: &str,
    preferred_model: &str,
    mode: &str,
    deepseek_configured: bool,
    openai_configured: bool,
) -> Result<SelectedRoute, String> {
    if preferred_provider != "auto" {
        provider::validate_provider_model(preferred_provider, preferred_model)
            .map_err(|error| error.message)?;
        return Ok(SelectedRoute {
            provider: preferred_provider.to_string(),
            model: preferred_model.to_string(),
            routing_mode: "manual".to_string(),
            reason: "You selected this provider and model for the conversation.".to_string(),
        });
    }

    if ["create_tasks", "propose_actions"].contains(&mode) && openai_configured {
        return Ok(SelectedRoute {
            provider: "openai".to_string(),
            model: "gpt-5-mini".to_string(),
            routing_mode: "auto".to_string(),
            reason: "Auto selected OpenAI for a structured proposal.".to_string(),
        });
    }
    if deepseek_configured {
        return Ok(SelectedRoute {
            provider: "deepseek".to_string(),
            model: if mode == "plan" {
                "deepseek-v4-pro"
            } else {
                "deepseek-v4-flash"
            }
            .to_string(),
            routing_mode: "auto".to_string(),
            reason: if mode == "plan" {
                "Auto selected DeepSeek V4 Pro for deeper planning."
            } else {
                "Auto selected DeepSeek V4 Flash for a fast general response."
            }
            .to_string(),
        });
    }
    if openai_configured {
        return Ok(SelectedRoute {
            provider: "openai".to_string(),
            model: "gpt-5-mini".to_string(),
            routing_mode: "auto".to_string(),
            reason: "Auto selected the configured OpenAI provider.".to_string(),
        });
    }
    Err("Auto could not find a configured AI provider. Add a provider key in Settings.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_routes_are_exact_and_validated() {
        let route = select_route("openai", "gpt-5.2", "ask", true, true).unwrap();
        assert_eq!(route.provider, "openai");
        assert_eq!(route.model, "gpt-5.2");
        assert_eq!(route.routing_mode, "manual");
        assert!(select_route("openai", "deepseek-v4-pro", "ask", true, true).is_err());
    }

    #[test]
    fn auto_is_deterministic_and_never_claims_an_unconfigured_provider() {
        assert_eq!(
            select_route("auto", "auto", "ask", true, true)
                .unwrap()
                .provider,
            "deepseek"
        );
        assert_eq!(
            select_route("auto", "auto", "create_tasks", true, true)
                .unwrap()
                .provider,
            "openai"
        );
        assert_eq!(
            select_route("auto", "auto", "ask", false, true)
                .unwrap()
                .provider,
            "openai"
        );
        assert!(select_route("auto", "auto", "ask", false, false).is_err());
    }
}
