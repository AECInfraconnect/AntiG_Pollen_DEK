use anyhow::Result;
use dek_control_plane_api::registry::*;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[async_trait::async_trait]
pub trait RegistryStore: Send + Sync {
    async fn upsert_agent(&self, agent: AiAgent) -> Result<AiAgent>;
    async fn get_agent(&self, tenant_id: &str, agent_id: &str) -> Result<Option<AiAgent>>;
    async fn list_agents(&self, tenant_id: &str) -> Result<Vec<AiAgent>>;

    async fn upsert_entity(&self, entity: Entity) -> Result<Entity>;
    async fn list_entities(&self, tenant_id: &str) -> Result<Vec<Entity>>;

    async fn upsert_resource(&self, resource: Resource) -> Result<Resource>;
    async fn list_resources(&self, tenant_id: &str) -> Result<Vec<Resource>>;

    async fn upsert_tool(&self, tool: Tool) -> Result<Tool>;
    async fn list_tools(&self, tenant_id: &str) -> Result<Vec<Tool>>;

    async fn upsert_mcp_server(&self, server: McpServer) -> Result<McpServer>;
    async fn list_mcp_servers(&self, tenant_id: &str) -> Result<Vec<McpServer>>;

    async fn upsert_relationship(&self, relationship: Relationship) -> Result<Relationship>;
    async fn list_relationships(&self, tenant_id: &str) -> Result<Vec<Relationship>>;

    async fn upsert_policy_raw(&self, tenant: &str, id: &str, data: &serde_json::Value) -> Result<()>;
    async fn get_policy_raw(&self, tenant: &str, id: &str) -> Result<Option<serde_json::Value>>;
    async fn put_blob(&self, tenant: &str, path: &str, bytes: &[u8]) -> Result<()>;
    async fn get_blob(&self, tenant: &str, path: &str) -> Result<Option<Vec<u8>>>;
}

