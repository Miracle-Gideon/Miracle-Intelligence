//! mi-models: core domain types shared across MI.
//!
//! Deliberately has no storage or network dependencies — every other crate
//! depends on this one, never the other way around.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------
// Shared enums
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "high" => Some(Confidence::High),
            "medium" => Some(Confidence::Medium),
            "low" => Some(Confidence::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    Observed,
    Inferred,
    Potential,
    Confirmed,
    Unknown,
}

impl FindingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingStatus::Observed => "observed",
            FindingStatus::Inferred => "inferred",
            FindingStatus::Potential => "potential",
            FindingStatus::Confirmed => "confirmed",
            FindingStatus::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "observed" => Some(FindingStatus::Observed),
            "inferred" => Some(FindingStatus::Inferred),
            "potential" => Some(FindingStatus::Potential),
            "confirmed" => Some(FindingStatus::Confirmed),
            "unknown" => Some(FindingStatus::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Severity::Low),
            "medium" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Domain,
    Subdomain,
    Ip,
    Service,
    Port,
    Certificate,
}

impl AssetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetKind::Domain => "domain",
            AssetKind::Subdomain => "subdomain",
            AssetKind::Ip => "ip",
            AssetKind::Service => "service",
            AssetKind::Port => "port",
            AssetKind::Certificate => "certificate",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "domain" => Some(AssetKind::Domain),
            "subdomain" => Some(AssetKind::Subdomain),
            "ip" => Some(AssetKind::Ip),
            "service" => Some(AssetKind::Service),
            "port" => Some(AssetKind::Port),
            "certificate" => Some(AssetKind::Certificate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeCategory {
    Domain,
    IpRange,
    Ip,
}

impl ScopeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeCategory::Domain => "domain",
            ScopeCategory::IpRange => "ip_range",
            ScopeCategory::Ip => "ip",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "domain" => Some(ScopeCategory::Domain),
            "ip_range" => Some(ScopeCategory::IpRange),
            "ip" => Some(ScopeCategory::Ip),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeStatus {
    InScope,
    OutOfScope,
}

impl ScopeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeStatus::InScope => "in_scope",
            ScopeStatus::OutOfScope => "out_of_scope",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "in_scope" => Some(ScopeStatus::InScope),
            "out_of_scope" => Some(ScopeStatus::OutOfScope),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------
// Core structs
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub kind: AssetKind,
    pub value: String,
    pub org_id: Option<Uuid>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub confidence: Confidence,
}

impl Asset {
    pub fn new(kind: AssetKind, value: impl Into<String>, confidence: Confidence) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            kind,
            value: value.into(),
            org_id: None,
            first_seen: now,
            last_seen: now,
            confidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub provider: String,
    pub raw: serde_json::Value,
    pub normalized: serde_json::Value,
    pub observed_at: DateTime<Utc>,
}

impl Observation {
    pub fn new(
        asset_id: Uuid,
        provider: impl Into<String>,
        raw: serde_json::Value,
        normalized: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            provider: provider.into(),
            raw,
            normalized,
            observed_at: Utc::now(),
        }
    }
}

/// Not fully wired up to storage until Phase 3 (exposure/risk engine),
/// but defined here now so downstream crates can compile against it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub title: String,
    pub severity: Severity,
    pub status: FindingStatus,
    pub confidence: Confidence,
    pub attack_techniques: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: Uuid,
    pub pattern: String,
    pub category: ScopeCategory,
    pub status: ScopeStatus,
    pub authorized_by: String,
    pub authorized_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Scope {
    pub fn new(
        pattern: impl Into<String>,
        category: ScopeCategory,
        authorized_by: impl Into<String>,
        authorized_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            pattern: pattern.into(),
            category,
            status: ScopeStatus::InScope,
            authorized_by: authorized_by.into(),
            authorized_until,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_round_trips() {
        for c in [Confidence::High, Confidence::Medium, Confidence::Low] {
            assert_eq!(Confidence::parse(c.as_str()), Some(c));
        }
    }

    #[test]
    fn scope_new_defaults_to_in_scope() {
        let s = Scope::new("example.com", ScopeCategory::Domain, "test", None);
        assert_eq!(s.status, ScopeStatus::InScope);
    }
}
