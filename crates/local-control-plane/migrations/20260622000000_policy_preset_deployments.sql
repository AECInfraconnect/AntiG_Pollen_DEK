CREATE TABLE policy_preset_deployments (
    tenant_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    preset_id TEXT NOT NULL,
    preset_version TEXT,
    control_mode TEXT NOT NULL,
    status TEXT NOT NULL,
    target_scopes_json TEXT NOT NULL,
    parameters_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, deployment_id)
);

CREATE TABLE pep_bindings (
    tenant_id TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    pep_type TEXT NOT NULL,
    config_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, binding_id),
    FOREIGN KEY (tenant_id, deployment_id) REFERENCES policy_preset_deployments (tenant_id, deployment_id) ON DELETE CASCADE
);