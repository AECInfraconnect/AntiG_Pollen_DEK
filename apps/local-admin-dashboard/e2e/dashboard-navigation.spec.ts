import { test, expect } from '@playwright/test';
import { installMockApi } from './mock-api';

test.describe('Dashboard Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto('/');
  });

  test('should render sidebar and navigate to all main sections', async ({ page }) => {
    // 1. Dashboard
    await expect(page.getByRole('heading', { name: 'Dashboard Overview' })).toBeVisible();

    // 2. AI Ecosystem
    await page.getByRole('link', { name: /agents & models/i }).click();
    await expect(page.getByRole('heading', { name: /agents/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /integrations/i }).click();
    await expect(page.getByRole('heading', { name: /integrations/i, exact: false }).first()).toBeVisible();

    // 3. Security & Policies
    await page.getByRole('link', { name: /policy presets/i }).click();
    await expect(page.getByRole('heading', { name: /policy presets/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /policy suggestions/i }).click();
    await expect(page.getByRole('heading', { name: /suggestions/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /active policies/i }).click();
    await expect(page.getByRole('heading', { name: /active policies|policy enforcer/i, exact: false }).first()).toBeVisible();

    // 4. Monitoring & Audit
    await page.getByRole('link', { name: /alerts & shadow ai/i }).click();
    await expect(page.getByRole('heading', { name: /alerts/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /audit logs/i }).click();
    await expect(page.getByRole('heading', { name: /decision logs/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /cost & tokens/i }).click();
    await expect(page.getByRole('heading', { name: /cost ledger/i, exact: false }).first()).toBeVisible();

    // 5. System Settings
    await page.getByRole('link', { name: /^identities$/i }).click();
    await expect(page.getByRole('heading', { name: /identity & network/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /data resources/i }).click();
    await expect(page.getByRole('heading', { name: /resources/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /simulator/i }).click();
    await expect(page.getByRole('heading', { name: /simulator/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /bundles & sync/i }).click();
    await expect(page.getByRole('heading', { name: /bundles/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /auto discovery/i }).click();
    await expect(page.getByRole('heading', { name: /auto discovery/i, exact: false }).first()).toBeVisible();

    await page.getByRole('link', { name: /global settings/i }).click();
    await expect(page.getByRole('heading', { name: /settings/i, exact: false }).first()).toBeVisible();
  });
});
