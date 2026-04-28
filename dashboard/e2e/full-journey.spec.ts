import { expect, test } from "@playwright/test";

test("full journey signup -> login -> onboarding -> payment -> settlement view", async ({ page }) => {
  await page.route("**/v1/merchants/register", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ ok: true }) });
  });
  await page.route("**/v1/merchants/login", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        token: "fake-token",
        merchant_id: "00000000-0000-0000-0000-000000000001",
        name: "Demo Merchant",
        api_key: "fp_test_demo",
      }),
    });
  });
  await page.route("**/v1/onboarding/kyc", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ status: "verified" }) });
  });
  await page.route("**/v1/onboarding/bank-account", async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ status: "submitted" }) });
  });
  await page.route("**/v1/settlements", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        settlements: [
          {
            id: "set_1",
            settlement_date: "2026-04-28",
            status: "completed",
            amount: 10000,
            fee_amount: 100,
            tax_amount: 18,
            net_amount: 9882,
            utr: "UTR123",
          },
        ],
      }),
    });
  });
  await page.route("**/v1/settlements/balance", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ available: 9882, pending: 0 }),
    });
  });

  await page.goto("/signup");
  await page.getByPlaceholder("John Doe").fill("Journey User");
  await page.getByPlaceholder("merchant@business.com").fill("journey@example.com");
  await page.getByPlaceholder("My Awesome Shop").fill("Journey Shop");
  await page.getByPlaceholder("••••••••").fill("StrongPass123!");
  await page.getByRole("button", { name: /get started/i }).click();

  await page.goto("/login");
  await page.getByPlaceholder("merchant@business.com").fill("journey@example.com");
  await page.getByPlaceholder("••••••••").fill("StrongPass123!");
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page).toHaveURL(/\/$/);

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

  await page.goto("/payments");
  await expect(page.getByText(/payments/i)).toBeVisible();
  await expect(page.getByText(/fp_8a3b2c1d/i)).toBeVisible();

  await page.goto("/settlements");
  await expect(page.getByText(/available balance/i)).toBeVisible();
  await expect(page.getByText(/utr123/i)).toBeVisible();
});

