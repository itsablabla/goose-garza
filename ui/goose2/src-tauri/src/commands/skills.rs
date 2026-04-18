use super::projects::{list_projects, ProjectInfo as ProjectRecord};
use etcetera::{choose_app_strategy, AppStrategy, AppStrategyArgs};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct SkillMetadata {
    name: String,
    description: String,
}

#[derive(Clone)]
struct ScanTarget {
    dir: PathBuf,
    source_kind: SkillSourceKind,
    project_links: Vec<SkillProjectLink>,
}

#[derive(Clone)]
struct SkillEntry {
    info: SkillInfo,
    canonical_directory: Option<String>,
}

fn skills_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(".agents").join("skills"))
}

fn config_dir() -> Result<PathBuf, String> {
    if let Ok(test_root) = std::env::var("GOOSE_PATH_ROOT") {
        return Ok(PathBuf::from(test_root).join("config"));
    }

    let strategy = choose_app_strategy(AppStrategyArgs {
        top_level_domain: "Block".to_string(),
        author: "Block".to_string(),
        app_name: "goose".to_string(),
    })
    .map_err(|e| format!("Could not determine config directory: {}", e))?;

    Ok(strategy.config_dir())
}

fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_source_label(
    source_kind: SkillSourceKind,
    project_links: &[SkillProjectLink],
) -> String {
    if source_kind == SkillSourceKind::Project {
        let mut names: Vec<_> = project_links.iter().map(|link| link.name.clone()).collect();
        names.sort();
        names.dedup();
        return match names.as_slice() {
            [] => "Project".to_string(),
            [only] => only.clone(),
            [first, rest @ ..] => format!("{} +{}", first, rest.len()),
        };
    }

    source_kind.label().to_string()
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git") | Some(".hg") | Some(".svn")
    )
}

fn walk_files_recursively<F, G>(
    dir: &Path,
    visited_dirs: &mut std::collections::HashSet<PathBuf>,
    should_descend: &mut G,
    visit_file: &mut F,
) where
    F: FnMut(&Path),
    G: FnMut(&Path) -> bool,
{
    let canonical_dir = match std::fs::canonicalize(dir) {
        Ok(path) => path,
        Err(_) => return,
    };

    if !visited_dirs.insert(canonical_dir) {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if should_descend(&path) {
                walk_files_recursively(&path, visited_dirs, should_descend, visit_file);
            }
        } else if path.is_file() {
            visit_file(&path);
        }
    }
}

fn collect_supporting_files(skill_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut visited_support_dirs = std::collections::HashSet::new();
    walk_files_recursively(
        skill_dir,
        &mut visited_support_dirs,
        &mut |path| !should_skip_dir(path) && !path.join("SKILL.md").is_file(),
        &mut |path| {
            if path.file_name().and_then(|n| n.to_str()) != Some("SKILL.md") {
                files.push(normalize_path_key(path));
            }
        },
    );
    files.sort();
    files
}

fn parse_skill_file(skill_file: &Path) -> Option<(SkillMetadata, String)> {
    let content = fs::read_to_string(skill_file).ok()?;
    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return None;
    }

    let metadata = serde_yaml::from_str::<SkillMetadata>(parts[1].trim()).ok()?;
    let body = parts[2..].join("---").trim().to_string();
    Some((metadata, body))
}

fn current_skill_name_and_body(skill_file: &Path) -> Result<(String, String), String> {
    let (metadata, body) =
        parse_skill_file(skill_file).ok_or("Skill file is missing valid frontmatter")?;
    Ok((metadata.name, body))
}

fn canonical_directory_key(path: &Path) -> Option<String> {
    fs::canonicalize(path)
        .ok()
        .map(|canonical| normalize_path_key(&canonical))
}

fn build_skill_info(
    skill_file: &Path,
    metadata: SkillMetadata,
    instructions: String,
    source_kind: SkillSourceKind,
    project_links: Vec<SkillProjectLink>,
) -> SkillEntry {
    let skill_dir = skill_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| skill_file.to_path_buf());
    let path = normalize_path_key(skill_file);
    let directory_path = normalize_path_key(&skill_dir);
    let is_symlink = fs::symlink_metadata(&skill_dir)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
        || fs::symlink_metadata(skill_file)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false);

    let mut projects = project_links;
    projects.sort_by(|a, b| a.name.cmp(&b.name).then(a.working_dir.cmp(&b.working_dir)));
    projects.dedup_by(|a, b| a.id == b.id && a.working_dir == b.working_dir);

    SkillEntry {
        info: SkillInfo {
            id: path.clone(),
            name: metadata.name,
            description: metadata.description,
            instructions,
            path,
            directory_path,
            source_kind,
            source_label: display_source_label(source_kind, &projects),
            project_links: projects,
            supporting_files: collect_supporting_files(&skill_dir),
            symlinked_locations: Vec::new(),
            is_symlink,
            editable: true,
            duplicate_name_count: 1,
        },
        canonical_directory: canonical_directory_key(&skill_dir),
    }
}

