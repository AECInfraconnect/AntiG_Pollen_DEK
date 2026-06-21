import type { AiAgent, McpServer, Tool, Resource, Entity, Relationship, PolicyDraft, TelemetryEventEnvelope, BlackboxAiProvider } from './types';
export type * from './types';

export class ControlPlaneClient {
  public baseUrl: string;
  public tenantId: string;
  public mockRole: string;

  constructor(profile: 'local' | 'mock-cloud' = 'local') {
    if (profile === 'mock-cloud') {
      this.baseUrl = 'http://localhost:43891/v1/tenants/local';
      this.mockRole = 'admin';
    } else {
      this.baseUrl = 'http://localhost:43890/v1/tenants/local';
      this.mockRole = '';
    }
    this.tenantId = 'local';
  }

  private async fetchApi(path: string, options?: RequestInit) {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (this.mockRole) {
      headers['x-mock-role'] = this.mockRole;
    }
    
    const res = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers: {
        ...headers,
        ...options?.headers,
      }
    });
    if (!res.ok) {
      throw new Error(await res.text());
    }
    return res.json();
  }

  // Registry
  async listAgents(): Promise<AiAgent[]> { return this.fetchApi('/registry/agents'); }
  async listMcpServers(): Promise<McpServer[]> { return this.fetchApi('/registry/mcp_servers'); }
  async listTools(): Promise<Tool[]> { return this.fetchApi('/registry/tools'); }
  async listResources(): Promise<Resource[]> { return this.fetchApi('/registry/resources'); }
  async listEntities(): Promise<Entity[]> { return this.fetchApi('/registry/entities'); }
  async listRelationships(): Promise<Relationship[]> { return this.fetchApi('/registry/relationships'); }
  async listBlackboxAiProviders(): Promise<BlackboxAiProvider[]> { return this.fetchApi('/registry/blackbox_ai_providers'); }
  
  // Policies
  async listPolicies(): Promise<PolicyDraft[]> { return this.fetchApi('/policies'); }
  async createPolicy(draft: PolicyDraft): Promise<PolicyDraft> {
    return this.fetchApi('/policies', { method: 'POST', body: JSON.stringify(draft) });
  }
  async publishPolicy(policyId: string): Promise<{ published: boolean; bundle_id: string; build_number: number }> {
    return this.fetchApi(`/policies/${policyId}/publish`, { method: 'POST' });
  }

  async simulatePolicy(req: any): Promise<any> {
    return this.fetchApi('/policies/simulate', { method: 'POST', body: JSON.stringify(req) });
  }

  // Bundles
  async listBundles(): Promise<any[]> {
    return this.fetchApi('/bundles');
  }
  
  async pushSync(): Promise<any> {
    // Note: the mock server push is a stream, but we might just hit a sync endpoint.
    // Assuming /bundles/sync or just let the dashboard know it triggers a reload
    return this.fetchApi('/bundles/sync', { method: 'POST' });
  }

  // Telemetry
  async listDecisionLogs(): Promise<TelemetryEventEnvelope[]> {
    const data = await this.fetchApi('/telemetry/decision-logs');
    return data.decisions ?? data;
  }
}

// Store the active profile in localStorage to persist across reloads
const getStoredProfile = (): 'local' | 'mock-cloud' => {
  const p = localStorage.getItem('dek_admin_profile');
  if (p === 'mock-cloud') return 'mock-cloud';
  return 'local';
};

// Global default client
export const defaultClient = new ControlPlaneClient(getStoredProfile());

// Helper to switch profile
export const switchProfile = (profile: 'local' | 'mock-cloud') => {
  localStorage.setItem('dek_admin_profile', profile);
  window.location.reload();
};

// Proxy objects for backward compatibility with existing code
export const RegistryApi = {
  listAgents: () => defaultClient.listAgents(),
  listMcpServers: () => defaultClient.listMcpServers(),
  listTools: () => defaultClient.listTools(),
  listResources: () => defaultClient.listResources(),
  listEntities: () => defaultClient.listEntities(),
  listRelationships: () => defaultClient.listRelationships(),
  listBlackboxAiProviders: () => defaultClient.listBlackboxAiProviders(),
};

export const PolicyApi = {
  list: () => defaultClient.listPolicies(),
  create: (draft: PolicyDraft) => defaultClient.createPolicy(draft),
  publish: (policyId: string) => defaultClient.publishPolicy(policyId),
  simulate: (req: any) => defaultClient.simulatePolicy(req),
};

export const BundleApi = {
  list: () => defaultClient.listBundles(),
  sync: () => defaultClient.pushSync(),
};

export const TelemetryApi = {
  listDecisionLogs: () => defaultClient.listDecisionLogs(),
};
