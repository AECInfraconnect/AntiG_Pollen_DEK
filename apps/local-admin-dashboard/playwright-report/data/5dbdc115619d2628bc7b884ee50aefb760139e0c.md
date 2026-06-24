# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: security-policies.spec.ts >> Security & Policies >> create and publish policy from dashboard
- Location: e2e\security-policies.spec.ts:10:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText('E2E Deny Critical')
Expected: visible
Error: strict mode violation: getByText('E2E Deny Critical') resolved to 2 elements:
    1) <div class="font-medium">E2E Deny Critical</div> aka getByText('E2E Deny Critical').first()
    2) <div class="font-medium">E2E Deny Critical</div> aka getByText('E2E Deny Critical').nth(1)

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText('E2E Deny Critical')

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - generic [ref=e4]:
    - generic [ref=e5]:
      - heading "Pollek Local Enforcement Kit" [level=1] [ref=e6]:
        - img [ref=e7]
        - text: Pollek Local Enforcement Kit
      - generic [ref=e10]: v2
    - navigation [ref=e12]:
      - link "Overview" [ref=e15] [cursor=pointer]:
        - /url: /
        - img [ref=e16]
        - text: Overview
      - generic [ref=e21]:
        - heading "AI Ecosystem" [level=3] [ref=e22]
        - generic [ref=e23]:
          - link "Agents & Models" [ref=e24] [cursor=pointer]:
            - /url: /agents
            - img [ref=e25]
            - text: Agents & Models
          - link "Integrations" [ref=e30] [cursor=pointer]:
            - /url: /integrations
            - img [ref=e31]
            - text: Integrations
          - link "Plugin Marketplace" [ref=e33] [cursor=pointer]:
            - /url: /plugin-marketplace
            - img [ref=e34]
            - text: Plugin Marketplace
      - generic [ref=e36]:
        - heading "Security & Policies" [level=3] [ref=e37]
        - generic [ref=e38]:
          - link "Policy Presets" [ref=e39] [cursor=pointer]:
            - /url: /policy-presets
            - img [ref=e40]
            - text: Policy Presets
          - link "Policy Suggestions" [ref=e43] [cursor=pointer]:
            - /url: /policy-suggestions
            - img [ref=e44]
            - text: Policy Suggestions
          - link "Active Policies" [ref=e46] [cursor=pointer]:
            - /url: /policies
            - img [ref=e47]
            - text: Active Policies
      - generic [ref=e51]:
        - heading "Monitoring & Audit" [level=3] [ref=e52]
        - generic [ref=e53]:
          - link "Alerts & Shadow AI" [ref=e54] [cursor=pointer]:
            - /url: /alerts
            - img [ref=e55]
            - text: Alerts & Shadow AI
          - link "Audit Logs" [ref=e57] [cursor=pointer]:
            - /url: /audit
            - img [ref=e58]
            - text: Audit Logs
          - link "Cost & Tokens" [ref=e60] [cursor=pointer]:
            - /url: /cost-ledger
            - img [ref=e61]
            - text: Cost & Tokens
      - generic [ref=e63]:
        - heading "System Settings" [level=3] [ref=e64]
        - generic [ref=e65]:
          - link "Identities" [ref=e66] [cursor=pointer]:
            - /url: /identities
            - img [ref=e67]
            - text: Identities
          - link "Entities" [ref=e72] [cursor=pointer]:
            - /url: /entities
            - img [ref=e73]
            - text: Entities
          - link "Data Resources" [ref=e78] [cursor=pointer]:
            - /url: /resources
            - img [ref=e79]
            - text: Data Resources
          - link "Simulator" [ref=e83] [cursor=pointer]:
            - /url: /simulator
            - img [ref=e84]
            - text: Simulator
          - link "Bundles & Sync" [ref=e86] [cursor=pointer]:
            - /url: /bundles
            - img [ref=e87]
            - text: Bundles & Sync
          - link "Auto Discovery" [ref=e90] [cursor=pointer]:
            - /url: /discovery
            - img [ref=e91]
            - text: Auto Discovery
          - link "Global Settings" [ref=e94] [cursor=pointer]:
            - /url: /settings
            - img [ref=e95]
            - text: Global Settings
    - generic [ref=e99] [cursor=pointer]:
      - img [ref=e101]
      - generic [ref=e105]:
        - generic [ref=e106]: Local Admin
        - generic [ref=e107]: "tenant: local"
  - generic [ref=e108]:
    - banner [ref=e109]:
      - generic [ref=e111]:
        - img [ref=e112]
        - searchbox "Search resources, policies, or agents..." [ref=e115]
      - generic [ref=e116]:
        - button "en" [ref=e117] [cursor=pointer]:
          - img [ref=e118]
          - generic [ref=e122]: en
        - button [ref=e123] [cursor=pointer]:
          - img [ref=e124]
        - button [ref=e130] [cursor=pointer]:
          - img [ref=e131]
    - main [ref=e135]:
      - generic [ref=e136]:
        - generic [ref=e137]:
          - generic [ref=e138]:
            - heading "Policy Enforcer" [level=2] [ref=e139]:
              - img [ref=e140]
              - text: Policy Enforcer
            - paragraph [ref=e144]: Author, compile, and publish signed policy bundles to the local workspace.
          - button "New Policy" [ref=e145] [cursor=pointer]:
            - img [ref=e146]
            - text: New Policy
        - table [ref=e148]:
          - rowgroup [ref=e149]:
            - row "Name Type Status Targets Actions" [ref=e150]:
              - columnheader "Name" [ref=e151]
              - columnheader "Type" [ref=e152]
              - columnheader "Status" [ref=e153]
              - columnheader "Targets" [ref=e154]
              - columnheader "Actions" [ref=e155]
          - rowgroup [ref=e156]:
            - row "E2E Deny Critical pol-1782268771765 cedar draft 0 target(s) Publish" [ref=e157]:
              - cell "E2E Deny Critical pol-1782268771765" [ref=e158]:
                - generic [ref=e159]: E2E Deny Critical
                - generic [ref=e160]: pol-1782268771765
              - cell "cedar" [ref=e161]
              - cell "draft" [ref=e162]
              - cell "0 target(s)" [ref=e163]
              - cell "Publish" [ref=e164]:
                - generic [ref=e165]:
                  - button "View Policy" [ref=e166] [cursor=pointer]:
                    - img [ref=e167]
                  - button "Edit Policy" [ref=e170] [cursor=pointer]:
                    - img [ref=e171]
                  - button "Delete Policy" [ref=e174] [cursor=pointer]:
                    - img [ref=e175]
                  - button "Publish" [ref=e178] [cursor=pointer]:
                    - img [ref=e179]
                    - text: Publish
            - row "E2E Deny Critical pol-1782268775600 cedar draft 0 target(s) Publish" [ref=e182]:
              - cell "E2E Deny Critical pol-1782268775600" [ref=e183]:
                - generic [ref=e184]: E2E Deny Critical
                - generic [ref=e185]: pol-1782268775600
              - cell "cedar" [ref=e186]
              - cell "draft" [ref=e187]
              - cell "0 target(s)" [ref=e188]
              - cell "Publish" [ref=e189]:
                - generic [ref=e190]:
                  - button "View Policy" [ref=e191] [cursor=pointer]:
                    - img [ref=e192]
                  - button "Edit Policy" [ref=e195] [cursor=pointer]:
                    - img [ref=e196]
                  - button "Delete Policy" [ref=e199] [cursor=pointer]:
                    - img [ref=e200]
                  - button "Publish" [ref=e203] [cursor=pointer]:
                    - img [ref=e204]
                    - text: Publish
            - row "E2E Deny Critical pol-1782267922939 cedar published 0 target(s) Publish" [ref=e207]:
              - cell "E2E Deny Critical pol-1782267922939" [ref=e208]:
                - generic [ref=e209]: E2E Deny Critical
                - generic [ref=e210]: pol-1782267922939
              - cell "cedar" [ref=e211]
              - cell "published" [ref=e212]
              - cell "0 target(s)" [ref=e213]
              - cell "Publish" [ref=e214]:
                - generic [ref=e215]:
                  - button "View Policy" [ref=e216] [cursor=pointer]:
                    - img [ref=e217]
                  - button "Edit Policy" [ref=e220] [cursor=pointer]:
                    - img [ref=e221]
                  - button "Delete Policy" [ref=e224] [cursor=pointer]:
                    - img [ref=e225]
                  - button "Publish" [ref=e228] [cursor=pointer]:
                    - img [ref=e229]
                    - text: Publish
