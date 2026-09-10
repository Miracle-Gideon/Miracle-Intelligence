CREATE TABLE IF NOT EXISTS assets (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL,
    value       TEXT NOT NULL,
    org_id      TEXT,
    confidence  TEXT NOT NULL,
    first_seen  TEXT NOT NULL,
    last_seen   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assets_value ON assets(value);

CREATE TABLE IF NOT EXISTS observations (
    id          TEXT PRIMARY KEY,
    asset_id    TEXT NOT NULL REFERENCES assets(id),
    provider    TEXT NOT NULL,
    raw         TEXT NOT NULL,
    normalized  TEXT NOT NULL,
    observed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_observations_asset ON observations(asset_id);

CREATE TABLE IF NOT EXISTS scopes (
    id                TEXT PRIMARY KEY,
    pattern           TEXT NOT NULL,
    category          TEXT NOT NULL,
    status            TEXT NOT NULL,
    authorized_by     TEXT NOT NULL,
    authorized_until  TEXT,
    created_at        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_scopes_pattern ON scopes(pattern);

-- Not used until Phase 3 (exposure/risk engine); created now so the
-- schema doesn't need a breaking migration later.
CREATE TABLE IF NOT EXISTS findings (
    id                 TEXT PRIMARY KEY,
    asset_id           TEXT NOT NULL REFERENCES assets(id),
    title              TEXT NOT NULL,
    severity           TEXT NOT NULL,
    status             TEXT NOT NULL,
    confidence         TEXT NOT NULL,
    attack_techniques  TEXT NOT NULL DEFAULT '[]',
    first_seen         TEXT NOT NULL,
    last_seen          TEXT NOT NULL,
    recommendation     TEXT NOT NULL DEFAULT ''
);
