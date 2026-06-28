import { expect, test } from "@playwright/test";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { installMockApi } from "./mock-api";
import { qaModes, qaRoutes } from "./qa-routes";
import {
  enableEnterpriseCloudMock,
  installQaMode,
  seedObservedData,
} from "./qa-helpers";

const require = createRequire(import.meta.url);
const axeSource = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");
const blockingImpacts = new Set(["critical"]);

test.describe("WCAG accessibility gate", () => {
  test.setTimeout(120_000);

  test.skip(
    process.env.DEK_PLAYWRIGHT_EXTERNAL_SERVER === "1",
    "Mock accessibility gate runs in dashboard-ci; real Local Control Plane E2E keeps its integration focus.",
  );

  for (const mode of qaModes) {
    const routes = qaRoutes.filter((route) => route.modes.includes(mode));

    test(`${mode} pages have no critical axe violations`, async ({
      page,
    }, testInfo) => {
      await installMockApi(page);
      if (mode === "enterprise_cloud") {
        await enableEnterpriseCloudMock(page);
      }
      await installQaMode(page, mode);

      const allBlockingViolations: unknown[] = [];

      for (const route of routes) {
        await seedObservedData(page, route);
        await page.goto(route.path);
        await expect(page.locator("#app")).toBeVisible();
        await page.addScriptTag({ content: axeSource });

        const result = await page.evaluate(async () => {
          const axe = (window as any).axe;
          return axe.run(document, {
            runOnly: {
              type: "tag",
              values: [
                "wcag2a",
                "wcag2aa",
                "wcag21a",
                "wcag21aa",
                "wcag22aa",
              ],
            },
          });
        });

        const blocking = result.violations
          .filter((violation: any) => blockingImpacts.has(violation.impact))
          .map((violation: any) => ({
            route: route.path,
            id: violation.id,
            impact: violation.impact,
            help: violation.help,
            nodes: violation.nodes.slice(0, 3).map((node: any) => ({
              target: node.target,
              failureSummary: node.failureSummary,
            })),
          }));

        allBlockingViolations.push(...blocking);
      }

      await testInfo.attach(`${mode}-axe-results.json`, {
        body: JSON.stringify(allBlockingViolations, null, 2),
        contentType: "application/json",
      });
      expect(allBlockingViolations).toEqual([]);
    });
  }
});
