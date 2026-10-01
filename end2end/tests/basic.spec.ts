import { test, expect } from "@playwright/test";

process.loadEnvFile("../.env");

test("homepage has title and heading text", async ({ page }) => {
  await page.goto("http://localhost:3000/");
  await expect(page.locator("span")).toHaveText("Vault Docs");
});

test("user can login through cognito", async ({ page }) => {
  let email = process.env.TEST_USERNAME;
  let password = process.env.TEST_PASSWORD;

  if (email === undefined)
    throw ("email not set");
  if (password === undefined)
    throw ("password not set");

  await page.goto("http://localhost:3000/");
  await page.getByRole('link', { name: 'Login' }).click();
  await page.waitForURL(/amazoncognito\.com/);
  await page.locator('input[name="username"]').fill(email);
  await page.getByRole("button", { name: "Next" }).click();
  await page.locator('input[name="password"]').fill(password);
  await page.getByRole("button", { name: "Continue" }).click();
  await page.waitForURL(/localhost:8080/);
  await expect(page.getByLabel("Profile")).toBeVisible();
});
