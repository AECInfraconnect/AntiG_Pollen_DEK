import { expect, test } from "@playwright/test";
import { installMockApi } from "./mock-api";

test.describe("Prompt Guard activity visibility", () => {
  test.skip(
    process.env.DEK_PLAYWRIGHT_EXTERNAL_SERVER === "1",
    "Runs in the dashboard mock E2E job; the external-server job validates the real Local Control Plane.",
  );

  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.addInitScript(() => {
      localStorage.setItem("pollek.mode", "desktop_simple");
    });
  });

  test("shows guard incidents in normal-user activity and the guard detail view", async ({
    page,
  }) => {
    await page.goto("/scan");
    await page
      .getByRole("button", { name: /^Deep Scan$/ })
      .first()
      .click();
    await expect(page.getByText("Antigravity").first()).toBeVisible({
      timeout: 20_000,
    });

    await page.goto("/activity?category=safety");
    await expect(
      page.getByRole("heading", { name: "AI Activity" }),
    ).toBeVisible();
    await expect(page.getByText("Activity data source")).toBeVisible();
    await expect(page.getByText("Local history")).toBeVisible();
    await expect(
      page.getByText("Antigravity protected Prompt injection attempt").first(),
    ).toBeVisible();
    await expect(
      page.getByText("Prompt Guard and private data safety"),
    ).toBeVisible();

    await page.goto("/alerts?tab=guard");
    await expect(page.getByText("Incident timeline")).toBeVisible();
    await expect(
      page.getByText("Pollek protected prompt injection attempt").first(),
    ).toBeVisible();
    await expect(page.getByText("API key or secret")).toBeVisible();
    await expect(page.getByText("llm01_prompt_injection")).toHaveCount(0);
    await expect(page.getByText("Request approval")).toHaveCount(0);
  });
});
