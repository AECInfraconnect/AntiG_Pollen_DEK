import { test, expect } from "@playwright/test";
import { installMockApi } from "./mock-api";

test.describe("Policy-First Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto("/");
  });

  test("should render sidebar and navigate to simple sections", async ({ page }) => {
    // 1. Dashboard Overview
    await expect(page.getByRole("heading", { name: "Dashboard Overview" })).toBeVisible();

    // 2. Protect
    await page.getByRole("link", { name: /(protect|สแกน)/i }).click();
    await expect(page.getByRole("heading", { name: /(protect|สแกน)/i, exact: false }).first()).toBeVisible();

    // 3. Activity
    await page.getByRole("link", { name: /(activity|กิจกรรม)/i }).click();
    await expect(page.getByRole("heading", { name: /(activity|กิจกรรม)/i, exact: false }).first()).toBeVisible();

    // 4. Alerts
    await page.getByRole("link", { name: /(alerts|แจ้งเตือน)/i }).click();
    await expect(page.getByRole("heading", { name: /(alerts|แจ้งเตือน)/i, exact: false }).first()).toBeVisible();
  });
});
