//! Shared, native-only RFC 5545 subscription normalization. Bearer URLs never enter returned
//! values; callers reconcile only after a complete bounded fetch and parse.
use crate::calendar::ExternalEventInput;
use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use futures_util::StreamExt;
use ical::{property::Property, IcalParser};
use rrule::{RRuleSet, Tz as RRuleTz};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::BufReader,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

pub const MAX_FEED_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_COMPONENTS: usize = 5_000;
pub const MAX_FEED_OCCURRENCES: usize = 2_000;
pub const MAX_RECURRENCE_OCCURRENCES: u16 = 2_000;
const MAX_PROPERTIES_PER_EVENT: usize = 128;
const MAX_PROPERTY_VALUE_BYTES: usize = 80_000;
const MAX_RECURRENCE_PROPERTIES: usize = 32;
const MAX_RDATE_VALUES: usize = 512;
const MAX_EXDATE_VALUES: usize = 512;
const MAX_CATEGORY_VALUES: usize = 64;
const MAX_UID_CHARS: usize = 512;
const MAX_SUMMARY_CHARS: usize = 500;
const MAX_DESCRIPTION_CHARS: usize = 20_000;
const MAX_LOCATION_CHARS: usize = 500;
const MAX_URL_CHARS: usize = 2_048;
const MAX_TIMEZONE_CHARS: usize = 128;
const MAX_RRULE_CHARS: usize = 2_048;
const MAX_RECURRENCE_VALUE_CHARS: usize = 128;
const MAX_CATEGORY_CHARS: usize = 200;
const EXPANSION_DAYS: i64 = 366;

