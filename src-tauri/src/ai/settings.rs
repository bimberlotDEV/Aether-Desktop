use crate::db::repositories;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::capabilities::cloud_backend_descriptors;

const MODE: &str = "ai.routing.mode";
const LOCAL_RUNTIME: &str = "ai.routing.preferred_local_runtime";
const LOCAL_MODEL: &str = "ai.routing.preferred_local_model";
const CLOUD_PROVIDER: &str = "ai.routing.preferred_cloud_provider";
const CLOUD_MODEL: &str = "ai.routing.preferred_cloud_model";
const CLOUD_FALLBACK: &str = "ai.routing.automatic_cloud_fallback";
const DISCLOSURE: &str = "ai.privacy.cloud_disclosure_policy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    LocalOnly,
    CloudOnly,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomaticCloudFallback {
    Never,
    PromptOnly,
    AskWhenNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudDisclosurePolicy {
    AlwaysAsk,
    AskForAetherData,
    AllowExplicitAttachments,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRoutingSettings {
    pub mode: RoutingMode,
    pub preferred_local_runtime: Option<String>,
    pub preferred_local_model: Option<String>,
    pub preferred_cloud_provider: Option<String>,
    pub preferred_cloud_model: Option<String>,
    pub automatic_cloud_fallback: AutomaticCloudFallback,
    pub cloud_disclosure_policy: CloudDisclosurePolicy,
}

impl Default for AiRoutingSettings {
    fn default() -> Self {
        Self {
            mode: RoutingMode::Automatic,
            preferred_local_runtime: None,
            preferred_local_model: None,
            preferred_cloud_provider: None,
            preferred_cloud_model: None,
            automatic_cloud_fallback: AutomaticCloudFallback::AskWhenNeeded,
            cloud_disclosure_policy: CloudDisclosurePolicy::AskForAetherData,
        }
    }
}

pub fn get(conn: &Connection) -> Result<AiRoutingSettings, String> {
    let defaults = AiRoutingSettings::default();
    Ok(AiRoutingSettings {
        mode: read_enum(conn, MODE)?.unwrap_or(defaults.mode),
        preferred_local_runtime: read_optional(conn, LOCAL_RUNTIME)?,
        preferred_local_model: read_optional(conn, LOCAL_MODEL)?,
        preferred_cloud_provider: read_optional(conn, CLOUD_PROVIDER)?,
        preferred_cloud_model: read_optional(conn, CLOUD_MODEL)?,
        automatic_cloud_fallback: read_enum(conn, CLOUD_FALLBACK)?
            .unwrap_or(defaults.automatic_cloud_fallback),
        cloud_disclosure_policy: read_enum(conn, DISCLOSURE)?
            .unwrap_or(defaults.cloud_disclosure_policy),
    })
}

pub fn set(conn: &Connection, value: &AiRoutingSettings) -> Result<AiRoutingSettings, String> {
    validate(value)?;
    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| format!("AI settings transaction error: {error}"))?;
    write_enum(&transaction, MODE, value.mode)?;
    write_optional(
        &transaction,
        LOCAL_RUNTIME,
        value.preferred_local_runtime.as_deref(),
    )?;
    write_optional(
        &transaction,
        LOCAL_MODEL,
        value.preferred_local_model.as_deref(),
    )?;
    write_optional(
        &transaction,
        CLOUD_PROVIDER,
        value.preferred_cloud_provider.as_deref(),
    )?;
    write_optional(
        &transaction,
        CLOUD_MODEL,
        value.preferred_cloud_model.as_deref(),
    )?;
    write_enum(&transaction, CLOUD_FALLBACK, value.automatic_cloud_fallback)?;
    write_enum(&transaction, DISCLOSURE, value.cloud_disclosure_policy)?;
    transaction
        .commit()
        .map_err(|error| format!("AI settings commit error: {error}"))?;
    get(conn)
}

fn validate(value: &AiRoutingSettings) -> Result<(), String> {
    if value.preferred_local_runtime.is_some() || value.preferred_local_model.is_some() {
        return Err("No local AI runtime is available in this version of Aether.".into());
    }
    match (
        &value.preferred_cloud_provider,
        &value.preferred_cloud_model,
    ) {
        (None, None) => Ok(()),
        (Some(provider), Some(model))
            if cloud_backend_descriptors().iter().any(|backend| {
                backend.id == *provider
                    && backend
                        .models
                        .iter()
                        .any(|candidate| candidate.id == *model)
            }) =>
        {
            Ok(())
        }
        (Some(_), Some(_)) => {
            Err("The preferred cloud provider and model are not registered.".into())
        }
        _ => Err("Preferred cloud provider and model must be set together.".into()),
    }
}

fn read_optional(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    Ok(repositories::settings::get(conn, key)?
        .map(|setting| setting.value)
        .filter(|value| !value.is_empty()))
}

fn read_enum<T: for<'de> Deserialize<'de>>(
    conn: &Connection,
    key: &str,
) -> Result<Option<T>, String> {
    repositories::settings::get(conn, key)?
        .map(|setting| {
            serde_json::from_str(&setting.value)
                .map_err(|_| format!("Invalid stored AI setting: {key}"))
        })
        .transpose()
}

fn write_optional(conn: &Connection, key: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        repositories::settings::set(conn, key, value, "string")
    } else {
        repositories::settings::delete(conn, key).map(|_| ())
    }
}

fn write_enum<T: Serialize>(conn: &Connection, key: &str, value: T) -> Result<(), String> {
    let encoded = serde_json::to_string(&value)
        .map_err(|error| format!("AI setting serialization error: {error}"))?;
    repositories::settings::set(conn, key, &encoded, "enum")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    #[test]
    fn defaults_and_valid_settings_round_trip_without_credentials() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        assert_eq!(get(&conn).unwrap(), AiRoutingSettings::default());
        let settings = AiRoutingSettings {
            mode: RoutingMode::CloudOnly,
            preferred_cloud_provider: Some("openai".into()),
            preferred_cloud_model: Some("gpt-5-mini".into()),
            cloud_disclosure_policy: CloudDisclosurePolicy::AllowExplicitAttachments,
            ..AiRoutingSettings::default()
        };
        assert_eq!(set(&conn, &settings).unwrap(), settings);
        let keys: Vec<String> = repositories::settings::list(&conn)
            .unwrap()
            .into_iter()
            .map(|item| item.key)
            .collect();
        assert!(keys
            .iter()
            .all(|key| !key.contains("credential") && !key.contains("api_key")));
    }

    #[test]
    fn rejects_unknown_models_and_unimplemented_local_preferences() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        let local = AiRoutingSettings {
            preferred_local_runtime: Some("ollama".into()),
            ..AiRoutingSettings::default()
        };
        assert!(set(&conn, &local).is_err());
        let unknown_cloud = AiRoutingSettings {
            preferred_cloud_provider: Some("openai".into()),
            preferred_cloud_model: Some("unknown".into()),
            ..AiRoutingSettings::default()
        };
        assert!(set(&conn, &unknown_cloud).is_err());
    }
}
