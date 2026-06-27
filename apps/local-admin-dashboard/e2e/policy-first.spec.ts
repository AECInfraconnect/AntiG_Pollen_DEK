import { test, expect } from "@playwright/test";
import { installMockApi } from "./mock-api";

test.describe("Policy-First Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.addInitScript(() => {
      localStorage.setItem("pollek.mode", "desktop_advanced");
    });
    await page.goto("/");
  });

  test("should render sidebar and navigate to simple sections", async ({
    page,
  }) => {
    // 1. Dashboard Overview
    await expect(
      page.getByRole("heading", { name: "Dashboard Overview" }),
    ).toBeVisible();

    // 2. Find AI Apps / legacy Scan & Discover
    await page
      .getByRole("link", { name: /(find ai apps|scan & discover)/i })
      .click();
    await expect(
      page.getByRole("heading", { name: "Auto Discovery" }),
    ).toBeVisible();

    // 3. Activity
    await page.getByRole("link", { name: /^AI Activity$/i }).click();
    await expect(
      page
        .getByRole("heading", { name: /(activity|กิจกรรม)/i, exact: false })
        .first(),
    ).toBeVisible();

    // 4. Alerts
    await page.getByRole("link", { name: /alerts & shadow ai/i }).click();
    await expect(
      page.getByRole("heading", { name: "Alerts & Shadow AI" }),
    ).toBeVisible();
  });

  test("relationship and activity pages do not show raw Vite fallback HTML", async ({
    page,
  }) => {
    await page.goto("/entity-graph");
    await expect(
      page.getByRole("heading", { name: "Relationship Map" }),
    ).toBeVisible();
    await expect(page.getByText("<!doctype html")).toHaveCount(0);

    await page.goto("/activity-timeline");
    await expect(
      page.getByRole("heading", { name: "Activity Timeline" }),
    ).toBeVisible();
    await expect(page.getByText("<!doctype html")).toHaveCount(0);
  });

  test("API HTML fallback errors are shortened for operators", async ({
    page,
  }) => {
    await page.route("**/v1/tenants/local/entity-graph**", (route) =>
      route.fulfill({
        status: 200,
        contentType: "text/html",
        body: '<!doctype html><html><head><script type="module" src="/assets/index.js"></script></head></html>',
      }),
    );

    await page.goto("/entity-graph");

    await expect(
      page.getByText(
        "Local Control Plane API returned dashboard HTML instead of JSON",
      ),
    ).toBeVisible();
    await expect(page.getByText('script type="module"')).toHaveCount(0);
  });
});
