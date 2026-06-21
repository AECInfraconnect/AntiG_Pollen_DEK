CREATE TABLE IF NOT EXISTS registry_objects (
  tenant_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL DEFAULT 'default',
  environment_id TEXT NOT NULL DEFAULT 'local',
  object_type TEXT NOT NULL,
  object_id TEXT NOT NULL,
  status TEXT NOT NULL,
  source TEXT NOT NULL,
  data_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (tenant_id, object_type, object_id)
);

CREATE INDEX IF NOT EXISTS idx_registry_objects_type_status
ON registry_objects (tenant_id, object_type, status);

CREATE TABLE IF NOT EXISTS policies (
  tenant_id TEXT NOT NULL,
  policy_id TEXT NOT NULL,
  status TEXT NOT NULL,
  data_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (tenant_id, policy_id)
);

CREATE TABLE IF NOT EXISTS bundles (
  tenant_id TEXT NOT NULL,
  bundle_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  data_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (tenant_id, device_id, bundle_id)
);

CREATE TABLE IF NOT EXISTS telemetry (
  tenant_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  device_id TEXT NOT NULL,
  data_json TEXT NOT NULL,
  PRIMARY KEY (tenant_id, event_id)
);
