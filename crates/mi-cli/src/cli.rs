use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mi", version, about = "Miracle Intelligence — security intelligence & forensics platform")]
pub struct Cli {
    /// Increase log detail; repeatable (-v, -vv, -vvv)
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Alternate config file path
    #[arg(short = 'c', long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage the authorization scope registry
    Scope {
        #[command(subcommand)]
        action: ScopeAction,
    },
    /// Inspect resolved configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
        Status,
}    Status,
}/// Show DB connectivity, counts, and API key health
    Status,
    /// Look up a single host via Shodan and store the result
    Host {
        /// IP address to look up
        ip: String,
    },
}

#[derive(Subcommand)]
pub enum ScopeAction {
    /// Add a target (domain, IP, or CIDR) to the scope registry
    Add {
        pattern: String,

        /// Why this target is authorized (client name, SOW ref, verbal auth details...)
        #[arg(short = 'A', long = "authorized-by")]
        authorized_by: String,

        /// Standing entry with no expiry (e.g. a home lab range)
        #[arg(long)]
        no_expiry: bool,

        /// RFC3339 expiry timestamp, e.g. 2026-12-31T23:59:59Z
        #[arg(long)]
        until: Option<String>,
    },
    /// List all registered scope entries
    List,
    /// Remove a scope entry by id
    Remove { id: String },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print resolved configuration (secrets masked)
    Show,
    /// Print where configuration is loaded from
    Path,
}
