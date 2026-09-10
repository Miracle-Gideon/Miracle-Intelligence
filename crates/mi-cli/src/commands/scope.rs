use anyhow::Result;
use chrono::{DateTime, Utc};
use mi_core::ScopeGuard;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::cli::ScopeAction;

pub async fn handle(pool: &SqlitePool, action: ScopeAction) -> Result<()> {
    let guard = ScopeGuard::new(pool);

    match action {
        ScopeAction::Add {
            pattern,
            authorized_by,
            no_expiry,
            until,
        } => {
            let until_dt: Option<DateTime<Utc>> = until
                .map(|s| {
                    DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc))
                })
                .transpose()?;

            let scope = guard
                .add_scope(&pattern, &authorized_by, no_expiry, until_dt)
                .await?;

            println!(
                "Added scope: {}  [{}]  authorized by \"{}\"",
                scope.pattern,
                scope.category.as_str(),
                scope.authorized_by
            );
        }
        ScopeAction::List => {
            let scopes = guard.list_scopes().await?;
            if scopes.is_empty() {
                println!("No scopes registered.");
            } else {
                for s in scopes {
                    let expiry = s
                        .authorized_until
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_else(|| "never".to_string());
                    println!(
                        "{}  {:<9}  {:<30}  by: {:<20}  expires: {}",
                        s.id,
                        s.category.as_str(),
                        s.pattern,
                        s.authorized_by,
                        expiry
                    );
                }
            }
        }
        ScopeAction::Remove { id } => {
            let uuid = Uuid::parse_str(&id)?;
            if guard.remove_scope(uuid).await? {
                println!("Removed scope {id}");
            } else {
                println!("No scope found with id {id}");
            }
        }
    }

    Ok(())
}
