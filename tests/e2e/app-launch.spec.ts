import { test, expect } from "@playwright/test";

test.describe("App Launch", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("app loads and shows the main container", async ({ page }) => {
    // The root app container should be visible
    const appContainer = page.locator("[data-testid='app-container']");
    await expect(appContainer).toBeVisible();
  });

  test("nav bar renders with all 3 view buttons", async ({ page }) => {
    const navBar = page.locator("[data-testid='nav-bar']");
    await expect(navBar).toBeVisible();

    const clipboardNav = page.locator("[data-testid='nav-quick-paste']");
    const historyNav = page.locator("[data-testid='nav-history']");
    const settingsNav = page.locator("[data-testid='nav-settings']");

    await expect(clipboardNav).toBeVisible();
    await expect(historyNav).toBeVisible();
    await expect(settingsNav).toBeVisible();

    await expect(clipboardNav).toHaveText(/Clipboard/);
    await expect(historyNav).toHaveText(/History/);
    await expect(settingsNav).toHaveText(/Settings/);
  });

  test("Clipboard view is shown by default", async ({ page }) => {
    const clipboardView = page.locator("[data-testid='view-quick-paste']");
    await expect(clipboardView).toBeVisible();

    // The app title should show
    await expect(page.locator("text=ContextPaste")).toBeVisible();
    await expect(page.locator("text=AI-Powered Smart Clipboard")).toBeVisible();
  });

  test("can navigate to History view", async ({ page }) => {
    await page.locator("[data-testid='nav-history']").click();

    const historyView = page.locator("[data-testid='history-panel']");
    await expect(historyView).toBeVisible();

    // Clipboard view should be hidden
    const clipboardView = page.locator("[data-testid='view-quick-paste']");
    await expect(clipboardView).not.toBeVisible();
  });

  test("can navigate to Settings view", async ({ page }) => {
    await page.locator("[data-testid='nav-settings']").click();

    const settingsView = page.locator("[data-testid='settings-panel']");
    await expect(settingsView).toBeVisible();
  });

  test("can navigate between all views", async ({ page }) => {
    // Start on Clipboard
    await expect(page.locator("[data-testid='view-quick-paste']")).toBeVisible();

    // Go to History
    await page.locator("[data-testid='nav-history']").click();
    await expect(page.locator("[data-testid='history-panel']")).toBeVisible();
    await expect(page.locator("[data-testid='view-quick-paste']")).not.toBeVisible();

    // Go to Settings
    await page.locator("[data-testid='nav-settings']").click();
    await expect(page.locator("[data-testid='settings-panel']")).toBeVisible();
    await expect(page.locator("[data-testid='history-panel']")).not.toBeVisible();

    // Back to Clipboard
    await page.locator("[data-testid='nav-quick-paste']").click();
    await expect(page.locator("[data-testid='view-quick-paste']")).toBeVisible();
    await expect(page.locator("[data-testid='settings-panel']")).not.toBeVisible();
  });

  test("theme toggle button is visible", async ({ page }) => {
    const themeToggle = page.locator("[data-testid='theme-toggle']");
    await expect(themeToggle).toBeVisible();
  });

  test("theme toggle switches between dark and light mode", async ({ page }) => {
    const themeToggle = page.locator("[data-testid='theme-toggle']");

    // Click toggle and check that the dark class is toggled on <html>
    const htmlElement = page.locator("html");

    // Get initial state
    const initialHasDark = await htmlElement.evaluate((el) =>
      el.classList.contains("dark"),
    );

    // Click toggle
    await themeToggle.click();

    // State should be inverted
    const afterClickHasDark = await htmlElement.evaluate((el) =>
      el.classList.contains("dark"),
    );
    expect(afterClickHasDark).toBe(!initialHasDark);

    // Click again to restore
    await themeToggle.click();
    const restoredHasDark = await htmlElement.evaluate((el) =>
      el.classList.contains("dark"),
    );
    expect(restoredHasDark).toBe(initialHasDark);
  });

  test("active nav item is visually highlighted", async ({ page }) => {
    // Clipboard should be active by default
    const clipboardNav = page.locator("[data-testid='nav-quick-paste']");
    await expect(clipboardNav).toHaveClass(/bg-cp-accent/);

    // Navigate to History
    await page.locator("[data-testid='nav-history']").click();
    const historyNav = page.locator("[data-testid='nav-history']");
    await expect(historyNav).toHaveClass(/bg-cp-accent/);

    // Clipboard should no longer be highlighted
    await expect(clipboardNav).not.toHaveClass(/bg-cp-accent/);
  });
});
