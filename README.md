# Miracle Intelligence (MI) — Phase 1

Phase 1 scope: workspace skeleton, core data model, SQLite storage, the
ScopeGuard authorization check, a basic Shodan provider, and a CLI shell
(`scope`, `config`, `status`). No exposure engine, risk engine, correlation,
or verification adapters yet — those are later phases.

## Crates

| Crate | Purpose |
|---|---|
| `mi-models` | Domain types (Asset, Observation, Scope, Finding) — no storage/network deps |
| `mi-storage` | SQLite persistence + repository pattern, migrations |
| `mi-core` | `ScopeGuard` — the centralized authorization check |
| `mi-providers` | `Provider` trait + Shodan implementation |
| `mi-cli` | The `mi` binary |

## Building

This is built via GitHub Actions (`.github/workflows/build.yml`), so pushing
to any branch triggers a build + test run and uploads the compiled `mi`
binary as an artifact — no local compile needed.

To build locally instead, on a machine with Rust installed:

```bash
cargo build --workspace
cargo test --workspace
```

## Configuration (BYOK)

No API key ships with MI. Set your own Shodan key either via `mi.toml`:

```toml
shodan_api_key = "your-key-here"
db_path = "mi.db"
```

or an environment variable:

```bash
export MI_SHODAN_API_KEY="your-key-here"
```

`mi.toml` is gitignored — never commit it.

## Usage

```bash
# Check DB connectivity, counts, and whether your Shodan key is valid
mi status

# Private ranges (10.x, 172.16-31.x, 192.168.x, loopback) never need scope —
# they're auto-trusted since nothing outside your network can reach them.

# Anything else needs explicit authorization first:
mi scope add example.com --authorized-by "SOW-2026-04"
mi scope add 203.0.113.0/24 --authorized-by "authorized pentest range" --no-expiry
mi scope list
mi scope remove <id>

# Inspect resolved config (secrets masked)
mi config show
mi config path
```

## What's deliberately NOT here yet

- Exposure/risk engine, attack-path correlation, ATT&CK tagging → Phase 3
- Verification adapters (nmap/sqlmap/Metasploit-RPC orchestration) → Phase 3+,
  gated behind `ScopeGuard` the same way everything else is
- Encrypted multi-key API pool with automatic failover → Phase 2 (Phase 1
  reads a single key straight from config/env to keep this phase small)
- Forensics/PCAP/IOC engine → later phase
