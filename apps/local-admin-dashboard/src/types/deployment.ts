export type LocalizedText = {
  en: string;
  th: string;
};

export enum DeploymentPhase {
  AgentDiscovery = 'agent_discovery',
  RoutePlanning = 'route_planning',
  PepDeploy = 'pep_deploy',
  Enforcement = 'enforcement',
  Rollback = 'rollback',
}

export enum EventStatus {
  Info = 'info',
  Success = 'success',
  Warning = 'warning',
  Error = 'error',
}

export type UserActionKind = 'RequireAuth' | 'RequireApproval' | 'RequireConfig';

export type UserAction = {
  kind: UserActionKind;
  action_url: string;
  expires_at?: string;
};

export type DeploymentEvent = {
  event_id: string;
  deployment_id: string;
  agent_id?: string;
  entity_id?: string;
  policy_id: string;
  phase: DeploymentPhase;
  status: EventStatus;
  title: LocalizedText;
  detail: LocalizedText;
  technical_detail?: string;
  user_action?: UserAction;
  created_at: string;
  correlation_id: string;
};

export enum DeploymentSessionStatus {
  Draft = 'draft',
  Planning = 'planning',
  Deploying = 'deploying',
  WaitingForUserAction = 'waiting_for_user_action',
  Active = 'active',
  PartiallyActive = 'partially_active',
  ActiveObserveOnly = 'active_observe_only',
  Failed = 'failed',
  RolledBack = 'rolled_back',
}

export type EnforcementLayer =
  | 'browser_extension'
  | 'macos_network_extension'
  | 'windows_wfp'
  | 'ebpf_network'
  | 'mcp_proxy'
  | 'mcp_stdio_wrapper'
  | 'http_proxy'
  | 'observe_only';

export type PdpEngine = 'OpenFga' | 'Cedar' | 'CloudAuthz';

export type RoutingPlan = {
  selected_pep: {
    layer: EnforcementLayer;
    name: LocalizedText;
  };
  selected_pdp: {
    engine: PdpEngine;
  };
  fallback_pep?: {
    layer: EnforcementLayer;
    name: LocalizedText;
  };
};

export type DeploymentSession = {
  deployment_id: string;
  policy_id: string;
  status: DeploymentSessionStatus;
  routing_plan?: RoutingPlan;
  created_at: string;
  updated_at: string;
};
