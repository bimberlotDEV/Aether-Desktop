use serde::{Deserialize, Serialize};

use crate::actions::ActionRequest;

const MAX_PROPOSALS: usize = 20;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDraft {
    pub index: usize,
    pub action_type: String,
    pub title: String,
    pub detail: String,
}

pub fn describe(actions: &[ActionRequest]) -> Vec<ActionDraft> {
    actions
        .iter()
        .enumerate()
        .map(|(index, action)| match action {
            ActionRequest::CreateTask {
                title, description, ..
            } => ActionDraft {
                index,
                action_type: "createTask".to_string(),
                title: title.clone(),
                detail: description.clone(),
            },
            ActionRequest::CreateNote { title, content, .. } => ActionDraft {
                index,
                action_type: "createNote".to_string(),
                title: title.clone(),
                detail: content.clone(),
            },
            _ => unreachable!("the AI parser only returns Task and Note drafts"),
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposalEnvelope {
    actions: Vec<RawAction>,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum RawAction {
    CreateTask {
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        due_date: Option<String>,
    },
    CreateNote {
        title: String,
        content: String,
    },
}

pub fn parse(content: &str, space_id: Option<&str>) -> Result<Vec<ActionRequest>, String> {
    if content.len() > 200_000 {
        return Err("The AI proposal is too large to review safely.".to_string());
    }
    let envelope: ProposalEnvelope = serde_json::from_str(content.trim()).map_err(|_| {
        "The AI response was not a valid Action proposal. Nothing was created.".to_string()
    })?;
    if envelope.actions.is_empty() || envelope.actions.len() > MAX_PROPOSALS {
        return Err(format!(
            "An AI proposal must contain 1 to {MAX_PROPOSALS} Actions."
        ));
    }
    envelope
        .actions
        .into_iter()
        .map(|action| match action {
            RawAction::CreateTask {
                title,
                description,
                due_date,
            } => {
                validate_text(&title, 200, "Task title")?;
                validate_optional_text(&description, 10_000, "Task description")?;
                if let Some(date) = due_date.as_deref() {
                    validate_date(date)?;
                }
                Ok(ActionRequest::CreateTask {
                    title,
                    description,
                    due_date,
                    space_id: space_id.map(str::to_string),
                })
            }
            RawAction::CreateNote { title, content } => {
                let space_id = space_id.ok_or_else(|| {
                    "AI can only propose a Note inside an active Space.".to_string()
                })?;
                validate_text(&title, 200, "Note title")?;
                validate_text(&content, 20_000, "Note content")?;
                Ok(ActionRequest::CreateNote {
                    title,
                    content,
                    space_id: space_id.to_string(),
                })
            }
        })
        .collect()
}

fn validate_text(value: &str, max: usize, label: &str) -> Result<(), String> {
    let count = value.trim().chars().count();
    if count == 0 || count > max {
        Err(format!("{label} must contain 1 to {max} characters."))
    } else {
        Ok(())
    }
}

fn validate_optional_text(value: &str, max: usize, label: &str) -> Result<(), String> {
    if value.chars().count() > max {
        Err(format!("{label} must contain at most {max} characters."))
    } else {
        Ok(())
    }
}

fn validate_date(value: &str) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "Task due date must be a real date formatted as YYYY-MM-DD.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_task_and_note_drafts_and_injects_trusted_scope() {
        let result = parse(r#"{"actions":[{"type":"createTask","title":"Ship","dueDate":"2026-09-01"},{"type":"createNote","title":"Decision","content":"Use the safe path"}]}"#, Some("space-1")).unwrap();
        assert_eq!(result.len(), 2);
        assert!(
            matches!(&result[0], ActionRequest::CreateTask { space_id: Some(value), .. } if value == "space-1")
        );
        assert!(
            matches!(&result[1], ActionRequest::CreateNote { space_id, .. } if space_id == "space-1")
        );
    }

    #[test]
    fn rejects_unknown_fields_types_fences_and_invalid_values() {
        assert!(parse(
            r#"{"actions":[{"type":"moveFile","sourceId":"x"}]}"#,
            Some("space")
        )
        .is_err());
        assert!(parse(
            r#"{"actions":[{"type":"createTask","title":"x","spaceId":"other"}]}"#,
            Some("space")
        )
        .is_err());
        assert!(parse("```json\n{\"actions\":[]}\n```", Some("space")).is_err());
        assert!(parse(
            r#"{"actions":[{"type":"createTask","title":"x","dueDate":"2026-99-99"}]}"#,
            Some("space")
        )
        .is_err());
        assert!(parse(
            r#"{"actions":[{"type":"createNote","title":"x","content":"body"}]}"#,
            None
        )
        .is_err());
    }
}
