import "@typespec/http";
using TypeSpec.Http;
namespace Pollen.Contract;

model TelemetryBatchRequest {
  schema_version: "telemetry-batch.v1";
  tenant_id: string;
  device_id: string;
  batch_id: string;
  events: unknown[];
}

model TelemetryIngestResponse {
  schema_version: "telemetry-ingest-response.v1";
  accepted: int32;
  rejected: int32;
  retry_after_seconds?: int32;
}

@route("/v1/telemetry/batches")
interface TelemetryApi {
  @post
  ingestBatch(
    ...PollenHeaders,
    @body body: TelemetryBatchRequest
  ): TelemetryIngestResponse | ErrorEnvelope;
}
