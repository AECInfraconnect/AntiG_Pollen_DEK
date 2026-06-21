import { test, expect } from '@playwright/test';
import { installMockApi } from './mock-api';

test.describe('Security & Policies', () => {
  test.beforeEach(async ({ page }) => {
    await installMockApi(page);
    await page.goto('/');
  });

  test('create and publish policy from dashboard', async ({ page }) => {
    await page.getByRole('link', { name: /active policies/i }).click();
    await page.getByRole('button', { name: /new policy/i }).click();
    
    await page.getByLabel(/name/i).fill('E2E Deny Critical');
    await page.getByLabel(/engine/i).selectOption('cedar');
    await page.getByLabel(/source/i).fill('forbid(principal, action, resource) when { context.risk_level == "critical" };');
    await page.getByRole('button', { name: /^save( draft)?$/i }).click();

    await expect(page.getByText('E2E Deny Critical')).toBeVisible();

    const row = page.locator('tr').filter({ hasText: 'E2E Deny Critical' }).first();
    await row.getByRole('button', { name: /^publish$/i }).click();

    await expect(page.getByText(/^Published .* bundle /i)).toBeVisible({ timeout: 10000 });
  });

  test('view policy presets', async ({ page }) => {
    await page.getByRole('link', { name: /policy presets/i }).click();
    await expect(page.getByRole('heading', { name: /policy presets/i, exact: false }).first()).toBeVisible();

  });

  test('view policy suggestions', async ({ page }) => {
    await page.getByRole('link', { name: /policy suggestions/i }).click();
    await expect(page.getByRole('heading', { name: /suggestions/i, exact: false }).first()).toBeVisible();
  });
});