#[derive(Debug, Clone, serde::Serialize)]
pub struct IcsValidation {
    pub usable: bool,
    pub event_count: usize,
    pub error_code: Option<String>,
    pub display_name: Option<String>,
    pub covered_start: Option<String>,
    pub covered_end: Option<String>,
    pub public_host: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
struct Parsed {
    uid: String,
    recurrence: Option<String>,
    status: String,
    start: String,
    end: Option<String>,
    all_day: bool,
    tz: String,
    summary: String,
    description: Option<String>,
    location: Option<String>,
    url: Option<String>,
    categories: Vec<String>,
    rrule: Option<String>,
    rdates: Vec<String>,
    exdates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMetadata {
    pub event_kind: String,
    pub group_references: Vec<String>,
}

fn value<'a>(props: &'a [Property], name: &str) -> Option<&'a Property> {
    props.iter().find(|p| p.name == name)
}
fn values<'a>(props: &'a [Property], name: &str) -> Vec<&'a Property> {
    props.iter().filter(|p| p.name == name).collect()
}
fn unescape_text(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}
fn bounded_text(p: Option<&Property>, max_chars: usize) -> Result<Option<String>, String> {
    let Some(value) = p.and_then(|property| property.value.as_deref()) else {
        return Ok(None);
    };
    if value.len() > MAX_PROPERTY_VALUE_BYTES {
        return Err("property_limit".into());
    }
    let value = unescape_text(value);
    if value.chars().count() > max_chars {
        return Err("property_limit".into());
    }
    Ok(Some(value))
}
fn param(p: &Property, key: &str) -> Option<String> {
    p.params
        .as_ref()?
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.first())
        .cloned()
}
fn bounded_param(p: &Property, key: &str, max_chars: usize) -> Result<Option<String>, String> {
    let value = param(p, key);
    if value
        .as_ref()
        .is_some_and(|value| value.chars().count() > max_chars)
    {
        return Err("property_limit".into());
    }
    Ok(value)
}
fn ensure_single(props: &[Property], name: &str) -> Result<(), String> {
    if props
        .iter()
        .filter(|property| property.name == name)
        .count()
        > 1
    {
        return Err("recurrence_property_limit".into());
    }
    Ok(())
}
fn split_property_values(
    props: &[Property],
    name: &str,
    max_properties: usize,
    max_values: usize,
    max_chars: usize,
) -> Result<Vec<String>, String> {
    let matching = values(props, name);
    if matching.len() > max_properties {
        return Err("recurrence_property_limit".into());
    }
    let mut output = Vec::with_capacity(matching.len().min(max_values));
    for property in matching {
        let Some(value) = bounded_text(Some(property), MAX_PROPERTY_VALUE_BYTES)? else {
            continue;
        };
        for item in value.split(',') {
            if output.len() == max_values {
                return Err(match name {
                    "RDATE" => "rdate_limit",
                    "EXDATE" => "exdate_limit",
                    _ => "property_limit",
                }
                .into());
            }
            if item.chars().count() > max_chars {
                return Err("property_limit".into());
            }
            output.push(item.to_string());
        }
    }
    Ok(output)
}
fn validate_content_lines(bytes: &[u8]) -> Result<(), String> {
    let mut logical_len = 0_usize;
    for raw_line in bytes.split(|byte| *byte == b'\n') {
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
        if matches!(line.first(), Some(b' ' | b'\t')) {
            logical_len = logical_len.saturating_add(line.len().saturating_sub(1));
        } else {
            logical_len = line.len();
        }
        if logical_len > MAX_PROPERTY_VALUE_BYTES {
            return Err("property_limit".into());
        }
    }
    Ok(())
}
fn datetime(value: &str, tzid: &str) -> Result<DateTime<Utc>, String> {
    if let Some(raw) = value.strip_suffix('Z') {
        return NaiveDateTime::parse_from_str(raw, "%Y%m%dT%H%M%S")
            .map(|d| Utc.from_utc_datetime(&d))
            .map_err(|_| "invalid_datetime".into());
    }
    let tz = Tz::from_str(tzid).map_err(|_| "unresolved_timezone".to_string())?;
    let d = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S")
        .map_err(|_| "invalid_datetime".to_string())?;
    tz.from_local_datetime(&d)
        .single()
        .map(|d| d.with_timezone(&Utc))
        .ok_or_else(|| "ambiguous_or_invalid_local_time".into())
}
fn date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y%m%d").map_err(|_| "invalid_date".into())
}
fn parse_event(props: &[Property]) -> Result<Parsed, String> {
    if props.len() > MAX_PROPERTIES_PER_EVENT {
        return Err("property_limit".into());
    }
    ensure_single(props, "RRULE")?;
    ensure_single(props, "RECURRENCE-ID")?;
    let recurrence_property_count = props
        .iter()
        .filter(|property| {
            matches!(
                property.name.as_str(),
                "RRULE" | "RDATE" | "EXDATE" | "RECURRENCE-ID"
            )
        })
        .count();
    if recurrence_property_count > MAX_RECURRENCE_PROPERTIES {
        return Err("recurrence_property_limit".into());
    }
    let uid = bounded_text(value(props, "UID"), MAX_UID_CHARS)?
        .filter(|v| !v.trim().is_empty())
        .ok_or("missing_uid")?;
    let start_p = value(props, "DTSTART").ok_or("missing_dtstart")?;
    let start =
        bounded_text(Some(start_p), MAX_RECURRENCE_VALUE_CHARS)?.ok_or("missing_dtstart")?;
    let all_day =
        bounded_param(start_p, "VALUE", 32)?.as_deref() == Some("DATE") || start.len() == 8;
    let tz = if all_day {
        "date".into()
    } else if start.ends_with('Z') {
        "UTC".into()
    } else {
        bounded_param(start_p, "TZID", MAX_TIMEZONE_CHARS)?.ok_or("floating_datetime")?
    };
    let url =
        bounded_text(value(props, "URL"), MAX_URL_CHARS)?.filter(|v| v.starts_with("https://"));
    let recurrence = bounded_text(value(props, "RECURRENCE-ID"), MAX_RECURRENCE_VALUE_CHARS)?
        .map(|v| {
            if all_day {
                Ok(v)
            } else {
                datetime(&v, &tz).map(|d| d.format("%Y%m%dT%H%M%SZ").to_string())
            }
        })
        .transpose()?;
    Ok(Parsed {
        uid,
        recurrence,
        status: bounded_text(value(props, "STATUS"), 64)?.unwrap_or_default(),
        start,
        end: bounded_text(value(props, "DTEND"), MAX_RECURRENCE_VALUE_CHARS)?,
        all_day,
        tz,
        summary: bounded_text(value(props, "SUMMARY"), MAX_SUMMARY_CHARS)?
            .unwrap_or_else(|| "Untitled event".into()),
        description: bounded_text(value(props, "DESCRIPTION"), MAX_DESCRIPTION_CHARS)?,
        location: bounded_text(value(props, "LOCATION"), MAX_LOCATION_CHARS)?,
        url,
        categories: split_property_values(
            props,
            "CATEGORIES",
            MAX_RECURRENCE_PROPERTIES,
            MAX_CATEGORY_VALUES,
            MAX_CATEGORY_CHARS,
        )?,
        rrule: bounded_text(value(props, "RRULE"), MAX_RRULE_CHARS)?,
        rdates: split_property_values(
            props,
            "RDATE",
            MAX_RECURRENCE_PROPERTIES,
            MAX_RDATE_VALUES,
            MAX_RECURRENCE_VALUE_CHARS,
        )?,
        exdates: split_property_values(
            props,
            "EXDATE",
            MAX_RECURRENCE_PROPERTIES,
            MAX_EXDATE_VALUES,
            MAX_RECURRENCE_VALUE_CHARS,
        )?,
    })
}
fn input(
    e: &Parsed,
    occurrence: Option<String>,
    start: &str,
    end: Option<&str>,
    metadata: EventMetadata,
) -> Result<ExternalEventInput, String> {
    let hash = format!(
        "{:x}",
        Sha256::digest(
            format!(
                "{}|{}|{}|{}|{}",
                e.uid,
                occurrence.clone().unwrap_or_default(),
                start,
                end.unwrap_or(""),
                e.status
            )
            .as_bytes()
        )
    );
    let status = if e.status.eq_ignore_ascii_case("CANCELLED") {
        "cancelled"
    } else {
        "active"
    }
    .into();
    if e.all_day {
        let s = date(start)?;
        let finish = match end {
            Some(v) => date(v)?,
            None => s.succ_opt().ok_or("invalid_date")?,
        };
        return Ok(ExternalEventInput {
            connection_id: String::new(),
            external_id: e.uid.clone(),
            occurrence_id: occurrence,
            title: e.summary.clone(),
            description: e.description.clone(),
            time_kind: "all_day".into(),
            start_at_utc: None,
            end_at_utc: None,
            start_date: Some(s.to_string()),
            end_date: Some(finish.to_string()),
            timezone: "date".into(),
            location: e.location.clone(),
            course_reference: None,
            group_references: metadata.group_references,
            event_kind: metadata.event_kind,
            status,
            source_url: e.url.clone(),
            ingestion_provenance: "ics_feed".into(),
            source_version: "rfc5545".into(),
            content_hash: hash,
        });
    }
    let s = datetime(start, &e.tz)?;
    let finish = match end {
        Some(v) => datetime(v, &e.tz)?,
        None => s + Duration::hours(1),
    };
    if finish <= s {
        return Err("invalid_duration".into());
    };
    Ok(ExternalEventInput {
        connection_id: String::new(),
        external_id: e.uid.clone(),
        occurrence_id: occurrence,
        title: e.summary.clone(),
        description: e.description.clone(),
        time_kind: "timed".into(),
        start_at_utc: Some(s.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        end_at_utc: Some(finish.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        start_date: None,
        end_date: None,
        timezone: e.tz.clone(),
        location: e.location.clone(),
        course_reference: None,
        group_references: metadata.group_references,
        event_kind: metadata.event_kind,
        status,
        source_url: e.url.clone(),
        ingestion_provenance: "ics_feed".into(),
        source_version: "rfc5545".into(),
        content_hash: hash,
    })
}

struct OccurrenceBudget {
    remaining: usize,
}

impl OccurrenceBudget {
    fn new() -> Self {
        Self {
            remaining: MAX_FEED_OCCURRENCES,
        }
    }

    fn consume(&mut self) -> Result<(), String> {
        if self.remaining == 0 {
            return Err("occurrence_limit".into());
        }
        self.remaining -= 1;
        Ok(())
    }

    fn push<T, F>(&mut self, output: &mut Vec<T>, build: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        self.consume()?;
        output.push(build()?);
        Ok(())
    }

    fn expansion_limit(&self) -> u16 {
        let feed_sentinel = self.remaining.saturating_add(1).min(usize::from(u16::MAX));
        let recurrence_sentinel = usize::from(MAX_RECURRENCE_OCCURRENCES).saturating_add(1);
        feed_sentinel.min(recurrence_sentinel) as u16
    }

    fn accept_expansion(&self, count: usize, limited: bool) -> Result<(), String> {
        if limited || count > self.remaining || count > usize::from(MAX_RECURRENCE_OCCURRENCES) {
            return Err("occurrence_limit".into());
        }
        Ok(())
    }
}

pub fn normalize(
    bytes: &[u8],
    connection_id: &str,
    now: DateTime<Utc>,
) -> Result<Vec<ExternalEventInput>, String> {
    normalize_with_classifier(bytes, connection_id, now, |_| "general".to_string())
}

pub fn normalize_with_classifier<F>(
    bytes: &[u8],
    connection_id: &str,
    now: DateTime<Utc>,
    classify: F,
) -> Result<Vec<ExternalEventInput>, String>
where
    F: Fn(&[String]) -> String,
{
    normalize_with_metadata(bytes, connection_id, now, |categories, _| EventMetadata {
        event_kind: classify(categories),
        group_references: Vec::new(),
    })
}

pub fn normalize_with_metadata<F>(
    bytes: &[u8],
    connection_id: &str,
    now: DateTime<Utc>,
    metadata: F,
) -> Result<Vec<ExternalEventInput>, String>
where
    F: Fn(&[String], Option<&str>) -> EventMetadata,
{
    if bytes.len() > MAX_FEED_BYTES {
        return Err("feed_too_large".into());
    };
    validate_content_lines(bytes)?;
    let calendars: Vec<_> = IcalParser::new(BufReader::new(bytes))
        .collect::<Result<_, _>>()
        .map_err(|_| "invalid_icalendar".to_string())?;
    let raw: Vec<_> = calendars.iter().flat_map(|c| c.events.iter()).collect();
    if raw.is_empty() || raw.len() > MAX_COMPONENTS {
        return Err("unusable_icalendar".into());
    };
    let events: Vec<Parsed> = raw
        .into_iter()
        .map(|e| parse_event(&e.properties))
        .collect::<Result<_, _>>()?;
    let overrides: HashSet<(String, String)> = events
        .iter()
        .filter_map(|e| e.recurrence.clone().map(|r| (e.uid.clone(), r)))
        .collect();
    let mut output = Vec::with_capacity(events.len().min(MAX_FEED_OCCURRENCES));
    let mut budget = OccurrenceBudget::new();
    for e in &events {
        if e.recurrence.is_some() {
            budget.push(&mut output, || {
                let mut occurrence = input(
                    e,
                    e.recurrence.clone(),
                    &e.start,
                    e.end.as_deref(),
                    metadata(&e.categories, e.description.as_deref()),
                )?;
                occurrence.connection_id = connection_id.into();
                Ok(occurrence)
            })?;
            continue;
        }
        if e.rrule.is_none() && e.rdates.is_empty() {
            budget.push(&mut output, || {
                let mut occurrence = input(
                    e,
                    None,
                    &e.start,
                    e.end.as_deref(),
                    metadata(&e.categories, e.description.as_deref()),
                )?;
                occurrence.connection_id = connection_id.into();
                Ok(occurrence)
            })?;
            continue;
        }
        if e.all_day {
            let start_date = date(&e.start)?;
            let start =
                Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).ok_or("invalid_date")?);
            let local = start.with_timezone(&RRuleTz::UTC);
            let mut set = RRuleSet::new(local);
            if let Some(rule) = &e.rrule {
                set = set.rrule(
                    rrule::RRule::from_str(rule)
                        .map_err(|_| "invalid_rrule")?
                        .validate(local)
                        .map_err(|_| "invalid_rrule")?,
                );
            }
            for r in &e.rdates {
                let d = date(r)?;
                set = set.rdate(
                    Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).ok_or("invalid_date")?)
                        .with_timezone(&RRuleTz::UTC),
                );
            }
            for r in &e.exdates {
                let d = date(r)?;
                set = set.exdate(
                    Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).ok_or("invalid_date")?)
                        .with_timezone(&RRuleTz::UTC),
                );
            }
            let result = set
                .after((now - Duration::days(EXPANSION_DAYS)).with_timezone(&RRuleTz::UTC))
                .before((now + Duration::days(EXPANSION_DAYS)).with_timezone(&RRuleTz::UTC))
                .all(budget.expansion_limit());
            budget.accept_expansion(result.dates.len(), result.limited)?;
            let duration = e
                .end
                .as_deref()
                .map(|v| date(v).map(|d| d - start_date))
                .transpose()?
                .unwrap_or_else(|| Duration::days(1));
            for d in result.dates {
                let original = d.format("%Y%m%d").to_string();
                if overrides.contains(&(e.uid.clone(), original.clone())) {
                    continue;
                };
                let end = (d.date_naive() + duration).format("%Y%m%d").to_string();
                budget.push(&mut output, || {
                    let mut occurrence = input(
                        e,
                        Some(original.clone()),
                        &original,
                        Some(&end),
                        metadata(&e.categories, e.description.as_deref()),
                    )?;
                    occurrence.connection_id = connection_id.into();
                    Ok(occurrence)
                })?;
            }
            continue;
        }
        let start = datetime(&e.start, &e.tz)?;
        let tz: RRuleTz = Tz::from_str(&e.tz)
            .map_err(|_| "unresolved_timezone")?
            .into();
        let local = start.with_timezone(&tz);
        let mut set = RRuleSet::new(local);
        if let Some(rule) = &e.rrule {
            let parsed = rrule::RRule::from_str(rule)
                .map_err(|_| "invalid_rrule")?
                .validate(local)
                .map_err(|_| "invalid_rrule")?;
            set = set.rrule(parsed);
        }
        for r in &e.rdates {
            set = set.rdate(datetime(r, &e.tz)?.with_timezone(&tz));
        }
        for x in &e.exdates {
            set = set.exdate(datetime(x, &e.tz)?.with_timezone(&tz));
        }
        let before = (now + Duration::days(EXPANSION_DAYS)).with_timezone(&tz);
        let after = (now - Duration::days(EXPANSION_DAYS)).with_timezone(&tz);
        let result = set
            .after(after)
            .before(before)
            .all(budget.expansion_limit());
        budget.accept_expansion(result.dates.len(), result.limited)?;
        let duration = e
            .end
            .as_deref()
            .map(|v| datetime(v, &e.tz).map(|d| d - start))
            .transpose()?
            .unwrap_or_else(|| Duration::hours(1));
        for d in result.dates {
            let original = d.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string();
            if overrides.contains(&(e.uid.clone(), original.clone())) {
                continue;
            };
            let end = (d.with_timezone(&Utc) + duration)
                .format("%Y%m%dT%H%M%SZ")
                .to_string();
            budget.push(&mut output, || {
                let mut occurrence = input(
                    e,
                    Some(original),
                    &d.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string(),
                    Some(&end),
                    metadata(&e.categories, e.description.as_deref()),
                )?;
                occurrence.connection_id = connection_id.into();
                Ok(occurrence)
            })?;
        }
    }
    Ok(output)
}
pub fn validate_with_host(bytes: &[u8], public_host: Option<String>) -> IcsValidation {
    let parsed: Result<Vec<_>, _> = IcalParser::new(BufReader::new(bytes)).collect();
    let Ok(calendars) = parsed else {
        return invalid_validation("invalid_icalendar", public_host);
    };
    if calendars.is_empty() {
        return invalid_validation("invalid_icalendar", public_host);
    }
    let display_property = calendars.iter().find_map(|calendar| {
        value(&calendar.properties, "X-WR-CALNAME").or_else(|| value(&calendar.properties, "NAME"))
    });
    let display_name = match bounded_text(display_property, 200) {
        Ok(value) => value,
        Err(code) => return invalid_validation(&code, public_host),
    };
    let raw_count = calendars
        .iter()
        .map(|calendar| calendar.events.len())
        .sum::<usize>();
    if raw_count == 0 {
        return IcsValidation {
            usable: true,
            event_count: 0,
            error_code: None,
            display_name,
            covered_start: None,
            covered_end: None,
            public_host,
            warnings: vec!["Calendar contains no events".into()],
        };
    }
    match normalize(bytes, "validation", Utc::now()) {
        Ok(events) => {
            let mut values = events
                .iter()
                .filter_map(|event| event.start_at_utc.as_ref().or(event.start_date.as_ref()))
                .cloned()
                .collect::<Vec<_>>();
            values.sort();
            IcsValidation {
                usable: true,
                event_count: events.len(),
                error_code: None,
                display_name,
                covered_start: values.first().cloned(),
                covered_end: values.last().cloned(),
                public_host,
                warnings: vec![],
            }
        }
        Err(code) => invalid_validation(&code, public_host),
    }
}

