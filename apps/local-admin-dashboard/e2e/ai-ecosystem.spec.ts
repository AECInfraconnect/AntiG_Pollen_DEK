import { test, expect } from "@playwright/test";
import { installMockApi } from "./mock-api";

test.describe("AI Ecosystem", () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto("/");
  });

  test("view agents and models", async ({ page }) => {
    await page.getByRole("link", { name: /agents & models/i }).click();
    await expect(
      page.getByRole("heading", { name: /agents/i, exact: false }).first(),
    ).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
  });

  test("view integrations", async ({ page }) => {
    await page.getByRole("link", { name: /integrations/i }).click();
    await expect(
      page
        .getByRole("heading", { name: /integrations/i, exact: false })
        .first(),
    ).toBeVisible();
  });
});
