CREATE TABLE telemetry_events (
    tenant_id TEXT,
    event_type TEXT,
    event_id TEXT,
    data_json TEXT,
    created_at TEXT,
    PRIMARY KEY(tenant_id, event_id)
);
