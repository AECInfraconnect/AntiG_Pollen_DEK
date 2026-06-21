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
}
