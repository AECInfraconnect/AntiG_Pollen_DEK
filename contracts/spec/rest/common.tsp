import "@typespec/http";
using TypeSpec.Http;
namespace Pollen.Contract;

model PollenHeaders {
  @header("X-Pollen-Contract-Version")
  contractVersion: string;

  @header("X-Pollen-Device-Id")
  deviceId?: string;

  @header("X-Pollen-Tenant-Id")
  tenantId?: string;
}

model ErrorEnvelope {
  schema_version: "error-envelope.v1";
  error: PollenError;
}

model PollenError {
  code: string;
  message: string;
  correlation_id?: string;
  details?: Record<unknown>;
}

model Page<T> {
  items: T[];
  next_cursor?: string;
}
