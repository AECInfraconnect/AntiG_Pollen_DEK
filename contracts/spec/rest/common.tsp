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

@error
model PollenError {
  code: string;
  message: string;
  trace_id?: string;
}

model NotModifiedResponse { @statusCode statusCode: 304; }

model Page<T> {
  items: T[];
  next_cursor?: string;
}
