//! Shared, native-only RFC 5545 subscription normalization. URLs never enter this module's
//! public values; callers pass already fetched bytes and reconcile only after a full parse.
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
pub const MAX_OCCURRENCES: u16 = 2_000;
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
fn text(p: Option<&Property>) -> Option<String> {
    p.and_then(|p| p.value.as_ref()).map(|v| {
        v.replace("\\n", "\n")
            .replace("\\,", ",")
            .replace("\\;", ";")
            .replace("\\\\", "\\")
    })
}
fn param(p: &Property, key: &str) -> Option<String> {
    p.params
        .as_ref()?
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.first())
        .cloned()
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
    let uid = text(value(props, "UID"))
        .filter(|v| !v.trim().is_empty())
        .ok_or("missing_uid")?;
    let start_p = value(props, "DTSTART").ok_or("missing_dtstart")?;
    let start = text(Some(start_p)).ok_or("missing_dtstart")?;
    let all_day = param(start_p, "VALUE").as_deref() == Some("DATE") || start.len() == 8;
    let tz = if all_day {
        "date".into()
    } else if start.ends_with('Z') {
        "UTC".into()
    } else {
        param(start_p, "TZID").ok_or("floating_datetime")?
    };
    let url = text(value(props, "URL")).filter(|v| v.starts_with("https://"));
    let recurrence = text(value(props, "RECURRENCE-ID"))
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
        status: text(value(props, "STATUS")).unwrap_or_default(),
        start,
        end: text(value(props, "DTEND")),
        all_day,
        tz,
        summary: text(value(props, "SUMMARY")).unwrap_or_else(|| "Untitled event".into()),
        description: text(value(props, "DESCRIPTION")),
        location: text(value(props, "LOCATION")),
        url,
        categories: values(props, "CATEGORIES")
            .into_iter()
            .filter_map(|property| text(Some(property)))
            .flat_map(|value| value.split(',').map(str::to_string).collect::<Vec<_>>())
            .collect(),
        rrule: text(value(props, "RRULE")),
        rdates: values(props, "RDATE")
            .into_iter()
            .filter_map(|p| text(Some(p)))
            .flat_map(|v| v.split(',').map(str::to_string).collect::<Vec<_>>())
            .collect(),
        exdates: values(props, "EXDATE")
            .into_iter()
            .filter_map(|p| text(Some(p)))
            .flat_map(|v| v.split(',').map(str::to_string).collect::<Vec<_>>())
            .collect(),
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
    let mut output = Vec::new();
    for e in &events {
        if e.recurrence.is_some() {
            let mut x = input(
                e,
                e.recurrence.clone(),
                &e.start,
                e.end.as_deref(),
                metadata(&e.categories, e.description.as_deref()),
            )?;
            x.connection_id = connection_id.into();
            output.push(x);
            continue;
        }
        if e.rrule.is_none() && e.rdates.is_empty() {
            let mut x = input(
                e,
                None,
                &e.start,
                e.end.as_deref(),
                metadata(&e.categories, e.description.as_deref()),
            )?;
            x.connection_id = connection_id.into();
            output.push(x);
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
                .all(MAX_OCCURRENCES);
            if result.limited {
                return Err("recurrence_limit".into());
            }
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
                let mut x = input(
                    e,
                    Some(original.clone()),
                    &original,
                    Some(&end),
                    metadata(&e.categories, e.description.as_deref()),
                )?;
                x.connection_id = connection_id.into();
                output.push(x);
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
        let result = set.after(after).before(before).all(MAX_OCCURRENCES);
        if result.limited {
            return Err("recurrence_limit".into());
        }
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
            let mut x = input(
                e,
                Some(original),
                &d.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string(),
                Some(&end),
                metadata(&e.categories, e.description.as_deref()),
            )?;
            x.connection_id = connection_id.into();
            output.push(x);
        }
    }
    if output.len() > usize::from(MAX_OCCURRENCES) {
        return Err("recurrence_limit".into());
    };
    Ok(output)
}
pub fn validate(bytes: &[u8]) -> IcsValidation {
    validate_with_host(bytes, None)
}

pub fn validate_with_host(bytes: &[u8], public_host: Option<String>) -> IcsValidation {
    let parsed: Result<Vec<_>, _> = IcalParser::new(BufReader::new(bytes)).collect();
    let Ok(calendars) = parsed else {
        return invalid_validation("invalid_icalendar", public_host);
    };
    if calendars.is_empty() {
        return invalid_validation("invalid_icalendar", public_host);
    }
    let display_name = calendars
        .iter()
        .find_map(|calendar| {
            text(value(&calendar.properties, "X-WR-CALNAME"))
                .or_else(|| text(value(&calendar.properties, "NAME")))
        })
        .map(|name| name.chars().take(200).collect());
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
        retry_after_at: Option<String>,
        rate_limit_reset_at: Option<String>,
    },
    RateLimited {
        retry_after_at: Option<String>,
        rate_limit_reset_at: Option<String>,
    },
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
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<FetchResult, String> {
    fetch_with_transport(url, etag, last_modified, &ReqwestTransport).await
}

async fn fetch_with_transport<T: FetchTransport + Sync>(
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
    transport: &T,
) -> Result<FetchResult, String> {
    crate::db::repositories::subscribed_calendars::validate_feed_url(url)?;
    let mut current =
        reqwest::Url::parse(url).map_err(|_| "feed_destination_invalid".to_string())?;
    let mut redirects = 0_u8;
    let response = loop {
        crate::db::repositories::subscribed_calendars::validate_feed_url(current.as_str())?;
        let (host, addresses) = transport.resolve(&current).await?;
        validate_resolved_addresses(&addresses)?;
        let response = transport
            .get(current.clone(), &host, &addresses, etag, last_modified)
            .await?;
        if !response.status.is_redirection() {
            break response;
        }
        let location = response
            .headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| "feed_redirect_invalid".to_string())?;
        current = redirect_target(&current, location, redirects)?;
        redirects += 1;
    };
    if response.status == reqwest::StatusCode::NOT_MODIFIED {
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
    let etag = response
        .headers
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let last_modified = response
        .headers
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    Ok(FetchResult::Complete {
        bytes: response.bytes,
        etag,
        last_modified,
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
        requests: Mutex<Vec<String>>,
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
            _etag: Option<&str>,
            _last_modified: Option<&str>,
        ) -> Result<TransportResponse, String> {
            self.requests.lock().unwrap().push(url.to_string());
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
            fetch_with_transport("https://public.example/source.ics", None, None, &transport).await,
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
            fetch_with_transport("https://public.example/source.ics", None, None, &transport)
                .await
                .unwrap_err(),
            "feed_redirect_limit"
        );
        assert_eq!(transport.requests.lock().unwrap().len(), 4);
    }
}
