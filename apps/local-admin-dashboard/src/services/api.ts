// Pollen DEK Registry API Models

export interface ObjectMeta {
  schema_version: string;
  tenant_id: string;
  workspace_id: string;
  environment_id: string;
  created_at: string;
  updated_at: string;
  created_by: string;
  updated_by: string;
  source: 'manual' | 'discovery' | 'import' | 'cloud_sync' | 'agent_self_registration';
  status: 'discovered' | 'pending_approval' | 'registered' | 'active' | 'suspended' | 'deleted';
  tags: string[];
}

export interface AiAgent {
  meta: ObjectMeta;
  agent_id: string;
  name: string;
  agent_type: 'claude_desktop' | 'openai_agent' | 'langchain_agent' | 'llama_index_agent' | 'custom_mcp_client' | 'browser_agent' | 'cli_agent' | 'unknown';
  vendor?: string;
  runtime: {
    runtime_name: string;
    version?: string;
  };
  entrypoints: {
    command: string;
    args: string[];
  }[];
  declared_tools: string[];
  declared_resources: string[];
  identity: {
    spiffe_id?: string;
    process_path?: string;
    user_subject?: string;
    signing_key_fingerprint?: string;
  };
  trust_level: 'untrusted' | 'low' | 'medium' | 'high' | 'system';
  capabilities: string[];
  labels: Record<string, string>;
}

export interface McpServer {
  meta: ObjectMeta;
  server_id: string;
  name: string;
  transport: 'stdio' | 'http' | 'sse' | 'web_socket';
  endpoint: string;
  owner_agent_id?: string;
  tools: string[];
  resources: string[];
  risk_level: 'low' | 'medium' | 'high' | 'critical';
}

export interface Tool {
  meta: ObjectMeta;
  tool_id: string;
  mcp_server_id?: string;
  name: string;
  description?: string;
  input_schema: any;
  output_schema?: any;
  side_effect_level: 'none' | 'local' | 'network' | 'system';
  data_access_level: 'none' | 'public' | 'internal' | 'confidential' | 'restricted';
  risk_level: 'low' | 'medium' | 'high' | 'critical';
}

export interface Resource {
  meta: ObjectMeta;
  resource_id: string;
  resource_type: 'file' | 'database' | 'api_endpoint' | 'mcp_resource' | 'vector_store' | 'topic' | 'queue' | 'device' | 'secret' | 'unknown';
  name: string;
  uri: string;
  classification: 'public' | 'internal' | 'confidential' | 'restricted';
  owner_entity_id?: string;
  attributes: Record<string, any>;
}

export interface Entity {
  meta: ObjectMeta;
  entity_id: string;
  entity_type: 'human_user' | 'service_account' | 'workload' | 'ai_agent' | 'organization' | 'tenant' | 'device';
  display_name: string;
  external_ids: { provider: string; id: string }[];
  roles: string[];
  attributes: Record<string, any>;
}

export interface Relationship {
  meta: ObjectMeta;
  relationship_id: string;
  subject: { object_type: string; object_id: string };
  relation: string;
  object: { object_type: string; object_id: string };
  conditions?: any;
}

const API_BASE = '/v1/tenants/local/registry';

export const RegistryApi = {
  listAgents: async (): Promise<AiAgent[]> => {
    const res = await fetch(`${API_BASE}/agents`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  listMcpServers: async (): Promise<McpServer[]> => {
    const res = await fetch(`${API_BASE}/mcp-servers`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  listTools: async (): Promise<Tool[]> => {
    const res = await fetch(`${API_BASE}/tools`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  listResources: async (): Promise<Resource[]> => {
    const res = await fetch(`${API_BASE}/resources`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  listEntities: async (): Promise<Entity[]> => {
    const res = await fetch(`${API_BASE}/entities`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  listRelationships: async (): Promise<Relationship[]> => {
    const res = await fetch(`${API_BASE}/relationships`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }
};