fn scan_skills_from_dir(target: &ScanTarget) -> Vec<SkillEntry> {
    let mut skill_files = Vec::new();
    let mut visited_dirs = std::collections::HashSet::new();

    walk_files_recursively(
        &target.dir,
        &mut visited_dirs,
        &mut |path| !should_skip_dir(path),
        &mut |path| {
            if path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
                skill_files.push(path.to_path_buf());
            }
        },
    );

    let mut skills = Vec::new();
    for skill_file in skill_files {
        let Some((metadata, instructions)) = parse_skill_file(&skill_file) else {
            continue;
        };
        skills.push(build_skill_info(
            &skill_file,
            metadata,
            instructions,
            target.source_kind,
            target.project_links.clone(),
        ));
    }

    skills
}

fn project_skill_roots(projects: &[ProjectRecord]) -> Vec<ScanTarget> {
    let mut targets = HashMap::<String, ScanTarget>::new();

    for project in projects {
        let link = SkillProjectLink {
            id: project.id.clone(),
            name: project.name.clone(),
            working_dir: String::new(),
        };

        for working_dir in &project.working_dirs {
            let normalized_working_dir = working_dir.trim();
            if normalized_working_dir.is_empty() {
                continue;
            }

            let project_link = SkillProjectLink {
                working_dir: normalized_working_dir.to_string(),
                ..link.clone()
            };

            for root in [
                PathBuf::from(normalized_working_dir).join(".goose/skills"),
                PathBuf::from(normalized_working_dir).join(".claude/skills"),
                PathBuf::from(normalized_working_dir).join(".agents/skills"),
            ] {
                let key = normalize_path_key(&root);
                targets
                    .entry(key)
                    .and_modify(|target| target.project_links.push(project_link.clone()))
                    .or_insert_with(|| ScanTarget {
                        dir: root,
                        source_kind: SkillSourceKind::Project,
                        project_links: vec![project_link.clone()],
                    });
            }
        }
    }

    let mut values: Vec<_> = targets.into_values().collect();
    values.sort_by(|a, b| normalize_path_key(&a.dir).cmp(&normalize_path_key(&b.dir)));
    values
}

fn global_skill_roots() -> Result<Vec<ScanTarget>, String> {
    let home = dirs::home_dir();
    let config = config_dir()?;

    let mut targets = Vec::new();
    let roots = [
        home.as_ref()
            .map(|h| (h.join(".agents/skills"), SkillSourceKind::User)),
        Some((config.join("skills"), SkillSourceKind::Global)),
        home.as_ref()
            .map(|h| (h.join(".claude/skills"), SkillSourceKind::Global)),
        home.as_ref()
            .map(|h| (h.join(".config/agents/skills"), SkillSourceKind::Global)),
    ];

    for root in roots.into_iter().flatten() {
        targets.push(ScanTarget {
            dir: root.0,
            source_kind: root.1,
            project_links: Vec::new(),
        });
    }

    Ok(targets)
}

