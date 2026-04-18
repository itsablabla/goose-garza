import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SkillsView } from "../SkillsView";

const mockSkills = [
  {
    id: "skill-code-review",
    name: "code-review",
    description: "Reviews code",
    instructions: "Review the code...",
    path: "/path/code-review/SKILL.md",
    directoryPath: "/path/code-review",
    sourceKind: "user",
    sourceLabel: "User",
    projectLinks: [],
    supportingFiles: [],
    symlinkedLocations: [],
    isSymlink: false,
    editable: true,
    duplicateNameCount: 1,
  },
  {
    id: "skill-test-writer",
    name: "test-writer",
    description: "Writes tests",
    instructions: "Write tests...",
    path: "/path/test-writer/SKILL.md",
    directoryPath: "/path/test-writer",
    sourceKind: "project",
    sourceLabel: "Alpha",
    projectLinks: [
      {
        id: "project-alpha",
        name: "Alpha",
        workingDir: "/tmp/alpha",
      },
    ],
    supportingFiles: ["/path/test-writer/template.md"],
    symlinkedLocations: [],
    isSymlink: false,
    editable: true,
    duplicateNameCount: 1,
  },
];

vi.mock("../../api/skills", () => ({
  listSkills: vi.fn().mockResolvedValue([]),
  deleteSkillAtPath: vi.fn().mockResolvedValue(undefined),
  exportSkill: vi
    .fn()
    .mockResolvedValue({ json: "{}", filename: "test.skill.json" }),
  importSkills: vi.fn().mockResolvedValue([]),
}));

const { listSkills, deleteSkillAtPath } = (await import(
  "../../api/skills"
)) as unknown as {
  listSkills: ReturnType<typeof vi.fn>;
  deleteSkillAtPath: ReturnType<typeof vi.fn>;
};

beforeEach(() => {
  vi.clearAllMocks();
  listSkills.mockResolvedValue([]);
});

describe("SkillsView", () => {
  it("shows the redesigned heading and description", () => {
    render(<SkillsView />);
    expect(screen.getByText("Skills")).toBeInTheDocument();
    expect(
      screen.getByText("Search, inspect, and launch reusable instructions."),
    ).toBeInTheDocument();
  });

  it("shows the empty state when no skills are available", async () => {
    render(<SkillsView />);
    await waitFor(() => {
      expect(screen.getByText("No skills yet")).toBeInTheDocument();
    });
    expect(
      screen.getByText("Create a skill or import one to get started."),
    ).toBeInTheDocument();
  });

  it("renders skills and opens the detail subpage", async () => {
    listSkills.mockResolvedValue(mockSkills);
    const user = userEvent.setup();

    render(<SkillsView />);
    await screen.findByText("code-review");

    await user.click(screen.getByRole("button", { name: /test-writer/i }));

    expect(screen.getByRole("button", { name: "Back to skills" })).toBeInTheDocument();
    expect(screen.getByText("Alpha")).toBeInTheDocument();
    expect(screen.getByText("Write tests...")).toBeInTheDocument();
    expect(screen.getByText("/path/test-writer/SKILL.md")).toBeInTheDocument();
  });

  it("returns to the list without losing filters", async () => {
    listSkills.mockResolvedValue(mockSkills);
    const user = userEvent.setup();

    render(<SkillsView />);
    await screen.findByText("code-review");

    await user.click(screen.getByRole("button", { name: "Alpha" }));
    await user.click(screen.getByRole("button", { name: /test-writer/i }));
    await user.click(screen.getByRole("button", { name: "Back to skills" }));

    expect(screen.getByText("test-writer")).toBeInTheDocument();
    expect(screen.queryByText("code-review")).not.toBeInTheDocument();
  });

  it("filters skills by search text", async () => {
    listSkills.mockResolvedValue(mockSkills);
    const user = userEvent.setup();

    render(<SkillsView />);
    await screen.findByText("code-review");

    await user.type(
      screen.getByPlaceholderText("Search skills by name or description..."),
      "writes tests",
    );

    expect(screen.queryByText("code-review")).not.toBeInTheDocument();
    expect(screen.getByText("test-writer")).toBeInTheDocument();
  });

  it("filters skills by project from the main filter row", async () => {
    listSkills.mockResolvedValue(mockSkills);
    const user = userEvent.setup();

    render(<SkillsView />);
    await screen.findByText("code-review");

    await user.click(screen.getByRole("button", { name: "Alpha" }));

    expect(screen.queryByText("code-review")).not.toBeInTheDocument();
    expect(screen.getByText("test-writer")).toBeInTheDocument();
  });

  it("shows a delete confirmation from the detail panel", async () => {
    listSkills.mockResolvedValue(mockSkills);
    const user = userEvent.setup();

    render(<SkillsView />);
    await screen.findByText("code-review");

    await user.click(screen.getByRole("button", { name: /code-review/i }));
    await user.click(screen.getByRole("button", { name: "Delete" }));

    expect(screen.getByText("Delete skill?")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(deleteSkillAtPath).toHaveBeenCalledWith(
        "code-review",
        "/path/code-review/SKILL.md",
      );
    });
  });
});
