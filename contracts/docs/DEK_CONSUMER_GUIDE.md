# DEK Consumer Guide

DEK acts as the client for Pollek Cloud and the Local Control Plane. It must adhere to the contracts provided here.

1. Implement Contract Discovery negotiation to handle version mismatch.
2. Provide fallback mechanisms (GracePeriod/LKG) when contract validation fails.
3. Verify downloaded bundle signatures using `bundle-signature.v1.schema.json`.
4. Run Consumer Contract Tests against generated Pact templates.
