-- 20260609000001_bundle_blobs.sql
CREATE TABLE IF NOT EXISTS bundle_blobs (
    tenant_id TEXT NOT NULL,
    path TEXT NOT NULL,
    bytes BLOB NOT NULL,
    PRIMARY KEY (tenant_id, path)
);