#[async_trait::async_trait]
pub trait TelemetryStore: Send + Sync {
    async fn put_telemetry(&self, tenant: &str, kind: &str, event_id: &str, data: &serde_json::Value) -> Result<()>;
    async fn list_telemetry(&self, tenant: &str, kind: &str) -> Result<Vec<serde_json::Value>>;
}

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    async fn upsert_object<T: Serialize>(
        &self,
        tenant_id: &str,
        object_type: &str,
        object_id: &str,
        status: &str,
        source: &str,
        data: &T,
    ) -> Result<()> {
        let json_data = serde_json::to_string(data)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO registry_objects (tenant_id, object_type, object_id, status, source, data_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(tenant_id, object_type, object_id) DO UPDATE SET
                status=excluded.status,
                source=excluded.source,
                data_json=excluded.data_json,
                updated_at=excluded.updated_at
            "#
        )
        .bind(tenant_id)
        .bind(object_type)
        .bind(object_id)
        .bind(status)
        .bind(source)
        .bind(json_data)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_object<T: for<'de> Deserialize<'de>>(
        &self,
        tenant_id: &str,
        object_type: &str,
        object_id: &str,
    ) -> Result<Option<T>> {
        let row = sqlx::query("SELECT data_json FROM registry_objects WHERE tenant_id = ?1 AND object_type = ?2 AND object_id = ?3")
            .bind(tenant_id)
            .bind(object_type)
            .bind(object_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let data_json: String = row.try_get("data_json")?;
            let obj: T = serde_json::from_str(&data_json)?;
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    async fn list_objects<T: for<'de> Deserialize<'de>>(
        &self,
        tenant_id: &str,
        object_type: &str,
    ) -> Result<Vec<T>> {
        let rows = sqlx::query(
            "SELECT data_json FROM registry_objects WHERE tenant_id = ?1 AND object_type = ?2",
        )
        .bind(tenant_id)
        .bind(object_type)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let data_json: String = row.try_get("data_json")?;
            let obj: T = serde_json::from_str(&data_json)?;
            results.push(obj);
        }
        Ok(results)
    }
}

#[async_trait::async_trait]
impl RegistryStore for SqliteStore {
    async fn upsert_agent(&self, agent: AiAgent) -> Result<AiAgent> {
        let status = serde_json::to_string(&agent.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&agent.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &agent.meta.tenant_id,
            "agent",
            &agent.agent_id,
            &status,
            &source,
            &agent,
        )
        .await?;
        Ok(agent)
    }

    async fn get_agent(&self, tenant_id: &str, agent_id: &str) -> Result<Option<AiAgent>> {
        self.get_object(tenant_id, "agent", agent_id).await
    }

    async fn list_agents(&self, tenant_id: &str) -> Result<Vec<AiAgent>> {
        self.list_objects(tenant_id, "agent").await
    }

    async fn upsert_entity(&self, entity: Entity) -> Result<Entity> {
        let status = serde_json::to_string(&entity.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&entity.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &entity.meta.tenant_id,
            "entity",
            &entity.entity_id,
            &status,
            &source,
            &entity,
        )
        .await?;
        Ok(entity)
    }

    async fn list_entities(&self, tenant_id: &str) -> Result<Vec<Entity>> {
        self.list_objects(tenant_id, "entity").await
    }

    async fn upsert_resource(&self, resource: Resource) -> Result<Resource> {
        let status = serde_json::to_string(&resource.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&resource.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &resource.meta.tenant_id,
            "resource",
            &resource.resource_id,
            &status,
            &source,
            &resource,
        )
        .await?;
        Ok(resource)
    }

    async fn list_resources(&self, tenant_id: &str) -> Result<Vec<Resource>> {
        self.list_objects(tenant_id, "resource").await
    }

    async fn upsert_tool(&self, tool: Tool) -> Result<Tool> {
        let status = serde_json::to_string(&tool.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&tool.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &tool.meta.tenant_id,
            "tool",
            &tool.tool_id,
            &status,
            &source,
            &tool,
        )
        .await?;
        Ok(tool)
    }

    async fn list_tools(&self, tenant_id: &str) -> Result<Vec<Tool>> {
        self.list_objects(tenant_id, "tool").await
    }

    async fn upsert_mcp_server(&self, server: McpServer) -> Result<McpServer> {
        let status = serde_json::to_string(&server.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&server.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &server.meta.tenant_id,
            "mcp_server",
            &server.server_id,
            &status,
            &source,
            &server,
        )
        .await?;
        Ok(server)
    }

    async fn list_mcp_servers(&self, tenant_id: &str) -> Result<Vec<McpServer>> {
        self.list_objects(tenant_id, "mcp_server").await
    }

    async fn upsert_relationship(&self, relationship: Relationship) -> Result<Relationship> {
        let status = serde_json::to_string(&relationship.meta.status)
            .unwrap()
            .replace("\"", "");
        let source = serde_json::to_string(&relationship.meta.source)
            .unwrap()
            .replace("\"", "");
        self.upsert_object(
            &relationship.meta.tenant_id,
            "relationship",
            &relationship.relationship_id,
            &status,
            &source,
            &relationship,
        )
        .await?;
        Ok(relationship)
    }

    async fn list_relationships(&self, tenant_id: &str) -> Result<Vec<Relationship>> {
        self.list_objects(tenant_id, "relationship").await
    }

    async fn upsert_policy_raw(&self, tenant: &str, id: &str, data: &serde_json::Value) -> Result<()> {
        self.upsert_object(tenant, "policy", id, "active", "local", data).await
    }

    async fn get_policy_raw(&self, tenant: &str, id: &str) -> Result<Option<serde_json::Value>> {
        self.get_object(tenant, "policy", id).await
    }

    async fn put_blob(&self, tenant: &str, path: &str, bytes: &[u8]) -> Result<()> {
        sqlx::query(
            "INSERT INTO bundle_blobs (tenant_id, path, bytes) VALUES (?1, ?2, ?3) ON CONFLICT(tenant_id, path) DO UPDATE SET bytes=excluded.bytes"
        )
        .bind(tenant)
        .bind(path)
        .bind(bytes)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_blob(&self, tenant: &str, path: &str) -> Result<Option<Vec<u8>>> {
        let row = sqlx::query("SELECT bytes FROM bundle_blobs WHERE tenant_id = ?1 AND path = ?2")
            .bind(tenant)
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;
        if let Some(row) = row {
            let bytes: Vec<u8> = row.try_get("bytes")?;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl TelemetryStore for SqliteStore {
    async fn put_telemetry(&self, tenant: &str, kind: &str, event_id: &str, data: &serde_json::Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO telemetry_events (tenant_id, event_type, event_id, data_json, created_at)
             VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(tenant_id,event_id) DO UPDATE SET data_json=?4"
        )
        .bind(tenant).bind(kind).bind(event_id)
        .bind(serde_json::to_string(data)?)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }
    async fn list_telemetry(&self, tenant: &str, kind: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query(
            "SELECT data_json FROM telemetry_events WHERE tenant_id=?1 AND event_type=?2 ORDER BY created_at DESC LIMIT 1000")
            .bind(tenant).bind(kind).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().filter_map(|r| {
            let j: String = r.try_get("data_json").ok()?;
            serde_json::from_str(&j).ok()
        }).collect())
    }
}
