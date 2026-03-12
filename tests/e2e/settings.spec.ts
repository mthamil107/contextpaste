import { test, expect } from "@playwright/test";

test.describe("Settings Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    // Navigate to Settings view
    await page.locator("[data-testid='nav-settings']").click();
    await expect(page.locator("[data-testid='settings-panel']")).toBeVisible();
  });

  test("settings panel renders", async ({ page }) => {
    const panel = page.locator("[data-testid='settings-panel']");
    await expect(panel).toBeVisible();
  });

  test("all 4 tabs are rendered", async ({ page }) => {
    const tabs = [
      { testid: "settings-tab-general", label: "General" },
      { testid: "settings-tab-shortcuts", label: "Shortcuts" },
      { testid: "settings-tab-security", label: "Security" },
      { testid: "settings-tab-ai", label: "AI" },
    ];

    for (const tab of tabs) {
      const button = page.locator(`[data-testid='${tab.testid}']`);
      await expect(button).toBeVisible();
      await expect(button).toHaveText(tab.label);
    }
  });

  test("General tab is active by default", async ({ page }) => {
    const generalTab = page.locator("[data-testid='settings-tab-general']");
    await expect(generalTab).toHaveClass(/border-cp-accent/);

    const generalContent = page.locator(
      "[data-testid='settings-content-general']",
    );
    await expect(generalContent).toBeVisible();
  });

  test("clicking Shortcuts tab shows shortcut settings", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-shortcuts']").click();

    const shortcutContent = page.locator(
      "[data-testid='settings-content-shortcuts']",
    );
    await expect(shortcutContent).toBeVisible();

    // Should show keyboard shortcut labels
    await expect(shortcutContent).toContainText("Quick Paste");
    await expect(shortcutContent).toContainText("History Browser");
  });

  test("clicking Security tab shows security settings", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-security']").click();

    const securityContent = page.locator(
      "[data-testid='settings-content-security']",
    );
    await expect(securityContent).toBeVisible();

    // Should show credential auto-expire setting
    await expect(securityContent).toContainText("Credential auto-expire");
    await expect(securityContent).toContainText("Clear All History");
  });

  test("clicking AI tab shows AI settings", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-ai']").click();

    const aiContent = page.locator("[data-testid='settings-content-ai']");
    await expect(aiContent).toBeVisible();

    await expect(aiContent).toContainText("AI & Predictions");
    await expect(aiContent).toContainText("AI Provider");
  });

  test("can navigate between all tabs", async ({ page }) => {
    const tabIds = ["general", "shortcuts", "security", "ai"] as const;

    for (const id of tabIds) {
      await page.locator(`[data-testid='settings-tab-${id}']`).click();
      const content = page.locator(`[data-testid='settings-content-${id}']`);
      await expect(content).toBeVisible();
    }
  });

  // General Settings form controls
  test("General: theme selector exists with correct options", async ({
    page,
  }) => {
    const themeSelect = page.locator("[data-testid='setting-theme']");
    await expect(themeSelect).toBeVisible();

    const options = themeSelect.locator("option");
    await expect(options).toHaveCount(3);
    await expect(options.nth(0)).toHaveText("System");
    await expect(options.nth(1)).toHaveText("Light");
    await expect(options.nth(2)).toHaveText("Dark");
  });

  test("General: max history items input exists", async ({ page }) => {
    const input = page.locator("[data-testid='setting-max-history-items']");
    await expect(input).toBeVisible();
    await expect(input).toHaveAttribute("type", "number");
  });

  test("General: overlay position selector exists", async ({ page }) => {
    const select = page.locator("[data-testid='setting-overlay-position']");
    await expect(select).toBeVisible();

    const options = select.locator("option");
    await expect(options).toHaveCount(3);
  });

  test("General: overlay max items input exists", async ({ page }) => {
    const input = page.locator("[data-testid='setting-overlay-max-items']");
    await expect(input).toBeVisible();
    await expect(input).toHaveAttribute("type", "number");
  });

  test("General: deduplication checkbox exists and toggles", async ({
    page,
  }) => {
    const checkbox = page.locator("[data-testid='setting-dedup-enabled']");
    await expect(checkbox).toBeVisible();
    await expect(checkbox).toHaveAttribute("type", "checkbox");

    const initialState = await checkbox.isChecked();
    await checkbox.click();
    const newState = await checkbox.isChecked();
    expect(newState).toBe(!initialState);
  });

  test("General: show type badges checkbox exists", async ({ page }) => {
    const checkbox = page.locator("[data-testid='setting-show-type-badges']");
    await expect(checkbox).toBeVisible();
    await expect(checkbox).toHaveAttribute("type", "checkbox");
  });

  test("General: show source context checkbox exists", async ({ page }) => {
    const checkbox = page.locator(
      "[data-testid='setting-show-source-context']",
    );
    await expect(checkbox).toBeVisible();
    await expect(checkbox).toHaveAttribute("type", "checkbox");
  });

  // Security tab controls
  test("Security: credential auto-expire input exists", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-security']").click();

    const input = page.locator(
      "[data-testid='setting-credential-auto-expire']",
    );
    await expect(input).toBeVisible();
    await expect(input).toHaveAttribute("type", "number");
  });

  test("Security: clear expired credentials button exists", async ({
    page,
  }) => {
    await page.locator("[data-testid='settings-tab-security']").click();

    const button = page.locator(
      "[data-testid='btn-clear-expired-credentials']",
    );
    await expect(button).toBeVisible();
    await expect(button).toHaveText(/Clear Expired Credentials/);
  });

  test("Security: clear all history button exists", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-security']").click();

    const button = page.locator("[data-testid='btn-clear-all-history']");
    await expect(button).toBeVisible();
    await expect(button).toHaveText(/Clear All History/);
  });

  // AI tab controls
  test("AI: enable predictions checkbox exists", async ({ page }) => {
    await page.locator("[data-testid='settings-tab-ai']").click();

    const checkbox = page.locator(
      "[data-testid='setting-enable-predictions']",
    );
    await expect(checkbox).toBeVisible();
    await expect(checkbox).toHaveAttribute("type", "checkbox");
  });

  test("AI: provider selector exists with correct options", async ({
    page,
  }) => {
    await page.locator("[data-testid='settings-tab-ai']").click();

    const select = page.locator("[data-testid='setting-ai-provider']");
    await expect(select).toBeVisible();

    const options = select.locator("option");
    await expect(options).toHaveCount(4);
    await expect(options.nth(0)).toHaveText("Local (ONNX)");
    await expect(options.nth(1)).toHaveText("OpenAI");
    await expect(options.nth(2)).toHaveText("Anthropic");
    await expect(options.nth(3)).toHaveText("Ollama");
  });

  test("AI: semantic search checkbox exists and is disabled", async ({
    page,
  }) => {
    await page.locator("[data-testid='settings-tab-ai']").click();

    const checkbox = page.locator(
      "[data-testid='setting-enable-semantic-search']",
    );
    await expect(checkbox).toBeVisible();
    await expect(checkbox).toBeDisabled();
  });
});
