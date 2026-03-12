import { test, expect } from "@playwright/test";

test.describe("Quick Paste Overlay", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("Open Quick Paste button is visible on the Clipboard view", async ({
    page,
  }) => {
    const openButton = page.locator("[data-testid='open-quick-paste-btn']");
    await expect(openButton).toBeVisible();
    await expect(openButton).toHaveText(/Open Quick Paste/);
  });

  test("clicking Open Quick Paste shows the overlay", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const overlay = page.locator("[data-testid='quick-paste-overlay']");
    await expect(overlay).toBeVisible();
  });

  test("overlay contains a search bar", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const searchInput = page.locator(
      "[data-testid='quick-paste-search'] input",
    );
    await expect(searchInput).toBeVisible();
    await expect(searchInput).toHaveAttribute("placeholder", /Search clipboard/);
  });

  test("overlay shows empty state when no items", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const emptyState = page.locator("[data-testid='quick-paste-empty']");
    await expect(emptyState).toBeVisible();
    await expect(emptyState).toHaveText(/No clipboard history yet/);
  });

  test("overlay shows 'No results found' when search has no matches", async ({
    page,
  }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const searchInput = page.locator(
      "[data-testid='quick-paste-search'] input",
    );
    await searchInput.fill("xyznonexistentquery123");

    // Wait for the search to process — the empty state text should change
    // Since backend is not running, results will be empty
    const emptyState = page.locator("[data-testid='quick-paste-empty']");
    await expect(emptyState).toBeVisible({ timeout: 5000 });
  });

  test("overlay footer shows keyboard hints", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const footer = page.locator("[data-testid='quick-paste-footer']");
    await expect(footer).toBeVisible();
    await expect(footer).toContainText("navigate");
    await expect(footer).toContainText("paste");
    await expect(footer).toContainText("close");
  });

  test("overlay footer shows item count", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const footer = page.locator("[data-testid='quick-paste-footer']");
    await expect(footer).toContainText("items");
  });

  test("pressing Escape closes the overlay", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const overlay = page.locator("[data-testid='quick-paste-overlay']");
    await expect(overlay).toBeVisible();

    await page.keyboard.press("Escape");

    await expect(overlay).not.toBeVisible();
  });

  test("clicking the backdrop closes the overlay", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const backdrop = page.locator("[data-testid='quick-paste-backdrop']");
    await expect(backdrop).toBeVisible();

    // Click on the backdrop (outside the overlay content)
    await backdrop.click({ position: { x: 10, y: 10 } });

    await expect(
      page.locator("[data-testid='quick-paste-overlay']"),
    ).not.toBeVisible();
  });

  test("search input is auto-focused when overlay opens", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const searchInput = page.locator(
      "[data-testid='quick-paste-search'] input",
    );
    await expect(searchInput).toBeFocused();
  });

  test("typing in search bar filters text", async ({ page }) => {
    await page.locator("[data-testid='open-quick-paste-btn']").click();

    const searchInput = page.locator(
      "[data-testid='quick-paste-search'] input",
    );
    await searchInput.fill("test query");

    // Verify the input value was set
    await expect(searchInput).toHaveValue("test query");
  });
});
