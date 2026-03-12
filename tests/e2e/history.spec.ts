import { test, expect } from "@playwright/test";

test.describe("History Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Navigate to History view
    await page.locator("[data-testid='nav-history']").click();
    await expect(page.locator("[data-testid='history-panel']")).toBeVisible();
  });

  test("history panel renders", async ({ page }) => {
    const panel = page.locator("[data-testid='history-panel']");
    await expect(panel).toBeVisible();
  });

  test("filter bar is visible", async ({ page }) => {
    const filterBar = page.locator("[data-testid='history-filter-bar']");
    await expect(filterBar).toBeVisible();
  });

  test("search input is visible in filter bar", async ({ page }) => {
    const searchInput = page.locator(
      "[data-testid='history-search-input']",
    );
    await expect(searchInput).toBeVisible();
    await expect(searchInput).toHaveAttribute("placeholder", /Search history/);
  });

  test("search input accepts text", async ({ page }) => {
    const searchInput = page.locator(
      "[data-testid='history-search-input']",
    );
    await searchInput.fill("test search");
    await expect(searchInput).toHaveValue("test search");
  });

  test("all type filter buttons are rendered", async ({ page }) => {
    const expectedFilters = ["All", "URLs", "Code", "JSON", "SQL", "Shell", "Secrets"];

    for (const label of expectedFilters) {
      const button = page.locator(
        `[data-testid='history-filter-btn-${label.toLowerCase()}']`,
      );
      await expect(button).toBeVisible();
      await expect(button).toHaveText(label);
    }
  });

  test("'All' filter is active by default", async ({ page }) => {
    const allButton = page.locator(
      "[data-testid='history-filter-btn-all']",
    );
    // Active filter should have the accent background class
    await expect(allButton).toHaveClass(/bg-cp-accent/);
  });

  test("clicking a filter button activates it", async ({ page }) => {
    const codeButton = page.locator(
      "[data-testid='history-filter-btn-code']",
    );
    await codeButton.click();

    // Code should now be active
    await expect(codeButton).toHaveClass(/bg-cp-accent/);

    // All should no longer be active
    const allButton = page.locator(
      "[data-testid='history-filter-btn-all']",
    );
    await expect(allButton).not.toHaveClass(/bg-cp-accent/);
  });

  test("clicking different filters switches active state", async ({
    page,
  }) => {
    const filters = ["urls", "code", "json", "sql", "shell", "secrets", "all"];

    for (const filter of filters) {
      const button = page.locator(
        `[data-testid='history-filter-btn-${filter}']`,
      );
      await button.click();
      await expect(button).toHaveClass(/bg-cp-accent/);
    }
  });

  test("empty state is shown when no history", async ({ page }) => {
    const emptyState = page.locator("[data-testid='history-empty']");
    await expect(emptyState).toBeVisible();
    await expect(emptyState).toHaveText(/No clipboard history/);
  });

  test("item list area exists", async ({ page }) => {
    const itemList = page.locator("[data-testid='history-item-list']");
    await expect(itemList).toBeVisible();
  });
});
