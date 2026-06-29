import "@typespec/http";
using TypeSpec.Http;
namespace Pollek.Contract;

model ContractDiscoveryResponse {
  schema_version: "contract-discovery.v1";
  supported: string[];
  preferred: string;
  minimum_dek_version?: string;
  sunset?: Record<string>;
  contract_version?: string;
  compatible_cloud_contracts?: string[];
  schemas?: Record<string>;
  interfaces?: Record<ContractInterface>;
  capabilities: string[];
}

model ContractInterface {
  schema?: string;
  direction: "local_to_dashboard" | "local_to_cloud" | "cloud_to_local" | "shared" | "bidirectional";
  hot_reload?: boolean;
  requires_spiffe?: boolean;
  requires_oauth?: boolean;
  requires_mtls?: boolean;
  tenant_scoped?: boolean;
  paths?: string[];
  controls?: string[];
  purpose?: string;
}

@tag("Discovery")
@route("/.well-known/pollek-contract")
interface ContractDiscoveryApi {
  @doc("Discover contract capabilities and supported protocol versions.")
  @get
  getDiscovery(...PollekHeaders): ContractDiscoveryResponse | PollekError;
}
