import { test, expect } from '@playwright/test';
import { installMockApi } from './mock-api';

test.describe('System Settings', () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto('/');
  });

  test('view identities', async ({ page }) => {
    await page.getByRole('link', { name: /^identities$/i }).click();
    await expect(page.getByRole('heading', { name: /identity/i, exact: false }).first()).toBeVisible();
  });

  test('view data resources', async ({ page }) => {
    await page.getByRole('link', { name: /data resources/i }).click();
    await expect(page.getByRole('heading', { name: /resources/i, exact: false }).first()).toBeVisible();
  });

  test('view simulator', async ({ page }) => {
    await page.getByRole('link', { name: /simulator/i }).click();
    await expect(page.getByRole('heading', { name: /simulator/i, exact: false }).first()).toBeVisible();
  });

  test('view bundles and sync', async ({ page }) => {
    await page.getByRole('link', { name: /bundles & sync/i }).click();
    await expect(page.getByRole('heading', { name: /bundles/i, exact: false }).first()).toBeVisible();
  });

  test('view auto discovery', async ({ page }) => {
    await page.getByRole('link', { name: /auto discovery/i }).click();
    await expect(page.getByRole('heading', { name: /auto discovery/i, exact: false }).first()).toBeVisible();
  });

  test('view global settings', async ({ page }) => {
    await page.getByRole('link', { name: /global settings/i }).click();
    await expect(page.getByRole('heading', { name: /settings/i, exact: false }).first()).toBeVisible();
  });
});