fn invalid_validation(code: &str, public_host: Option<String>) -> IcsValidation {
    IcsValidation {
        usable: false,
        event_count: 0,
        error_code: Some(code.into()),
        display_name: None,
        covered_start: None,
        covered_end: None,
        public_host,
        warnings: vec![],
    }
}

#[derive(Debug)]
pub enum FetchResult {
    NotModified,
    Complete {
        bytes: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
        validator_origin: String,
        retry_after_at: Option<String>,
        rate_limit_reset_at: Option<String>,
    },
    RateLimited {
        retry_after_at: Option<String>,
        rate_limit_reset_at: Option<String>,
    },
}

#[derive(Clone, Copy)]
pub struct ConditionalValidators<'a> {
    pub origin: &'a str,
    pub etag: Option<&'a str>,
    pub last_modified: Option<&'a str>,
}
pub fn retry_after_at(value: Option<&str>, now: DateTime<Utc>) -> Option<String> {
    let value = value?.trim();
    let at = value
        .parse::<i64>()
        .ok()
        .and_then(|seconds| {
            seconds
                .checked_abs()
                .map(|_| now + Duration::seconds(seconds.max(0)))
        })
        .or_else(|| {
            DateTime::parse_from_rfc2822(value)
                .ok()
                .map(|time| time.with_timezone(&Utc))
        })?;
    Some(at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}
pub fn rate_limit_reset_at(value: Option<&str>) -> Option<String> {
    let seconds = value?.trim().parse::<i64>().ok()?;
    DateTime::from_timestamp(seconds, 0)
        .map(|at| at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || octets[0] == 0
                || octets[0] >= 224
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 0)
                || (octets[0] == 198 && matches!(octets[1], 18 | 19))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113))
        }
        IpAddr::V6(ip) => {
            let first = ip.octets()[0];
            !(ip.to_ipv4_mapped()
                .is_some_and(|mapped| !public_ip(IpAddr::V4(mapped)))
                || ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (first == 0xfe && (ip.octets()[1] & 0xc0) == 0x80)
                || (first & 0xfe) == 0xfc)
        }
    }
}

