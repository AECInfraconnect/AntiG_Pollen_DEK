import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi";
import "./rest/common.tsp";
import "./rest/contract-discovery.tsp";
import "./rest/bundles.tsp";
import "./rest/telemetry.tsp";
import "./rest/registry.tsp";
import "./rest/enrollment.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;
using TypeSpec.OpenAPI;

@service(#{ title: "Pollen Cloud-DEK Contract" })
@info(#{ version: "1.0" })
@server("https://api.pollen.cloud", "Pollen Cloud")
@server("http://127.0.0.1:17891", "Local Control Plane / Mock Cloud")
namespace Pollen.Contract;
