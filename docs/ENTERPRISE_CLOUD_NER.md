# Enterprise Cloud Third-Party NER

Pollek OSS Local Control Plane keeps NER disabled by default. Enterprise Cloud can enable a third-party NER provider by sending guard configuration to the local control plane over the existing secure policy/config channel.

The local guard pipeline accepts this shape:

```json
{
  "enable_ner": true,
  "ner_provider": {
    "provider_id": "enterprise-gliner",
    "provider_kind": "custom_http",
    "endpoint": "http://127.0.0.1:8080/ner",
    "labels": ["person", "address", "organization"],
    "min_confidence": 0.8,
    "timeout_ms": 80
  }
}
```

Provider behavior:

- `provider_kind: "gliner"` or `"custom_http"` uses the GLiNER-compatible HTTP adapter.
- The request body is `{ "text": "...", "labels": [...] }`.
- The response may be either an array or `{ "entities": [...] }`.
- Each entity must include `label`, `start`, `end`, and `score`.
- Endpoint errors, timeouts, non-2xx responses, and invalid JSON return an empty span set. The pipeline does not treat a failed NER call as evidence that content is safe.

Recommended Enterprise Cloud operation:

- Keep `enable_ner` off for Simple and Advanced local modes.
- Enable it only after Enterprise Cloud has verified the endpoint health and policy scope.
- Prefer a local sidecar on the same host or private network for regulated workloads.
- Do not place provider API keys in the guard config. Use local sidecar auth, workload identity, or secret injection owned by the deployment layer.