async fn resolve_public_destination(
    url: &reqwest::Url,
) -> Result<(String, Vec<SocketAddr>), String> {
    let host = url
        .host_str()
        .ok_or_else(|| "feed_destination_invalid".to_string())?
        .trim_matches(['[', ']'])
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "feed_destination_invalid".to_string())?;
    let addresses = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|_| "feed_destination_unavailable".to_string())?
        .collect::<Vec<_>>();
    Ok((host, addresses))
}

fn validate_resolved_addresses(addresses: &[SocketAddr]) -> Result<(), String> {
    if addresses.is_empty() || addresses.iter().any(|address| !public_ip(address.ip())) {
        return Err("feed_destination_unsafe".to_string());
    }
    Ok(())
}

fn redirect_target(
    current: &reqwest::Url,
    location: &str,
    hops: u8,
) -> Result<reqwest::Url, String> {
    if hops >= 3 {
        return Err("feed_redirect_limit".into());
    }
    let next = current
        .join(location)
        .map_err(|_| "feed_redirect_invalid".to_string())?;
    crate::db::repositories::subscribed_calendars::validate_feed_url(next.as_str())
        .map_err(|_| "feed_redirect_unsafe".to_string())?;
    Ok(next)
}

fn is_follow_redirect(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::MOVED_PERMANENTLY
            | reqwest::StatusCode::FOUND
            | reqwest::StatusCode::SEE_OTHER
            | reqwest::StatusCode::TEMPORARY_REDIRECT
            | reqwest::StatusCode::PERMANENT_REDIRECT
    )
}

fn request_origin(url: &reqwest::Url) -> String {
    url.origin().ascii_serialization()
}

fn response_header(
    headers: &reqwest::header::HeaderMap,
    name: reqwest::header::HeaderName,
    max_bytes: usize,
) -> Result<Option<String>, String> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_| "feed_validator_invalid")?;
    if value.len() > max_bytes {
        return Err("feed_validator_invalid".into());
    }
    Ok(Some(value.to_string()))
}

struct TransportResponse {
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    bytes: Vec<u8>,
}

