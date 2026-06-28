import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { installMockApi } from "./mock-api";
import { qaModes, qaRoutes } from "./qa-routes";
import {
  enableEnterpriseCloudMock,
  installQaMode,
  screenshotName,
  seedObservedData,
} from "./qa-helpers";

const entityPages = [
  {
    path: "/agents",
    masterHeading: "Agents & Models",
    detailText: "Detail Workspace",
  },
  {
    path: "/activity-timeline",
    masterHeading: "Activity Timeline",
    detailText: "Create rule from event",
  },
  {
    path: "/deployments",
    masterHeading: "Deployments",
    detailText: "Rollback",
  },
] as const;

function installPageErrorGuard(page: Page) {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => {
    pageErrors.push(`${page.url()}: ${error.message}`);
  });
  page.on("console", (message) => {
    if (message.type() === "error" && /Objects are not valid as a React child/.test(message.text())) {
      pageErrors.push(`${page.url()}: ${message.text()}`);
    }
  });
  return pageErrors;
}

async function expectNoPageErrors(pageErrors: string[]) {
  await expect.poll(() => pageErrors, { timeout: 250 }).toEqual([]);
}

async function seedObservedAgent(page: Page) {
  await page.goto("/scan");
  await page.getByRole("button", { name: /^Deep Scan$/ }).first().click();
  await expect(page.getByText("Antigravity").first()).toBeVisible({
    timeout: 20_000,
  });
}

async function switchToReadableLightMode(page: Page) {
  const themeButton = page.getByRole("button", { name: /Change theme/i });
  for (let i = 0; i < 3; i += 1) {
    if (!(await page.locator("html").evaluate((root) => root.classList.contains("dark")))) {
      return;
    }
    await themeButton.click();
  }
  await expect(page.locator("html")).not.toHaveClass(/dark/);
}

test.describe("Visual smoke for entity pages", () => {
  test.setTimeout(180_000);

  test.skip(
    process.env.DEK_PLAYWRIGHT_EXTERNAL_SERVER === "1",
    "Uses mock API fixtures to validate dashboard layout states.",
  );

  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.addInitScript(() => {
      localStorage.setItem("pollek.mode", "desktop_advanced");
      localStorage.setItem("pollek.theme", "dark");
    });
  });

  for (const pageCase of entityPages) {
    test(`${pageCase.masterHeading} has usable master/detail layout in desktop and mobile`, async ({
      page,
    }) => {
      const pageErrors = installPageErrorGuard(page);
      await page.setViewportSize({ width: 1440, height: 900 });
      await seedObservedAgent(page);
      await page.goto(pageCase.path);
      await expect(
        page.getByRole("heading", { name: pageCase.masterHeading }),
      ).toBeVisible();

      const masterList = page.getByRole("listbox", { name: "Items" });
      const firstCard = masterList.getByRole("option").first();
      await expect(firstCard).toBeVisible();
      await expect(firstCard).toHaveCSS("cursor", "pointer");

      await firstCard.click();
      await expect(page.getByText(pageCase.detailText).first()).toBeVisible();
      await expect(page.getByText("<!doctype html")).toHaveCount(0);
      await expectNoPageErrors(pageErrors);

      await page.setViewportSize({ width: 390, height: 844 });
      await page.goto(pageCase.path);
      await expect(
        page.getByRole("heading", { name: pageCase.masterHeading }),
      ).toBeVisible();
      await expect(
        page
          .getByRole("listbox", { name: "Items" })
          .getByRole("option")
          .first(),
      ).toBeVisible();
      await expect(page.getByText("<!doctype html")).toHaveCount(0);
      await expectNoPageErrors(pageErrors);
    });
  }

  test("light mode keeps core dashboard text readable", async ({ page }) => {
    const pageErrors = installPageErrorGuard(page);
    await page.setViewportSize({ width: 1280, height: 800 });
    await seedObservedAgent(page);
    await page.goto("/agents");
    await switchToReadableLightMode(page);
    await expect(
      page.getByRole("heading", { name: "Agents & Models" }),
    ).toBeVisible();
    await expect(
      page
        .getByRole("listbox", { name: "Items" })
        .getByRole("option", { name: /Antigravity/ }),
    ).toBeVisible();
    await expect(page.getByText("<!doctype html")).toHaveCount(0);
    await expectNoPageErrors(pageErrors);
  });

  for (const mode of qaModes) {
    test(`${mode} captures visual QA screenshots for reachable pages`, async ({
      page,
    }, testInfo) => {
      const pageErrors = installPageErrorGuard(page);
      await page.setViewportSize({ width: 1440, height: 900 });
      await installMockApi(page);
      if (mode === "enterprise_cloud") {
        await enableEnterpriseCloudMock(page);
      }
      await installQaMode(page, mode);

      const routes = qaRoutes.filter((route) => route.modes.includes(mode));

      for (const route of routes) {
        await seedObservedData(page, route);
        await page.goto(route.path);
        await expect(page.locator("body")).toBeVisible();
        await expect(page.getByText("<!doctype html")).toHaveCount(0);
        await expectNoPageErrors(pageErrors);
        await testInfo.attach(screenshotName(mode, route, "dark"), {
          body: await page.screenshot({ fullPage: true }),
          contentType: "image/png",
        });
      }

      const lightRoute = mode === "desktop_simple" ? "/activity" : "/agents";
      await page.goto(lightRoute);
      await switchToReadableLightMode(page);
      await expectNoPageErrors(pageErrors);
      await testInfo.attach(`${mode}-light.png`, {
        body: await page.screenshot({ fullPage: true }),
        contentType: "image/png",
      });

      await page.setViewportSize({ width: 390, height: 844 });
      await page.goto("/activity");
      await expect(page.getByRole("heading", { name: "AI Activity" })).toBeVisible();
      await expectNoPageErrors(pageErrors);
      await testInfo.attach(`${mode}-mobile-activity.png`, {
        body: await page.screenshot({ fullPage: true }),
        contentType: "image/png",
      });
    });
  }
});
