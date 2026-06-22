CREATE TABLE IF NOT EXISTS agent_inventory (
  tenant TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  inventory_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, agent_id)
);

CREATE TABLE IF NOT EXISTS control_bindings (
  tenant TEXT NOT NULL,
  binding_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  pep_type TEXT NOT NULL,
  action TEXT NOT NULL,
  status TEXT NOT NULL,
  config_backup_id TEXT,
  binding_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, binding_id)
);

CREATE TABLE IF NOT EXISTS policy_deployments (
  tenant TEXT NOT NULL,
  deployment_id TEXT NOT NULL,
  status TEXT NOT NULL,
  deployment_json TEXT NOT NULL,
  rollback_snapshot_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, deployment_id)
);

CREATE TABLE IF NOT EXISTS tool_invocation_logs (
  tenant TEXT NOT NULL,
  event_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  tool_id TEXT,
  event_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, event_id)
);

CREATE TABLE IF NOT EXISTS resource_access_logs (
  tenant TEXT NOT NULL,
  event_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  resource_id TEXT,
  event_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, event_id)
);