#[async_trait]
trait FetchTransport {
    async fn resolve(&self, url: &reqwest::Url) -> Result<(String, Vec<SocketAddr>), String>;
    async fn get(
        &self,
        url: reqwest::Url,
        host: &str,
        addresses: &[SocketAddr],
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<TransportResponse, String>;
}

struct ReqwestTransport;

#[async_trait]
impl FetchTransport for ReqwestTransport {
    async fn resolve(&self, url: &reqwest::Url) -> Result<(String, Vec<SocketAddr>), String> {
        resolve_public_destination(url).await
    }

    async fn get(
        &self,
        url: reqwest::Url,
        host: &str,
        addresses: &[SocketAddr],
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<TransportResponse, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(host, addresses)
            .build()
            .map_err(|_| "feed_transport_error")?;
        let mut request = client
            .get(url)
            .header(reqwest::header::ACCEPT, "text/calendar");
        if let Some(value) = etag {
            request = request.header(reqwest::header::IF_NONE_MATCH, value);
        }
        if let Some(value) = last_modified {
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, value);
        }
        let response = request.send().await.map_err(|_| "feed_fetch_failed")?;
        if response
            .content_length()
            .is_some_and(|length| length as usize > MAX_FEED_BYTES)
        {
            return Err("feed_too_large".into());
        }
        let status = response.status();
        let headers = response.headers().clone();
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(part) = stream.next().await {
            let part = part.map_err(|_| "feed_fetch_failed")?;
            if bytes.len() + part.len() > MAX_FEED_BYTES {
                return Err("feed_too_large".into());
            }
            bytes.extend_from_slice(&part);
        }
        Ok(TransportResponse {
            status,
            headers,
            bytes,
        })
    }
}

pub async fn fetch(
    url: &str,
    validators: Option<ConditionalValidators<'_>>,
) -> Result<FetchResult, String> {
    fetch_with_transport(url, validators, &ReqwestTransport).await
}

