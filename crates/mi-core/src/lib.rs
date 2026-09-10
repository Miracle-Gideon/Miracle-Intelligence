//! mi-core: domain logic that isn't storage or transport — right now, just
//! the ScopeGuard. Every provider call and verification adapter call must
//! pass through `ScopeGuard::check` before any network traffic is sent.

use std::net::IpAddr;
use std::str::FromStr;

use chrono::Utc;
use ipnetwork::IpNetwork;
use mi_models::{Scope, ScopeCategory, ScopeStatus};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ScopeError {
    #[error(
        "target '{target}' is not in scope — add it first:\n  mi scope add {target} --authorized-by \"<reason>\""
    )]
    NotAuthorized { target: String },

    #[error(transparent)]
    Storage(#[from] mi_storage::StorageError),
}

pub type Result<T> = std::result::Result<T, ScopeError>;

pub struct ScopeGuard<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ScopeGuard<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// The single choke point every network-touching action must call first.
    pub async fn check(&self, target: &str) -> Result<()> {
        if is_private_target(target) {
            tracing::debug!(target, "auto-trusted: private/loopback address");
            return Ok(());
        }

        let scopes = mi_storage::list_scopes(self.pool).await?;
        let now = Utc::now();

        let authorized = scopes.iter().any(|s| {
            s.status == ScopeStatus::InScope
                && s.authorized_until.map_or(true, |until| until > now)
                && scope_matches(&s.pattern, target)
        });

        if authorized {
            Ok(())
        } else {
            Err(ScopeError::NotAuthorized {
                target: target.to_string(),
            })
        }
    }

    pub async fn add_scope(
        &self,
        pattern: &str,
        authorized_by: &str,
        no_expiry: bool,
        until: Option<chrono::DateTime<Utc>>,
    ) -> Result<Scope> {
        let category = infer_category(pattern);
        let expiry = if no_expiry { None } else { until };
        let scope = Scope::new(pattern, category, authorized_by, expiry);
        mi_storage::insert_scope(self.pool, &scope).await?;
        Ok(scope)
    }

    pub async fn list_scopes(&self) -> Result<Vec<Scope>> {
        Ok(mi_storage::list_scopes(self.pool).await?)
    }

    pub async fn remove_scope(&self, id: Uuid) -> Result<bool> {
        Ok(mi_storage::remove_scope(self.pool, id).await?)
    }
}

/// RFC1918 / loopback / link-local addresses are auto-trusted — nothing
/// outside your own machine or LAN can be reached at these addresses, so
/// there's no third party to authorize against.
fn is_private_target(target: &str) -> bool {
    match IpAddr::from_str(target) {
        Ok(IpAddr::V4(v4)) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        // IPv6 unique-local (fc00::/7) detection is deliberately deferred —
        // loopback is the only IPv6 case we auto-trust for now, so an IPv6
        // private-range target will just require an explicit `mi scope add`.
        Ok(IpAddr::V6(v6)) => v6.is_loopback(),
        Err(_) => false, // domains are never auto-trusted
    }
}

fn infer_category(pattern: &str) -> ScopeCategory {
    if pattern.contains('/') {
        ScopeCategory::IpRange
    } else if IpAddr::from_str(pattern).is_ok() {
        ScopeCategory::Ip
    } else {
        ScopeCategory::Domain
    }
}

/// Matches a scope pattern against a target. Supports:
///   - exact string match ("example.com" == "example.com")
///   - wildcard subdomains ("*.example.com" matches "api.example.com")
///   - CIDR ranges ("203.0.113.0/24" matches "203.0.113.5")
fn scope_matches(pattern: &str, target: &str) -> bool {
    if pattern.eq_ignore_ascii_case(target) {
        return true;
    }

    if let Some(base) = pattern.strip_prefix("*.") {
        let base_lower = base.to_ascii_lowercase();
        let target_lower = target.to_ascii_lowercase();
        return target_lower == base_lower || target_lower.ends_with(&format!(".{base_lower}"));
    }

    if pattern.contains('/') {
        if let (Ok(network), Ok(addr)) = (IpNetwork::from_str(pattern), IpAddr::from_str(target)) {
            return network.contains(addr);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_ranges_are_auto_trusted() {
        assert!(is_private_target("192.168.1.50"));
        assert!(is_private_target("10.0.0.5"));
        assert!(is_private_target("172.16.0.1"));
        assert!(is_private_target("127.0.0.1"));
        assert!(!is_private_target("203.0.113.5"));
        assert!(!is_private_target("example.com"));
    }

    #[test]
    fn exact_match() {
        assert!(scope_matches("example.com", "example.com"));
        assert!(!scope_matches("example.com", "other.com"));
    }

    #[test]
    fn wildcard_subdomain_match() {
        assert!(scope_matches("*.example.com", "api.example.com"));
        assert!(scope_matches("*.example.com", "example.com"));
        assert!(!scope_matches("*.example.com", "example.org"));
    }

    #[test]
    fn cidr_match() {
        assert!(scope_matches("203.0.113.0/24", "203.0.113.5"));
        assert!(!scope_matches("203.0.113.0/24", "198.51.100.5"));
    }
}