```

# Test source

```ts
  1  | import { test, expect } from "@playwright/test";
  2  | import { installMockApi } from "./mock-api";
  3  | 
  4  | test.describe("Security & Policies", () => {
  5  |   test.beforeEach(async ({ page }) => {
  6  |     await installMockApi(page);
  7  |     await page.goto("/");
  8  |   });
  9  | 
  10 |   test("create and publish policy from dashboard", async ({ page }) => {
  11 |     await page.getByRole("link", { name: /active policies/i }).click();
  12 |     await page.getByRole("button", { name: /new policy/i }).click();
  13 | 
  14 |     await page.getByLabel(/name/i).fill("E2E Deny Critical");
  15 |     await page.getByLabel(/engine/i).selectOption("cedar");
  16 |     await page
  17 |       .getByLabel(/source/i)
  18 |       .fill(
  19 |         'forbid(principal, action, resource) when { context.risk_level == "critical" };',
  20 |       );
  21 |     await page.getByRole("button", { name: /^save( draft)?$/i }).click();
  22 | 
> 23 |     await expect(page.getByText("E2E Deny Critical")).toBeVisible();
     |                                                       ^ Error: expect(locator).toBeVisible() failed
  24 | 
  25 |     const row = page
  26 |       .locator("tr")
  27 |       .filter({ hasText: "E2E Deny Critical" })
  28 |       .first();
  29 |     await row.getByRole("button", { name: /^publish$/i }).click();
  30 | 
  31 |     await expect(page.getByText(/^Published .* bundle /i)).toBeVisible({
  32 |       timeout: 10000,
  33 |     });
  34 |   });
  35 | 
  36 |   test("view policy presets", async ({ page }) => {
  37 |     await page.getByRole("link", { name: /policy presets/i }).click();
  38 |     await expect(
  39 |       page
  40 |         .getByRole("heading", { name: /policy presets/i, exact: false })
  41 |         .first(),
  42 |     ).toBeVisible();
  43 |   });
  44 | 
  45 |   test("view policy suggestions", async ({ page }) => {
  46 |     await page.getByRole("link", { name: /policy suggestions/i }).click();
  47 |     await expect(
  48 |       page.getByRole("heading", { name: /suggestions/i, exact: false }).first(),
  49 |     ).toBeVisible();
  50 |   });
  51 | });
  52 | 
```