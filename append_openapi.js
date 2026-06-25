const fs = require("fs");
const file = "docs/contracts/pollen-cloud-dek-api.openapi.yaml";
const newEndpoints = `
  /v1/tenants/local/scan:
    post:
      summary: Start Local Scan Session
      responses:
        "200":
          description: OK
  /v1/tenants/local/scans/{scan_id}:
    get:
      summary: Get Local Scan Session by ID
      parameters:
        - in: path
          name: scan_id
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
  /v1/tenants/local/scans/{scan_id}/events:
    get:
      summary: Stream Local Scan Session Events (SSE)
      parameters:
        - in: path
          name: scan_id
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
  /v1/tenants/local/capability-snapshot:
    get:
      summary: Get latest local capability snapshot
      responses:
        "200":
          description: OK
  /v1/tenants/local/policy-suggestions:
    get:
      summary: List suggested policies based on capabilities and discovered agents
      responses:
        "200":
          description: OK
  /v1/tenants/local/policies/feasibility:
    post:
      summary: Check policy feasibility
      responses:
        "200":
          description: OK
  /v1/tenants/local/deployment-sessions:
    post:
      summary: Create a deployment session
      responses:
        "200":
          description: OK
  /v1/tenants/local/deployment-sessions/{id}:
    get:
      summary: Get deployment session
      parameters:
        - in: path
          name: id
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
  /v1/tenants/local/deployment-sessions/{id}/events:
    get:
      summary: Stream deployment session events (SSE)
      parameters:
        - in: path
          name: id
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
  /v1/tenants/local/deployment-sessions/{id}/actions/{action_id}/approve:
    post:
      summary: Approve an action in a deployment session
      parameters:
        - in: path
          name: id
          required: true
          schema:
            type: string
        - in: path
          name: action_id
          required: true
          schema:
            type: string
      responses:
        "200":
          description: OK
  /v1/tenants/local/health/local-enforcement:
    get:
      summary: Get health of local enforcement layer
      responses:
        "200":
          description: OK
`;
fs.appendFileSync(file, newEndpoints);
console.log("Appended to " + file);

