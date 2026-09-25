use std::{collections::HashMap, sync::Mutex};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    capabilities::ExecutionLocation,
    settings::{CloudDisclosurePolicy, RoutingMode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    General,
    Personal,
    Sensitive,
    Prohibited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCategory {
    pub category: String,
    pub data_class: DataClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPolicySnapshot {
    pub highest_class: DataClass,
    pub categories: Vec<DataCategory>,
    pub cloud_permitted: bool,
    pub cloud_approval_required: bool,
    pub cloud_approval_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyDecision {
    OnDevice,
    StandingCloudConsent,
    ExplicitAttachmentConsent,
    ApprovedCloudDisclosure,
}

pub fn classify(has_explicit_aether_context: bool) -> Vec<DataCategory> {
    let mut categories = vec![DataCategory {
        category: "prompt_and_history".into(),
        data_class: DataClass::Personal,
    }];
    if has_explicit_aether_context {
        categories.push(DataCategory {
            category: "explicit_aether_context".into(),
            data_class: DataClass::Sensitive,
        });
    }
    categories
}

pub fn snapshot(
    categories: Vec<DataCategory>,
    mode: RoutingMode,
    policy: CloudDisclosurePolicy,
    approval_present: bool,
) -> DataPolicySnapshot {
    let highest_class = categories
        .iter()
        .map(|item| item.data_class)
        .max()
        .unwrap_or(DataClass::General);
    let prohibited = highest_class == DataClass::Prohibited;
    let sensitive = highest_class >= DataClass::Sensitive;
    let cloud_approval_required = !prohibited
        && match policy {
            CloudDisclosurePolicy::AlwaysAsk => true,
            CloudDisclosurePolicy::AskForAetherData => sensitive,
            CloudDisclosurePolicy::AllowExplicitAttachments => false,
        };
    let standing_cloud_only = mode == RoutingMode::CloudOnly && !sensitive;
    let cloud_permitted =
        !prohibited && (standing_cloud_only || !cloud_approval_required || approval_present);
    DataPolicySnapshot {
        highest_class,
        categories,
        cloud_permitted,
        cloud_approval_required,
        cloud_approval_present: approval_present,
    }
}

pub fn decision(
    location: ExecutionLocation,
    policy: CloudDisclosurePolicy,
    snapshot: &DataPolicySnapshot,
) -> Option<PrivacyDecision> {
    if location == ExecutionLocation::OnDevice {
        return Some(PrivacyDecision::OnDevice);
    }
    if !snapshot.cloud_permitted {
        return None;
    }
    if snapshot.cloud_approval_present {
        Some(PrivacyDecision::ApprovedCloudDisclosure)
    } else if policy == CloudDisclosurePolicy::AllowExplicitAttachments
        && snapshot.highest_class >= DataClass::Sensitive
    {
        Some(PrivacyDecision::ExplicitAttachmentConsent)
    } else {
        Some(PrivacyDecision::StandingCloudConsent)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Phase 1 approval foundation; UI issuance is intentionally deferred.
struct ApprovalGrant {
    request_hash: String,
    backend_id: String,
    model_id: String,
    category_hash: String,
    expires_at: DateTime<Utc>,
}

#[derive(Default)]
#[allow(dead_code)] // Kept separate from Safe Actions for the future disclosure UI.
pub struct DisclosureApprovalRuntime {
    grants: Mutex<HashMap<String, ApprovalGrant>>,
}

#[allow(dead_code)]
impl DisclosureApprovalRuntime {
    pub fn issue(
        &self,
        request_hash: &str,
        backend_id: &str,
        model_id: &str,
        categories: &[DataCategory],
        future_tool_result_categories: &[DataCategory],
    ) -> Result<String, String> {
        let token = Uuid::now_v7().to_string();
        let grant = ApprovalGrant {
            request_hash: request_hash.into(),
            backend_id: backend_id.into(),
            model_id: model_id.into(),
            category_hash: category_hash(categories, future_tool_result_categories),
            expires_at: Utc::now() + Duration::minutes(5),
        };
        self.grants
            .lock()
            .map_err(|_| "Cloud disclosure approvals are unavailable.".to_string())?
            .insert(token.clone(), grant);
        Ok(token)
    }

    pub fn consume(
        &self,
        token: &str,
        request_hash: &str,
        backend_id: &str,
        model_id: &str,
        categories: &[DataCategory],
        future_tool_result_categories: &[DataCategory],
    ) -> Result<(), String> {
        let grant = self
            .grants
            .lock()
            .map_err(|_| "Cloud disclosure approvals are unavailable.".to_string())?
            .remove(token)
            .ok_or_else(|| {
                "Cloud disclosure approval is invalid or was already used.".to_string()
            })?;
        if grant.expires_at < Utc::now() {
            return Err("Cloud disclosure approval expired.".into());
        }
        if grant.request_hash != request_hash
            || grant.backend_id != backend_id
            || grant.model_id != model_id
            || grant.category_hash != category_hash(categories, future_tool_result_categories)
        {
            return Err("Cloud disclosure approval does not match this request.".into());
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn request_hash(parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

#[allow(dead_code)]
fn category_hash(
    categories: &[DataCategory],
    future_tool_result_categories: &[DataCategory],
) -> String {
    let encoded =
        serde_json::to_string(&(categories, future_tool_result_categories)).unwrap_or_default();
    request_hash(&[&encoded])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_classification_cannot_be_lowered_and_prohibited_never_allows_cloud() {
        assert_eq!(
            classify(true).last().unwrap().data_class,
            DataClass::Sensitive
        );
        let snapshot = snapshot(
            vec![DataCategory {
                category: "credential".into(),
                data_class: DataClass::Prohibited,
            }],
            RoutingMode::CloudOnly,
            CloudDisclosurePolicy::AllowExplicitAttachments,
            true,
        );
        assert!(!snapshot.cloud_permitted);
    }

    #[test]
    fn sensitive_aether_context_requires_policy_or_matching_one_time_approval() {
        let categories = classify(true);
        let blocked = snapshot(
            categories.clone(),
            RoutingMode::Automatic,
            CloudDisclosurePolicy::AskForAetherData,
            false,
        );
        assert!(!blocked.cloud_permitted);
        let runtime = DisclosureApprovalRuntime::default();
        let token = runtime
            .issue("request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap();
        runtime
            .consume(&token, "request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap();
        assert!(runtime
            .consume(&token, "request", "openai", "gpt-5-mini", &categories, &[])
            .is_err());
        let altered = runtime
            .issue("request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap();
        assert!(runtime
            .consume(
                &altered,
                "changed",
                "openai",
                "gpt-5-mini",
                &categories,
                &[]
            )
            .is_err());
        let altered_tools = runtime
            .issue("request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap();
        assert!(runtime
            .consume(
                &altered_tools,
                "request",
                "openai",
                "gpt-5-mini",
                &categories,
                &[DataCategory {
                    category: "future_tool_result".into(),
                    data_class: DataClass::Sensitive,
                }],
            )
            .is_err());
    }

    #[test]
    fn expired_approval_is_rejected() {
        let categories = classify(true);
        let runtime = DisclosureApprovalRuntime::default();
        let token = runtime
            .issue("request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap();
        runtime
            .grants
            .lock()
            .unwrap()
            .get_mut(&token)
            .unwrap()
            .expires_at = Utc::now() - Duration::seconds(1);
        assert!(runtime
            .consume(&token, "request", "openai", "gpt-5-mini", &categories, &[])
            .unwrap_err()
            .contains("expired"));
    }

    #[test]
    fn provenance_snapshot_contains_categories_not_content() {
        let snapshot = snapshot(
            classify(true),
            RoutingMode::Automatic,
            CloudDisclosurePolicy::AllowExplicitAttachments,
            false,
        );
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("explicit_aether_context"));
        assert!(!json.contains("secret prompt"));
    }
}
