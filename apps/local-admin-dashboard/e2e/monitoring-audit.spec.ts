import { test, expect } from '@playwright/test';
import { installMockApi } from './mock-api';

test.describe('Monitoring & Audit', () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto('/');
  });

  test('view alerts and shadow AI', async ({ page }) => {
    await page.getByRole('link', { name: /alerts & shadow ai/i }).click();
    await expect(page.getByRole('heading', { name: /alerts/i, exact: false }).first()).toBeVisible();
  });

  test('view audit logs', async ({ page }) => {
    await page.getByRole('link', { name: /audit logs/i }).click();
    await expect(page.getByRole('heading', { name: /decision logs/i, exact: false }).first()).toBeVisible();
  });

  test('view cost ledger', async ({ page }) => {
    await page.getByRole('link', { name: /cost & tokens/i }).click();
    await expect(page.getByRole('heading', { name: /cost ledger/i, exact: false }).first()).toBeVisible();
  });
});
