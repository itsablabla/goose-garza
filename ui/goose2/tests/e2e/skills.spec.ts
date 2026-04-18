import {
  test,
  expect,
  navigateToSkills,
  buildInitScript,
} from "./fixtures/tauri-mock";

test.describe("Skills view", () => {
  test("navigates to skills view and shows the redesigned header", async ({
    tauriMocked: page,
  }) => {
    await navigateToSkills(page);

    await expect(page.locator("h1", { hasText: "Skills" })).toBeVisible();
    await expect(
      page.getByText("Search, inspect, and launch reusable instructions."),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Import" })).toBeVisible();
    await expect(page.getByRole("button", { name: "New Skill" })).toBeVisible();
  });

  test("shows skills in the list and opens a dedicated detail page", async ({
    tauriMocked: page,
  }) => {
    await navigateToSkills(page);

    await expect(page.getByText("code-review")).toBeVisible();
    await expect(page.getByText("test-writer")).toBeVisible();

    await page.getByRole("button", { name: "Open test-writer details" }).click();

    await expect(page.getByRole("button", { name: "Back to skills" })).toBeVisible();
    await expect(page.locator("h1", { hasText: "test-writer" })).toBeVisible();
    await expect(page.getByText("Alpha")).toBeVisible();
    await expect(page.getByText("/tmp/alpha")).toBeVisible();
    await expect(page.getByText("/mock/.agents/skills/test-writer/SKILL.md")).toBeVisible();
    await expect(page.getByText("Write tests...", { exact: false })).toBeVisible();
  });

  test("search filters the list", async ({ tauriMocked: page }) => {
    await navigateToSkills(page);

    await page
      .getByPlaceholder("Search skills by name or description...")
      .fill("review");

    await expect(page.getByText("code-review")).toBeVisible();
    await expect(page.getByText("test-writer")).not.toBeVisible();
  });

  test("project filtering isolates project skills from the main filter row", async ({ tauriMocked: page }) => {
    await navigateToSkills(page);

    await page.getByRole("button", { name: "Alpha" }).click();

    await expect(page.getByText("test-writer")).toBeVisible();
    await expect(page.getByText("code-review")).not.toBeVisible();
  });

  test("project filtering isolates a project's skills", async ({
    tauriMocked: page,
  }) => {
    await navigateToSkills(page);

    await page.getByRole("button", { name: "Alpha" }).first().click();

    await expect(page.getByText("test-writer")).toBeVisible();
    await expect(page.getByText("code-review")).not.toBeVisible();
  });

  test("opens the create skill dialog", async ({ tauriMocked: page }) => {
    await navigateToSkills(page);

    await page.getByRole("button", { name: "New Skill" }).click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog.locator("h2", { hasText: "New Skill" })).toBeVisible();
    await expect(dialog.getByPlaceholder("my-skill-name")).toBeVisible();
    await expect(
      dialog.getByPlaceholder("What it does and when to use it..."),
    ).toBeVisible();
  });

  test("shows the empty state when no skills are available", async ({
    tauriMocked: page,
  }) => {
    await page.addInitScript({
      content: buildInitScript({ personas: [], projects: [], skills: [] }),
    });

    await navigateToSkills(page);

    await expect(page.getByText("No skills yet")).toBeVisible();
    await expect(
      page.getByText("Create a skill or import one to get started."),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "New Skill" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Import" })).toBeVisible();
  });
});
