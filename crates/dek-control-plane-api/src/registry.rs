use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObjectMeta {
    pub schema_version: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub environment_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub updated_by: String,
    pub source: RegistrationSource,
    pub status: RegistryStatus,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationSource {
    Manual,
    Discovery,
    Import,
    CloudSync,
    AgentSelfRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegistryStatus {
    Discovered,
    PendingApproval,
    Registered,
    Active,
    Suspended,
    Deleted,
    Draft,
    Compiled,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiAgent {
    pub meta: ObjectMeta,
    pub agent_id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub vendor: Option<String>,
    pub runtime: AgentRuntime,
    pub entrypoints: Vec<AgentEntrypoint>,
    pub declared_tools: Vec<String>,
    pub declared_resources: Vec<String>,
    pub identity: AgentIdentity,
    pub trust_level: TrustLevel,
    pub capabilities: Vec<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeDesktop,
    OpenAIAgent,
    LangChainAgent,
    LlamaIndexAgent,
    CustomMcpClient,
    BrowserAgent,
    CliAgent,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentRuntime {
    pub runtime_name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentEntrypoint {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentIdentity {
    pub spiffe_id: Option<String>,
    pub process_path: Option<String>,
    pub user_subject: Option<String>,
    pub signing_key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Untrusted,
    Low,
    Medium,
    High,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServer {
    pub meta: ObjectMeta,
    pub server_id: String,
    pub name: String,
    pub transport: McpTransport,
    pub endpoint: String,
    pub owner_agent_id: Option<String>,
    pub tools: Vec<String>,
    pub resources: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum McpTransport {
    Stdio,
    Http,
    Sse,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Tool {
    pub meta: ObjectMeta,
    pub tool_id: String,
    pub mcp_server_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub side_effect_level: SideEffectLevel,
    pub data_access_level: DataAccessLevel,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectLevel {
    None,
    Local,
    Network,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataAccessLevel {
    None,
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Resource {
    pub meta: ObjectMeta,
    pub resource_id: String,
    pub resource_type: ResourceType,
    pub name: String,
    pub uri: String,
    pub classification: DataClassification,
    pub owner_entity_id: Option<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    File,
    Database,
    ApiEndpoint,
    McpResource,
    VectorStore,
    Topic,
    Queue,
    Device,
    Secret,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub meta: ObjectMeta,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub display_name: String,
    pub external_ids: Vec<ExternalId>,
    pub roles: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    HumanUser,
    ServiceAccount,
    Workload,
    AiAgent,
    Organization,
    Tenant,
    Device,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalId {
    pub provider: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    pub meta: ObjectMeta,
    pub relationship_id: String,
    pub subject: RelationshipRef,
    pub relation: String,
    pub object: RelationshipRef,
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipRef {
    pub object_type: String,
    pub object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlackboxProviderType {
    OpenAiCompatible,
    Ollama,
    HuggingFaceEndpoint,
    AzureOpenAi,
    AnthropicCompatible,
    LocalModelServer,
    CustomHttp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlackboxModelRef {
    pub model_id: String,
    pub display_name: String,
    pub context_window: Option<u32>,
    pub pii_allowed: bool,
    pub max_latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataBoundary {
    LocalOnly,
    PrivateNetwork,
    ExternalCloud,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlackboxAiProvider {
    pub meta: ObjectMeta,
    pub provider_id: String,
    pub name: String,
    pub provider_type: BlackboxProviderType,
    pub endpoint: Option<String>,
    pub model_catalog: Vec<BlackboxModelRef>,
    pub supported_tasks: Vec<String>,
    pub data_boundary: DataBoundary,
    pub auth_ref: Option<String>,
    pub risk_level: RiskLevel,
    pub labels: HashMap<String, String>,
}
