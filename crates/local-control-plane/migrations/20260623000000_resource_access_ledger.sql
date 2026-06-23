CREATE TABLE IF NOT EXISTS resource_access_log (
  id TEXT PRIMARY KEY,
  timestamp_ms INTEGER NOT NULL,
  agent_id TEXT NOT NULL,
  user_id TEXT,
  action TEXT NOT NULL,
  resource_uri TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  pep_type TEXT,
  pdp_runtime_id TEXT,
  policy_bundle_id TEXT,
  decision TEXT NOT NULL,
  reason TEXT,
  latency_ms INTEGER,
  redacted INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_resource_access_agent_time
ON resource_access_log(agent_id, timestamp_ms DESC);

CREATE INDEX IF NOT EXISTS idx_resource_access_resource_time
ON resource_access_log(resource_uri, timestamp_ms DESC);