fn discover_skills() -> Result<Vec<SkillInfo>, String> {
    let projects = list_projects().unwrap_or_default();
    let mut entries = Vec::new();

    for target in project_skill_roots(&projects) {
        entries.extend(scan_skills_from_dir(&target));
    }

    for target in global_skill_roots()? {
        entries.extend(scan_skills_from_dir(&target));
    }

    let mut canonical_groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, entry) in entries.iter().enumerate() {
        if let Some(key) = &entry.canonical_directory {
            canonical_groups.entry(key.clone()).or_default().push(index);
        }
    }

    let mut dropped_indices = std::collections::HashSet::new();

    for indices in canonical_groups.values() {
        if indices.len() < 2 {
            continue;
        }

        let retain_index = indices
            .iter()
            .copied()
            .min_by(|left, right| {
                let left_entry = &entries[*left].info;
                let right_entry = &entries[*right].info;
                left_entry
                    .is_symlink
                    .cmp(&right_entry.is_symlink)
                    .then(
                        left_entry
                            .directory_path
                            .len()
                            .cmp(&right_entry.directory_path.len()),
                    )
                    .then(left_entry.directory_path.cmp(&right_entry.directory_path))
            })
            .unwrap_or(indices[0]);

        let mut locations: Vec<_> = indices
            .iter()
            .map(|index| entries[*index].info.directory_path.clone())
            .collect();
        locations.sort();
        locations.dedup();

        let mut project_links: Vec<_> = indices
            .iter()
            .flat_map(|index| entries[*index].info.project_links.clone())
            .collect();
        project_links.sort_by(|a, b| a.name.cmp(&b.name).then(a.working_dir.cmp(&b.working_dir)));
        project_links.dedup_by(|a, b| a.id == b.id && a.working_dir == b.working_dir);

        for index in indices {
            if *index != retain_index {
                dropped_indices.insert(*index);
            }
        }

        let retain_entry = &mut entries[retain_index].info;
        retain_entry.project_links = project_links;
        retain_entry.source_label =
            display_source_label(retain_entry.source_kind, &retain_entry.project_links);
        retain_entry.symlinked_locations = locations
            .into_iter()
            .filter(|location| *location != retain_entry.directory_path)
            .collect();
    }

    let mut skills: Vec<_> = entries
        .into_iter()
        .enumerate()
        .filter_map(|(index, entry)| (!dropped_indices.contains(&index)).then_some(entry.info))
        .collect();

    let mut duplicate_counts: HashMap<String, usize> = HashMap::new();
    for skill in &skills {
        *duplicate_counts.entry(skill.name.clone()).or_insert(0) += 1;
    }

    for skill in &mut skills {
        skill.duplicate_name_count = duplicate_counts.get(&skill.name).copied().unwrap_or(1);
    }

    skills.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.name.cmp(&b.name))
            .then(a.source_label.cmp(&b.source_label))
            .then(a.path.cmp(&b.path))
    });
    Ok(skills)
}

fn path_source_kind(path: &Path) -> Result<SkillSourceKind, String> {
    let user_root = skills_dir()?;
    let config_root = config_dir()?.join("skills");
    if path.starts_with(&user_root) {
        return Ok(SkillSourceKind::User);
    }
    if path.starts_with(&config_root) {
        return Ok(SkillSourceKind::Global);
    }
    Ok(SkillSourceKind::Project)
}

fn resolve_skill_file(name: &str, path: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = path {
        let skill_file = PathBuf::from(path);
        if skill_file.file_name().and_then(|value| value.to_str()) != Some("SKILL.md") {
            return Err("Expected a SKILL.md file path".to_string());
        }
        if !skill_file.exists() {
            return Err("Skill file not found".to_string());
        }
        let normalized_path = normalize_path_key(&skill_file);
        let is_discovered = discover_skills()?
            .into_iter()
            .any(|skill| skill.path == normalized_path);
        if !is_discovered {
            return Err("Skill file is not part of the discovered skills library".to_string());
        }
        return Ok(skill_file);
    }

    validate_skill_name(name)?;
    let skill_file = skills_dir()?.join(name).join("SKILL.md");
    if !skill_file.exists() {
        return Err(format!("Skill \"{}\" not found", name));
    }
    Ok(skill_file)
}

fn skill_info_for_file(skill_file: &Path) -> Result<SkillInfo, String> {
    let path = normalize_path_key(skill_file);
    if let Some(skill) = discover_skills()?
        .into_iter()
        .find(|skill| skill.path == path)
    {
        return Ok(skill);
    }

    let (metadata, instructions) =
        parse_skill_file(skill_file).ok_or("Skill file is missing valid frontmatter")?;
    Ok(build_skill_info(
        skill_file,
        metadata,
        instructions,
        path_source_kind(skill_file)?,
        Vec::new(),
    )
    .info)
}

/// Validates that a skill name is kebab-case only: `^[a-z0-9]+(-[a-z0-9]+)*$`.
/// This prevents path traversal attacks (e.g. `../../.ssh/authorized_keys`).
fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Skill name must not be empty".to_string());
    }
    let mut expect_alnum = true;
    for ch in name.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            expect_alnum = false;
        } else if ch == '-' && !expect_alnum {
            expect_alnum = true;
        } else {
            return Err(format!(
                "Invalid skill name \"{}\". Names must be kebab-case (lowercase letters, digits, and hyphens; \
                 must not start or end with a hyphen or contain consecutive hyphens).",
                name
            ));
        }
    }
    if expect_alnum {
        return Err(format!(
            "Invalid skill name \"{}\". Names must not end with a hyphen.",
            name
        ));
    }
    Ok(())
}

fn build_skill_md(name: &str, description: &str, instructions: &str) -> String {
    let safe_desc = description.replace('\'', "''");
    let mut md = format!("---\nname: {}\ndescription: '{}'\n---\n", name, safe_desc);
    if !instructions.is_empty() {
        md.push('\n');
        md.push_str(instructions);
        md.push('\n');
    }
    md
}

