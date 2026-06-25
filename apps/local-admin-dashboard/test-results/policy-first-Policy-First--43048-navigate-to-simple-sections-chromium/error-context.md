# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: policy-first.spec.ts >> Policy-First Navigation >> should render sidebar and navigate to simple sections
- Location: e2e\policy-first.spec.ts:10:3

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: locator.click: Test timeout of 30000ms exceeded.
Call log:
  - waiting for getByRole('link', { name: /protect/i })

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - region "Notifications alt+T"
  - generic [ref=e3]:
    - complementary "Main navigation" [ref=e4]:
      - generic [ref=e5]:
        - generic [ref=e6]: POLLEK
        - generic [ref=e7]: AI Local Enforcement Kit
      - navigation [ref=e8]
      - generic [ref=e9]:
        - generic [ref=e14] [cursor=pointer]: Local Admin
        - combobox [ref=e15]:
          - option "Simple" [selected]
          - option "Advanced"
          - option "Enterprise"
    - generic [ref=e16]:
      - banner [ref=e17]:
        - generic [ref=e19]:
          - img [ref=e20]
          - searchbox "Search resources, policies, or agents... (⌘K)" [ref=e23]
          - generic:
            - generic: ⌘
            - text: K
        - generic [ref=e24]:
          - combobox [ref=e25]:
            - option "Simple" [selected]
            - option "Advanced"
            - option "Enterprise"
          - button "en" [ref=e26] [cursor=pointer]:
            - img [ref=e27]
            - generic [ref=e31]: en
          - button [ref=e32] [cursor=pointer]:
            - img [ref=e33]
          - button [ref=e39] [cursor=pointer]:
            - img [ref=e40]
      - main [ref=e44]:
        - generic [ref=e45]:
          - generic [ref=e46]:
            - generic [ref=e47]:
              - heading "Dashboard Overview" [level=2] [ref=e48]
              - paragraph [ref=e49]: Real-time metrics and system health for your local Pollek Local Enforcement Kit.
            - generic [ref=e50]:
              - img [ref=e51]
              - generic [ref=e53]: Air-Gap Ready / Sovereign Mode
          - generic [ref=e54]:
            - generic [ref=e55]:
              - generic [ref=e57]:
                - generic [ref=e58]: Active Agents
                - img [ref=e59]
              - generic [ref=e64]:
                - generic [ref=e65]: "0"
                - generic [ref=e66]: Live
            - generic [ref=e67]:
              - generic [ref=e69]:
                - generic [ref=e70]: Connected MCPs
                - img [ref=e71]
              - generic [ref=e74]:
                - generic [ref=e75]: "0"
                - generic [ref=e76]: Live
            - generic [ref=e77]:
              - generic [ref=e79]:
                - generic [ref=e80]: Registered Tools
                - img [ref=e81]
              - generic [ref=e83]:
                - generic [ref=e84]: "0"
                - generic [ref=e85]: Live
            - generic [ref=e86]:
              - generic [ref=e88]:
                - generic [ref=e89]: Known Resources
                - img [ref=e90]
              - generic [ref=e92]:
                - generic [ref=e93]: "0"
                - generic [ref=e94]: Live
          - generic [ref=e95]:
            - heading "What POLLEK can do" [level=3] [ref=e97]:
              - generic [ref=e98]: What POLLEK can do
            - generic [ref=e99]:
              - heading "Recent Audit Activity" [level=3] [ref=e100]
              - paragraph [ref=e102]: No recent activity.
```

# Test source

```ts
  1  | import { test, expect } from "@playwright/test";
  2  | import { installMockApi } from "./mock-api";
  3  | 
  4  | test.describe("Policy-First Navigation", () => {
  5  |   test.beforeEach(async ({ page }) => {
  6  |     await installMockApi(page);
  7  |     await page.goto("/");
  8  |   });
  9  | 
  10 |   test("should render sidebar and navigate to simple sections", async ({ page }) => {
  11 |     // 1. Dashboard Overview
  12 |     await expect(page.getByRole("heading", { name: "Dashboard Overview" })).toBeVisible();
  13 | 
  14 |     // 2. Protect
> 15 |     await page.getByRole("link", { name: /protect/i }).click();
     |                                                        ^ Error: locator.click: Test timeout of 30000ms exceeded.
  16 |     await expect(page.getByRole("heading", { name: /protect/i, exact: false }).first()).toBeVisible();
  17 | 
  18 |     // 3. Activity
  19 |     await page.getByRole("link", { name: /activity/i }).click();
  20 |     await expect(page).toHaveURL(/.*audit/);
  21 |   });
  22 | });
  23 | 
```