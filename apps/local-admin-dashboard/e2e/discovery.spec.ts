import { test, expect } from '@playwright/test';

test('load contract discovery in settings', async ({ page }) => {
  await page.goto('http://127.0.0.1:3000');
  
  // Navigate to Settings
  await page.getByRole('link', { name: /settings/i }).click();

  // Wait for Contract Discovery to load and verify "Preferred Contract" exists
  await expect(page.getByText('Contract Discovery')).toBeVisible();
  
  // Check that the schema version appears
  await expect(page.getByText('contract-discovery.v1')).toBeVisible({ timeout: 10000 });
});
