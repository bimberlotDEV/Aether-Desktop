use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SubscribedCalendar {
    pub id: String,
    pub connection_id: String,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribedCalendarInput {
    pub connection_id: String,
    pub feed_url: String,
    pub display_name: Option<String>,
}

fn row(row: &rusqlite::Row) -> rusqlite::Result<SubscribedCalendar> {
    Ok(SubscribedCalendar {
        id: row.get(0)?,
        connection_id: row.get(1)?,
        display_name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub fn validate_feed_url(url: &str) -> Result<(), String> {
    let parsed =
        reqwest::Url::parse(url.trim()).map_err(|_| "Calendar feed URL is invalid".to_string())?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || parsed.username() != ""
        || parsed.password().is_some()
    {
        return Err("Calendar feed URL must be HTTPS without embedded credentials".to_string());
    }
    let host = parsed
        .host_str()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err("Calendar feed URL must use a public HTTPS destination".to_string());
    }
    if let Ok(ip) = host.trim_matches(['[', ']']).parse::<IpAddr>() {
        let unsafe_ip = match ip {
            IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast()
            }
            IpAddr::V6(ip) => {
                let first = ip.octets()[0];
                ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_multicast()
                    || (first == 0xfe && (ip.octets()[1] & 0xc0) == 0x80)
                    || (first & 0xfe) == 0xfc
            }
        };
        if unsafe_ip {
            return Err("Calendar feed URL must use a public HTTPS destination".to_string());
        }
    }
    if url.len() > 2048 {
        return Err("Calendar feed URL is too long".to_string());
    }
    Ok(())
}

pub fn create(
    conn: &Connection,
    input: &SubscribedCalendarInput,
) -> Result<SubscribedCalendar, String> {
    validate_feed_url(&input.feed_url)?;
    let connection_id = input.connection_id.trim();
    let auth: Option<String> = conn
        .query_row(
            "SELECT auth_type FROM integrations WHERE id=?1",
            [connection_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Calendar subscription connection lookup failed".to_string())?;
    if auth.as_deref() != Some("ics_feed") {
        return Err("Calendar subscriptions require an existing ics_feed Integration".to_string());
    }
    let display_name = input
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    if display_name
        .as_ref()
        .is_some_and(|v| v.chars().count() > 200)
    {
        return Err("Calendar subscription name is too long".to_string());
    }
    let id = Uuid::now_v7().to_string();
    conn.execute("INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES (?1,?2,?3) ON CONFLICT(connection_id) DO UPDATE SET display_name=excluded.display_name,updated_at=datetime('now')", params![id, connection_id, display_name]).map_err(|_| "Calendar subscription could not be saved".to_string())?;
    get_by_connection(conn, connection_id)?
        .ok_or_else(|| "Calendar subscription could not be loaded".to_string())
}

pub fn get_by_connection(
    conn: &Connection,
    connection_id: &str,
) -> Result<Option<SubscribedCalendar>, String> {
    conn.query_row("SELECT id,connection_id,display_name,created_at,updated_at FROM subscribed_calendars WHERE connection_id=?1", [connection_id], row).optional().map_err(|_| "Calendar subscription lookup failed".to_string())
}

pub fn credential_key(conn: &Connection, connection_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT credential_key FROM integrations WHERE id=?1 AND auth_type='ics_feed'",
        [connection_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map_err(|_| "Calendar subscription credential lookup failed".to_string())?
    .flatten()
    .ok_or_else(|| "Calendar subscription is not configured".to_string())
}

#[cfg(test)]
mod url_tests {
    use super::*;

    #[test]
    fn rejects_non_public_or_credential_bearing_urls() {
        for value in [
            "http://example.com/feed.ics",
            "https://user:pass@example.com/feed.ics",
            "https://localhost/feed.ics",
            "https://127.0.0.1/feed.ics",
            "https://169.254.1.2/feed.ics",
            "https://[::1]/feed.ics",
        ] {
            assert!(validate_feed_url(value).is_err(), "{value}");
        }
        assert!(validate_feed_url("https://calendar.example.edu/feed.ics").is_ok());
    }
}
