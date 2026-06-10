import { test, expect } from '@playwright/test';

test('register agent and publish policy from dashboard', async ({ page }) => {
  await page.goto('http://127.0.0.1:3000');

  await page.getByRole('link', { name: /agents/i }).click();
  await page.getByRole('button', { name: /add agent/i }).click();
  await page.getByLabel(/name/i).fill('E2E Dashboard Agent');
  await page.getByLabel(/agent id/i).fill('agent-dashboard-e2e');
  await page.getByRole('button', { name: /save/i }).click();
  await expect(page.getByText('E2E Dashboard Agent')).toBeVisible();

  await page.getByRole('link', { name: /policies/i }).click();
  await page.getByRole('button', { name: /new policy/i }).click();
  await page.getByLabel(/name/i).fill('E2E Deny Critical');
  await page.getByLabel(/type/i).selectOption('cedar');
  await page.getByLabel(/source/i).fill('forbid(principal, action, resource) when { context.risk_level == "critical" };');
  await page.getByRole('button', { name: /publish/i }).click();

  await expect(page.getByText(/published/i)).toBeVisible();
});
