import "@typespec/http";
import "@typespec/rest";
using TypeSpec.Http;
using TypeSpec.Rest;

namespace Pollen.Contract;

@tag("LocalDashboard")
@route("/v1/tenants/local")
interface LocalDashboardApi {
  @doc("Start Local Scan Session")
  @route("scan")
  @post startLocalScanSession(...PollenHeaders): void | PollenError;

  @doc("Get Local Scan Session by ID")
  @route("scans/{scan_id}")
  @get getLocalScanSession(@path scan_id: string, ...PollenHeaders): unknown | PollenError;

  @doc("Stream Local Scan Session Events (SSE)")
  @route("scans/{scan_id}/events")
  @get streamLocalScanSessionEvents(@path scan_id: string, ...PollenHeaders): void | PollenError;

  @doc("Get latest local capability snapshot")
  @route("capability-snapshot")
  @get getLocalCapabilitySnapshot(...PollenHeaders): unknown | PollenError;

  @doc("List suggested policies based on capabilities and discovered agents")
  @route("policy-suggestions")
  @get listLocalPolicySuggestions(...PollenHeaders): unknown | PollenError;

  @doc("Check policy feasibility")
  @route("policies/feasibility")
  @post checkLocalPolicyFeasibility(...PollenHeaders, @body body: unknown): unknown | PollenError;

  @doc("Create a deployment session")
  @route("deployment-sessions")
  @post createLocalDeploymentSession(...PollenHeaders, @body body: unknown): unknown | PollenError;

  @doc("Get deployment session")
  @route("deployment-sessions/{id}")
  @get getLocalDeploymentSession(@path id: string, ...PollenHeaders): unknown | PollenError;

  @doc("Stream deployment session events (SSE)")
  @route("deployment-sessions/{id}/events")
  @get streamLocalDeploymentSessionEvents(@path id: string, ...PollenHeaders): void | PollenError;

  @doc("Approve an action in a deployment session")
  @route("deployment-sessions/{id}/actions/{action_id}/approve")
  @post approveLocalDeploymentAction(@path id: string, @path action_id: string, ...PollenHeaders): void | PollenError;

  @doc("Get health of local enforcement layer")
  @route("health/local-enforcement")
  @get getLocalEnforcementHealth(...PollenHeaders): unknown | PollenError;
}
