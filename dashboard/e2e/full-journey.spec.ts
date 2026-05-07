import { expect, test } from "@playwright/test";
import { installMerchantApi, merchant } from "./merchant-fixtures";

test("full journey signup -> login -> onboarding -> payment -> settlement view", async ({ page }) => {
  await installMerchantApi(page);

  // 1. Signup
  await page.goto("/signup");
  await page.getByPlaceholder("John Doe").fill("Journey User");
  await page.getByPlaceholder("merchant@business.com").fill(merchant.email);
  await page.getByPlaceholder("My Awesome Shop").fill("Journey Shop");
  await page.getByPlaceholder("••••••••").fill("StrongPass123!");
  await page.getByRole("button", { name: /get started/i }).click();

  // 2. Login
  await page.goto("/login");
  await page.getByLabel("Email").fill(merchant.email);
  await page.getByLabel("Password").fill("StrongPass123!");
  await page.getByRole("button", { name: /sign in/i }).click();

  // Dashboard landing
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole("heading", { name: /dashboard/i })).toBeVisible();

  // 3. Onboarding
  await page.goto("/onboarding");
  await page.getByPlaceholder("Your company name").fill("Journey Shop");
  await page.getByRole("button", { name: /continue/i }).click();
  await page.getByPlaceholder("AAAPL1234C").fill("ABCDE1234F");
  await page.getByRole("button", { name: /continue/i }).click();
  await page.getByPlaceholder("HDFC Bank").fill("HDFC Bank");
  await page.getByPlaceholder("1234567890").fill("123456789012");
  await page.getByPlaceholder("HDFC0001234").fill("HDFC0001234");
  await page.getByRole("button", { name: /continue/i }).click();
  await page.getByRole("button", { name: /submit application/i }).click();
  await expect(page.getByText(/onboarding submitted and verified/i)).toBeVisible();

  // 4. Payments
  await page.getByRole("link", { name: /payments/i }).first().click();
  await expect(page.getByRole("heading", { name: /payments/i })).toBeVisible();
  await expect(page.getByText(/pay_seed_success/i)).toBeVisible();

  // 5. Settlements
  await page.getByRole("link", { name: /settlements/i }).first().click();
  await expect(page.getByText(/available balance/i)).toBeVisible();
  await expect(page.getByText(/utrtest123/i)).toBeVisible();
});
