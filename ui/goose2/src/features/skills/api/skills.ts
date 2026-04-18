import { invoke } from "@tauri-apps/api/core";

export interface SkillProjectLink {
  id: string;
  name: string;
  workingDir: string;
}

export type SkillSourceKind = "project" | "user" | "global";

export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  instructions: string;
  path: string;
  directoryPath: string;
  sourceKind: SkillSourceKind;
  sourceLabel: string;
  projectLinks: SkillProjectLink[];
  supportingFiles: string[];
  symlinkedLocations: string[];
  isSymlink: boolean;
  editable: boolean;
  duplicateNameCount: number;
}

export async function createSkill(
  name: string,
  description: string,
  instructions: string,
): Promise<void> {
  return invoke("create_skill", { name, description, instructions });
}

export async function listSkills(): Promise<SkillInfo[]> {
  return invoke("list_skills");
}

export async function deleteSkill(name: string): Promise<void> {
  return invoke("delete_skill", { name });
}

export async function deleteSkillAtPath(
  name: string,
  path: string,
): Promise<void> {
  return invoke("delete_skill", { name, path });
}

export async function updateSkill(
  name: string,
  description: string,
  instructions: string,
  path?: string,
): Promise<SkillInfo> {
  return invoke("update_skill", { name, description, instructions, path });
}

export async function exportSkill(
  name: string,
  path?: string,
): Promise<{ json: string; filename: string }> {
  return invoke("export_skill", { name, path });
}

export async function importSkills(
  fileBytes: number[],
  fileName: string,
): Promise<SkillInfo[]> {
  return invoke("import_skills", {
    fileBytes: Array.from(fileBytes),
    fileName,
  });
}
