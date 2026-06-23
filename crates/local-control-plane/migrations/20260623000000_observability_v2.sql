ALTER TABLE observation_events ADD COLUMN event_kind TEXT;
ALTER TABLE observation_events ADD COLUMN provider TEXT;
ALTER TABLE observation_events ADD COLUMN input_tokens INTEGER;
ALTER TABLE observation_events ADD COLUMN output_tokens INTEGER;
ALTER TABLE observation_events ADD COLUMN total_tokens INTEGER;
ALTER TABLE observation_events ADD COLUMN latency_ms INTEGER;

CREATE INDEX IF NOT EXISTS idx_obs_agent_time ON observation_events(tenant_id, agent_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_obs_kind ON observation_events(tenant_id, event_kind);
CREATE INDEX IF NOT EXISTS idx_cost_agent ON cost_ledger(agent_id, timestamp);