async fn fetch_with_transport<T: FetchTransport + Sync>(
    url: &str,
    validators: Option<ConditionalValidators<'_>>,
    transport: &T,
) -> Result<FetchResult, String> {
    crate::db::repositories::subscribed_calendars::validate_feed_url(url)?;
    let mut current =
        reqwest::Url::parse(url).map_err(|_| "feed_destination_invalid".to_string())?;
    let mut redirects = 0_u8;
    let mut validators = validators.filter(|validators| {
        validators.origin == request_origin(&current)
            && (validators.etag.is_some() || validators.last_modified.is_some())
    });
    let (response, final_url, sent_conditionals) = loop {
        crate::db::repositories::subscribed_calendars::validate_feed_url(current.as_str())?;
        let (host, addresses) = transport.resolve(&current).await?;
        validate_resolved_addresses(&addresses)?;
        let etag = validators.and_then(|validators| validators.etag);
        let last_modified = validators.and_then(|validators| validators.last_modified);
        let sent_conditionals = etag.is_some() || last_modified.is_some();
        let response = transport
            .get(current.clone(), &host, &addresses, etag, last_modified)
            .await?;
        if !is_follow_redirect(response.status) {
            break (response, current, sent_conditionals);
        }
        let location = response
            .headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| "feed_redirect_invalid".to_string())?;
        let next = redirect_target(&current, location, redirects)?;
        if request_origin(&next) != request_origin(&current) {
            validators = None;
        }
        current = next;
        redirects += 1;
    };
    if response.status == reqwest::StatusCode::NOT_MODIFIED {
        if !sent_conditionals {
            return Err("feed_unexpected_not_modified".into());
        }
        return Ok(FetchResult::NotModified);
    };
    let retry_after_at = retry_after_at(
        response
            .headers
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok()),
        Utc::now(),
    );
    let rate_limit_reset_at = rate_limit_reset_at(
        response
            .headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok()),
    );
    if response.status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Ok(FetchResult::RateLimited {
            retry_after_at,
            rate_limit_reset_at,
        });
    }
    if !response.status.is_success() {
        return Err("feed_http_error".into());
    };
    let etag = response_header(&response.headers, reqwest::header::ETAG, 512)?;
    let last_modified = response_header(&response.headers, reqwest::header::LAST_MODIFIED, 128)?;
    Ok(FetchResult::Complete {
        bytes: response.bytes,
        etag,
        last_modified,
        validator_origin: request_origin(&final_url),
        retry_after_at,
        rate_limit_reset_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex};

    const HEAD: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n";
    const TAIL: &str = "END:VCALENDAR\r\n";

    struct FixtureTransport {
        resolutions: Mutex<VecDeque<Vec<SocketAddr>>>,
        responses: Mutex<VecDeque<TransportResponse>>,
        requests: Mutex<Vec<FixtureRequest>>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct FixtureRequest {
        url: String,
        etag: Option<String>,
        last_modified: Option<String>,
    }

    impl FixtureTransport {
        fn new(resolutions: Vec<Vec<SocketAddr>>, responses: Vec<TransportResponse>) -> Self {
            Self {
                resolutions: Mutex::new(resolutions.into()),
                responses: Mutex::new(responses.into()),
                requests: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl FetchTransport for FixtureTransport {
        async fn resolve(&self, url: &reqwest::Url) -> Result<(String, Vec<SocketAddr>), String> {
            Ok((
                url.host_str().unwrap_or_default().to_string(),
                self.resolutions
                    .lock()
                    .unwrap()
                    .pop_front()
                    .ok_or_else(|| "fixture_missing_resolution".to_string())?,
            ))
        }

        async fn get(
            &self,
            url: reqwest::Url,
            _host: &str,
            _addresses: &[SocketAddr],
            etag: Option<&str>,
            last_modified: Option<&str>,
        ) -> Result<TransportResponse, String> {
            self.requests.lock().unwrap().push(FixtureRequest {
                url: url.to_string(),
                etag: etag.map(str::to_string),
                last_modified: last_modified.map(str::to_string),
            });
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| "fixture_missing_response".to_string())
        }
    }

    fn fixture_response(
        status: reqwest::StatusCode,
        location: Option<&str>,
        body: &[u8],
    ) -> TransportResponse {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(location) = location {
            headers.insert(reqwest::header::LOCATION, location.parse().unwrap());
        }
        TransportResponse {
            status,
            headers,
            bytes: body.to_vec(),
        }
    }

    fn fixture_response_with_validators(
        status: reqwest::StatusCode,
        location: Option<&str>,
        body: &[u8],
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> TransportResponse {
        let mut response = fixture_response(status, location, body);
        if let Some(etag) = etag {
            response
                .headers
                .insert(reqwest::header::ETAG, etag.parse().unwrap());
        }
        if let Some(last_modified) = last_modified {
            response.headers.insert(
                reqwest::header::LAST_MODIFIED,
                last_modified.parse().unwrap(),
            );
        }
        response
    }

    fn public_socket() -> SocketAddr {
        "8.8.8.8:443".parse().unwrap()
    }
    fn events(body: &str) -> Vec<ExternalEventInput> {
        normalize(
            format!("{HEAD}{body}{TAIL}").as_bytes(),
            "connection",
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        )
        .unwrap()
    }
    fn recurring_feed(count: usize) -> String {
        format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:series\r\nDTSTART:20260102T000000Z\r\nRRULE:FREQ=HOURLY;COUNT={count}\r\nSUMMARY:Bounded series\r\nEND:VEVENT\r\n{TAIL}"
        )
    }
    #[test]
    fn normalizes_timed_all_day_timezone_recurrence_exdate_override_and_cancellation() {
        let output=events("BEGIN:VEVENT\r\nUID:one\r\nDTSTART:20260103T090000Z\r\nDTEND:20260103T100000Z\r\nSUMMARY:Timed\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:day\r\nDTSTART;VALUE=DATE:20260104\r\nDTEND;VALUE=DATE:20260105\r\nSUMMARY:Day\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:series\r\nDTSTART;TZID=Europe/Berlin:20260105T090000\r\nDTEND;TZID=Europe/Berlin:20260105T100000\r\nRRULE:FREQ=DAILY;COUNT=3\r\nEXDATE;TZID=Europe/Berlin:20260106T090000\r\nSUMMARY:Series\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:series\r\nRECURRENCE-ID;TZID=Europe/Berlin:20260107T090000\r\nDTSTART;TZID=Europe/Berlin:20260107T110000\r\nDTEND;TZID=Europe/Berlin:20260107T120000\r\nSUMMARY:Moved\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:cancel\r\nDTSTART:20260108T090000Z\r\nDTEND:20260108T100000Z\r\nSTATUS:CANCELLED\r\nSUMMARY:Cancelled\r\nEND:VEVENT\r\n");
        assert_eq!(output.len(), 5);
        assert!(output.iter().any(|e| e.time_kind == "all_day"));
        assert!(output.iter().any(|e| e.timezone == "Europe/Berlin"));
        assert!(output
            .iter()
            .any(|e| e.title == "Moved" && e.occurrence_id.as_deref() == Some("20260107T080000Z")));
        assert!(output.iter().any(|e| e.status == "cancelled"));
    }
    #[test]
    fn rejects_floating_unsafe_and_unbounded_inputs() {
        assert!(normalize(
            format!(
                "{HEAD}BEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260101T090000\r\nEND:VEVENT\r\n{TAIL}"
            )
            .as_bytes(),
            "c",
            Utc::now()
        )
        .is_err());
        assert!(
            crate::db::repositories::subscribed_calendars::validate_feed_url(
                "http://example.test/a.ics"
            )
            .is_err()
        );
        assert!(normalize(&vec![b'x'; MAX_FEED_BYTES + 1], "c", Utc::now()).is_err());
    }

    #[test]
    fn aggregate_budget_accepts_exact_boundary_and_rejects_one_more() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let exact = normalize(
            recurring_feed(MAX_FEED_OCCURRENCES).as_bytes(),
            "connection",
            now,
        )
        .unwrap();
        assert_eq!(exact.len(), MAX_FEED_OCCURRENCES);
        assert_eq!(
            normalize(
                recurring_feed(MAX_FEED_OCCURRENCES + 1).as_bytes(),
                "connection",
                now,
            )
            .unwrap_err(),
            "occurrence_limit"
        );
    }

    #[test]
    fn combined_recurrence_series_share_one_incremental_budget() {
        let mut body = String::new();
        for index in 0..3 {
            body.push_str(&format!(
                "BEGIN:VEVENT\r\nUID:series-{index}\r\nDTSTART:20260102T000000Z\r\nRRULE:FREQ=HOURLY;COUNT=1000\r\nSUMMARY:Series {index}\r\nEND:VEVENT\r\n"
            ));
        }
        let error = normalize(
            format!("{HEAD}{body}{TAIL}").as_bytes(),
            "connection",
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error, "occurrence_limit");
    }

    #[test]
    fn ordinary_and_rdate_occurrences_share_the_feed_budget() {
        let mut body = String::new();
        for index in 0..(MAX_FEED_OCCURRENCES - 1) {
            body.push_str(&format!(
                "BEGIN:VEVENT\r\nUID:ordinary-{index}\r\nDTSTART:20260102T000000Z\r\nEND:VEVENT\r\n"
            ));
        }
        body.push_str(
            "BEGIN:VEVENT\r\nUID:rdate-series\r\nDTSTART:20260103T000000Z\r\nRDATE:20260104T000000Z,20260105T000000Z\r\nEND:VEVENT\r\n",
        );
        assert_eq!(
            normalize(
                format!("{HEAD}{body}{TAIL}").as_bytes(),
                "connection",
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            )
            .unwrap_err(),
            "occurrence_limit"
        );
    }

    #[test]
    fn occurrence_budget_never_appends_past_the_limit() {
        let mut budget = OccurrenceBudget::new();
        let mut output = Vec::new();
        for value in 0..MAX_FEED_OCCURRENCES {
            budget.push(&mut output, || Ok(value)).unwrap();
        }
        assert_eq!(output.len(), MAX_FEED_OCCURRENCES);
        assert_eq!(
            budget.push(&mut output, || Ok(MAX_FEED_OCCURRENCES)),
            Err("occurrence_limit".into())
        );
        assert_eq!(output.len(), MAX_FEED_OCCURRENCES);
    }

    #[test]
    fn bounds_recurrence_lists_repeated_properties_and_untrusted_text() {
        let dates = (0..=MAX_RDATE_VALUES)
            .map(|index| format!("202601{:02}T{:02}0000Z", 1 + index / 24, index % 24))
            .collect::<Vec<_>>()
            .join(",");
        let rdate_feed = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:rdates\r\nDTSTART:20260101T000000Z\r\nRDATE:{dates}\r\nEND:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(&rdate_feed.into_bytes(), "c", Utc::now()).unwrap_err(),
            "rdate_limit"
        );

        let exdates = (0..=MAX_EXDATE_VALUES)
            .map(|index| format!("202601{:02}T{:02}0000Z", 1 + index / 24, index % 24))
            .collect::<Vec<_>>()
            .join(",");
        let exdate_feed = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:exdates\r\nDTSTART:20260101T000000Z\r\nRRULE:FREQ=DAILY;COUNT=1\r\nEXDATE:{exdates}\r\nEND:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(&exdate_feed.into_bytes(), "c", Utc::now()).unwrap_err(),
            "exdate_limit"
        );

        let repeated_rule = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:repeat\r\nDTSTART:20260101T000000Z\r\nRRULE:FREQ=DAILY;COUNT=1\r\nRRULE:FREQ=WEEKLY;COUNT=1\r\nEND:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(repeated_rule.as_bytes(), "c", Utc::now()).unwrap_err(),
            "recurrence_property_limit"
        );

        let repeated_rdates = (0..=MAX_RECURRENCE_PROPERTIES)
            .map(|_| "RDATE:20260102T000000Z\r\n")
            .collect::<String>();
        let repeated_feed = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:repeat-rdate\r\nDTSTART:20260101T000000Z\r\n{repeated_rdates}END:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(repeated_feed.as_bytes(), "c", Utc::now()).unwrap_err(),
            "recurrence_property_limit"
        );

        let oversized_summary = "x".repeat(MAX_SUMMARY_CHARS + 1);
        let text_feed = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:text\r\nDTSTART:20260101T000000Z\r\nSUMMARY:{oversized_summary}\r\nEND:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(text_feed.as_bytes(), "c", Utc::now()).unwrap_err(),
            "property_limit"
        );
    }

    #[test]
    fn rejects_oversized_logical_property_before_ical_allocation() {
        let oversized = "x".repeat(MAX_PROPERTY_VALUE_BYTES + 1);
        let feed = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:large\r\nDTSTART:20260101T000000Z\r\nDESCRIPTION:{oversized}\r\nEND:VEVENT\r\n{TAIL}"
        );
        assert_eq!(
            normalize(feed.as_bytes(), "c", Utc::now()).unwrap_err(),
            "property_limit"
        );
    }
    #[test]
    fn parses_retry_after_delta_http_date_and_reset() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(
            retry_after_at(Some("30"), now).as_deref(),
            Some("2026-01-01T00:00:30Z")
        );
        assert_eq!(
            retry_after_at(Some("Thu, 01 Jan 2026 00:01:00 GMT"), now).as_deref(),
            Some("2026-01-01T00:01:00Z")
        );
        assert_eq!(
            rate_limit_reset_at(Some("1767225660")).as_deref(),
            Some("2026-01-01T00:01:00Z")
        );
    }
    #[test]
    fn shared_destination_policy_rejects_private_reserved_and_mapped_addresses() {
        for value in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.1.1",
            "100.64.0.1",
            "198.18.0.1",
            "::1",
            "fe80::1",
            "fd00::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(!public_ip(value.parse().unwrap()), "{value}");
        }
        assert!(public_ip("8.8.8.8".parse().unwrap()));
        assert!(public_ip("2606:4700:4700::1111".parse().unwrap()));
    }
    #[test]
    fn resolved_hostname_addresses_are_checked_before_fetch_without_exposing_them() {
        for address in [
            "127.0.0.1:443",
            "10.0.0.1:443",
            "[::1]:443",
            "[fe80::1]:443",
            "[::ffff:127.0.0.1]:443",
        ] {
            let error = validate_resolved_addresses(&[address.parse().unwrap()]).unwrap_err();
            assert_eq!(error, "feed_destination_unsafe");
            assert!(!error.contains(address));
        }
        assert!(validate_resolved_addresses(&["8.8.8.8:443".parse().unwrap()]).is_ok());
    }
    #[test]
    fn redirect_policy_preserves_https_and_rejects_unsafe_targets_without_leaking_them() {
        let current = reqwest::Url::parse("https://public.example/secret?token=value").unwrap();
        assert!(redirect_target(&current, "https://calendar.example/next", 0).is_ok());
        for target in [
            "http://calendar.example/feed",
            "https://localhost/feed",
            "https://127.0.0.1/feed",
            "https://[::1]/feed",
        ] {
            let error = redirect_target(&current, target, 0).unwrap_err();
            assert_eq!(error, "feed_redirect_unsafe");
            assert!(!error.contains("token"));
            assert!(!error.contains("127.0.0.1"));
        }
        assert_eq!(
            redirect_target(&current, "https://calendar.example/feed", 3).unwrap_err(),
            "feed_redirect_limit"
        );
    }

    #[tokio::test]
    async fn fetch_loop_rejects_unsafe_dns_before_request_without_network_or_secret_leakage() {
        for address in ["10.0.0.8:443", "[fe80::42]:443"] {
            let transport = FixtureTransport::new(vec![vec![address.parse().unwrap()]], vec![]);
            let error = fetch_with_transport(
                "https://public.example/private-path?token=source-secret",
                None,
                &transport,
            )
            .await
            .unwrap_err();
            assert_eq!(error, "feed_destination_unsafe");
            assert!(transport.requests.lock().unwrap().is_empty());
            for secret in ["source-secret", "private-path", "10.0.0.8", "fe80::42"] {
                assert!(!error.contains(secret));
            }
        }
    }

    #[tokio::test]
    async fn fetch_loop_manually_blocks_unsafe_redirects_before_following_them() {
        for target in [
            "https://10.0.0.8/private?target=secret",
            "https://localhost/private?target=secret",
            "https://[fe80::42]/private?target=secret",
        ] {
            let transport = FixtureTransport::new(
                vec![vec![public_socket()]],
                vec![fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some(target),
                    b"",
                )],
            );
            let error = fetch_with_transport(
                "https://public.example/source-path?token=source-secret",
                None,
                &transport,
            )
            .await
            .unwrap_err();
            assert_eq!(error, "feed_redirect_unsafe");
            assert_eq!(transport.requests.lock().unwrap().len(), 1);
            for secret in [
                "source-secret",
                "source-path",
                "target=secret",
                "10.0.0.8",
                "fe80::42",
            ] {
                assert!(!error.contains(secret));
            }
        }
    }

    #[tokio::test]
    async fn redirect_destination_dns_is_revalidated_before_the_next_request() {
        let transport = FixtureTransport::new(
            vec![vec![public_socket()], vec!["10.0.0.8:443".parse().unwrap()]],
            vec![fixture_response(
                reqwest::StatusCode::FOUND,
                Some("https://calendar.example/private-dns.ics"),
                b"",
            )],
        );
        assert_eq!(
            fetch_with_transport("https://public.example/source.ics", None, &transport)
                .await
                .unwrap_err(),
            "feed_destination_unsafe"
        );
        assert_eq!(transport.requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn fetch_loop_follows_safe_https_redirect_and_enforces_hop_limit() {
        let body = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:event\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n{TAIL}"
        );
        let transport = FixtureTransport::new(
            vec![vec![public_socket()], vec![public_socket()]],
            vec![
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://calendar.example/final.ics"),
                    b"",
                ),
                fixture_response(reqwest::StatusCode::OK, None, body.as_bytes()),
            ],
        );
        assert!(matches!(
            fetch_with_transport("https://public.example/source.ics", None, &transport).await,
            Ok(FetchResult::Complete { .. })
        ));
        assert_eq!(transport.requests.lock().unwrap().len(), 2);

        let redirects = (0..4)
            .map(|_| {
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://public.example/next"),
                    b"",
                )
            })
            .collect();
        let transport = FixtureTransport::new(vec![vec![public_socket()]; 4], redirects);
        assert_eq!(
            fetch_with_transport("https://public.example/source.ics", None, &transport)
                .await
                .unwrap_err(),
            "feed_redirect_limit"
        );
        assert_eq!(transport.requests.lock().unwrap().len(), 4);
    }

    #[tokio::test]
    async fn direct_and_same_origin_redirects_preserve_origin_bound_validators_and_304() {
        let validators = ConditionalValidators {
            origin: "https://public.example",
            etag: Some("\"old\""),
            last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT"),
        };
        let transport = FixtureTransport::new(
            vec![vec![public_socket()], vec![public_socket()]],
            vec![
                fixture_response(reqwest::StatusCode::FOUND, Some("/next.ics"), b""),
                fixture_response(reqwest::StatusCode::NOT_MODIFIED, None, b""),
            ],
        );
        assert!(matches!(
            fetch_with_transport(
                "https://public.example/source.ics",
                Some(validators),
                &transport,
            )
            .await,
            Ok(FetchResult::NotModified)
        ));
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        for request in requests.iter() {
            assert_eq!(request.etag.as_deref(), Some("\"old\""));
            assert_eq!(
                request.last_modified.as_deref(),
                Some("Wed, 21 Oct 2015 07:28:00 GMT")
            );
        }
    }

    #[tokio::test]
    async fn ordinary_direct_request_remains_unconditioned_and_returns_complete_data() {
        let body = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:event\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n{TAIL}"
        );
        let transport = FixtureTransport::new(
            vec![vec![public_socket()]],
            vec![fixture_response(
                reqwest::StatusCode::OK,
                None,
                body.as_bytes(),
            )],
        );
        assert!(matches!(
            fetch_with_transport("https://public.example/source.ics", None, &transport).await,
            Ok(FetchResult::Complete { .. })
        ));
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].etag.is_none());
        assert!(requests[0].last_modified.is_none());
    }

    #[tokio::test]
    async fn cross_origin_redirect_strips_both_validators_and_returns_final_origin_metadata() {
        let body = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:event\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n{TAIL}"
        );
        let transport = FixtureTransport::new(
            vec![vec![public_socket()], vec![public_socket()]],
            vec![
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://calendar.example/final.ics"),
                    b"",
                ),
                fixture_response_with_validators(
                    reqwest::StatusCode::OK,
                    None,
                    body.as_bytes(),
                    Some("\"new\""),
                    Some("Thu, 22 Oct 2015 07:28:00 GMT"),
                ),
            ],
        );
        let result = fetch_with_transport(
            "https://public.example/source.ics",
            Some(ConditionalValidators {
                origin: "https://public.example",
                etag: Some("\"old\""),
                last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT"),
            }),
            &transport,
        )
        .await
        .unwrap();
        let FetchResult::Complete {
            etag,
            last_modified,
            validator_origin,
            ..
        } = result
        else {
            panic!("expected complete response");
        };
        assert_eq!(etag.as_deref(), Some("\"new\""));
        assert_eq!(
            last_modified.as_deref(),
            Some("Thu, 22 Oct 2015 07:28:00 GMT")
        );
        assert_eq!(validator_origin, "https://calendar.example");
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].etag.as_deref(), Some("\"old\""));
        assert!(requests[1].etag.is_none());
        assert!(requests[1].last_modified.is_none());
    }

    #[tokio::test]
    async fn validators_never_reappear_after_a_chain_crosses_origin() {
        let body = format!(
            "{HEAD}BEGIN:VEVENT\r\nUID:event\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n{TAIL}"
        );
        let transport = FixtureTransport::new(
            vec![vec![public_socket()]; 3],
            vec![
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://calendar.example/middle.ics"),
                    b"",
                ),
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://public.example/final.ics"),
                    b"",
                ),
                fixture_response(reqwest::StatusCode::OK, None, body.as_bytes()),
            ],
        );
        fetch_with_transport(
            "https://public.example/source.ics",
            Some(ConditionalValidators {
                origin: "https://public.example",
                etag: Some("\"old\""),
                last_modified: None,
            }),
            &transport,
        )
        .await
        .unwrap();
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].etag.as_deref(), Some("\"old\""));
        assert!(requests[1].etag.is_none());
        assert!(requests[2].etag.is_none());
    }

    #[tokio::test]
    async fn cross_origin_unconditioned_304_is_rejected() {
        let transport = FixtureTransport::new(
            vec![vec![public_socket()], vec![public_socket()]],
            vec![
                fixture_response(
                    reqwest::StatusCode::FOUND,
                    Some("https://calendar.example/final.ics"),
                    b"",
                ),
                fixture_response(reqwest::StatusCode::NOT_MODIFIED, None, b""),
            ],
        );
        let error = fetch_with_transport(
            "https://public.example/source.ics",
            Some(ConditionalValidators {
                origin: "https://public.example",
                etag: Some("\"old\""),
                last_modified: None,
            }),
            &transport,
        )
        .await
        .unwrap_err();
        assert_eq!(error, "feed_unexpected_not_modified");
    }
}
