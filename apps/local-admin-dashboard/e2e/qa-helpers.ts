import { expect, type Page } from "@playwright/test";
import type { QaMode, QaRoute } from "./qa-routes";

export async function installQaMode(page: Page, mode: QaMode) {
  await page.addInitScript((nextMode) => {
    localStorage.setItem("pollek.mode", nextMode);
    localStorage.setItem("i18nextLng", "en");
    localStorage.setItem("pollek.sidebar.collapsed", "false");
  }, mode);
}

export async function enableEnterpriseCloudMock(page: Page) {
  await page.route("**/v1/tenants/local/pdp/cloud**", (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        status: "connected",
        manual_override_enabled: false,
        health: {
          state: "ok",
          detail: "Enterprise Cloud QA fixture is connected.",
        },
      }),
    }),
  );
}

export async function seedObservedData(page: Page, route: QaRoute) {
  if (!route.seedScan) return;
  await page.goto("/scan");
  await page.getByRole("button", { name: /^Deep Scan$/ }).first().click();
  await expect(page.getByText("Antigravity").first()).toBeVisible({
    timeout: 20_000,
  });
}

export function screenshotName(mode: QaMode, route: QaRoute, theme: string) {
  const cleanPath =
    route.path === "/"
      ? "overview"
      : route.path.replace(/^\//, "").replace(/[^a-z0-9-]+/gi, "-");
  return `${mode}-${theme}-${cleanPath}.png`;
}
