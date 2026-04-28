import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("login page renders on all viewport projects", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();
});

test("a11y baseline on login page", async ({ page }) => {
  await page.goto("/login");
  const accessibilityScanResults = await new AxeBuilder({ page }).analyze();
  expect(accessibilityScanResults.violations).toEqual([]);
});