#[derive(serde::Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SkillSourceKind {
    Project,
    User,
    Global,
}

impl SkillSourceKind {
    fn label(self) -> &'static str {
        match self {
            SkillSourceKind::Project => "Project",
            SkillSourceKind::User => "Personal",
            SkillSourceKind::Global => "Personal",
        }
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillProjectLink {
    pub id: String,
    pub name: String,
    pub working_dir: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub path: String,
    pub directory_path: String,
    pub source_kind: SkillSourceKind,
    pub source_label: String,
    pub project_links: Vec<SkillProjectLink>,
    pub supporting_files: Vec<String>,
    pub symlinked_locations: Vec<String>,
    pub is_symlink: bool,
    pub editable: bool,
    pub duplicate_name_count: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillExportV1 {
    version: u32,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    instructions: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSkillResult {
    json: String,
    filename: String,
}

#[tauri::command]
pub fn create_skill(name: String, description: String, instructions: String) -> Result<(), String> {
    validate_skill_name(&name)?;
    let dir = skills_dir()?.join(&name);

    if dir.exists() {
        return Err(format!("A skill named \"{}\" already exists", name));
    }

    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create skill directory: {}", e))?;

    let skill_path = dir.join("SKILL.md");
    let content = build_skill_md(&name, &description, &instructions);

    fs::write(&skill_path, content).map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn list_skills() -> Result<Vec<SkillInfo>, String> {
    discover_skills()
}

#[tauri::command]
pub fn delete_skill(name: String, path: Option<String>) -> Result<(), String> {
    let skill_file = resolve_skill_file(&name, path)?;
    let skill_dir = skill_file
        .parent()
        .ok_or("Could not resolve skill directory")?;
    if !skill_dir.exists() {
        return Err(format!("Skill \"{}\" not found", name));
    }
    fs::remove_dir_all(skill_dir).map_err(|e| format!("Failed to delete skill: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_skill(
    name: String,
    description: String,
    instructions: String,
    path: Option<String>,
) -> Result<SkillInfo, String> {
    let skill_file = resolve_skill_file(&name, path)?;
    let (current_name, _) = current_skill_name_and_body(&skill_file)?;
    let content = build_skill_md(&current_name, &description, &instructions);

    fs::write(&skill_file, content).map_err(|e| format!("Failed to write SKILL.md: {}", e))?;
    skill_info_for_file(&skill_file)
}

#[tauri::command]
pub fn export_skill(name: String, path: Option<String>) -> Result<ExportSkillResult, String> {
    let skill_file = resolve_skill_file(&name, path)?;
    let (metadata, instructions) =
        parse_skill_file(&skill_file).ok_or("Skill file is missing valid frontmatter")?;
    let skill_name = metadata.name;
    let description = metadata.description;

    let export = SkillExportV1 {
        version: 1,
        name: skill_name.clone(),
        description,
        instructions,
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize skill: {}", e))?;

    let filename = format!("{}.skill.json", skill_name);

    Ok(ExportSkillResult { json, filename })
}

#[tauri::command]
pub fn import_skills(file_bytes: Vec<u8>, file_name: String) -> Result<Vec<SkillInfo>, String> {
    if !file_name.ends_with(".skill.json") && !file_name.ends_with(".json") {
        return Err("File must have a .skill.json or .json extension".to_string());
    }

    let text =
        String::from_utf8(file_bytes).map_err(|e| format!("File is not valid UTF-8: {}", e))?;

    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid JSON: {}", e))?;

    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid \"version\" field")?;
    if version != 1 {
        return Err(format!("Unsupported skill export version: {}", version));
    }

    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid \"name\" field")?
        .to_string();
    if name.is_empty() {
        return Err("Skill name must not be empty".to_string());
    }

    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid \"description\" field")?
        .to_string();
    if description.is_empty() {
        return Err("Skill description must not be empty".to_string());
    }

    let instructions = value
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    validate_skill_name(&name)?;

    let base_dir = skills_dir()?;
    let mut final_name = name.clone();
    if base_dir.join(&final_name).exists() {
        final_name = format!("{}-imported", name);
        let mut counter = 2u32;
        while base_dir.join(&final_name).exists() {
            final_name = format!("{}-imported-{}", name, counter);
            counter += 1;
        }
    }

    let dir = base_dir.join(&final_name);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create skill directory: {}", e))?;

    let skill_path = dir.join("SKILL.md");
    let content = build_skill_md(&final_name, &description, &instructions);
    fs::write(&skill_path, content).map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    Ok(vec![skill_info_for_file(&skill_path)?])
}
