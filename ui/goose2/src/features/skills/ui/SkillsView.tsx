import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ArrowLeft,
  AtSign,
  FolderOpen,
  Pencil,
  Plus,
  Share2,
  Trash2,
  Upload,
} from "lucide-react";
import { cn } from "@/shared/lib/cn";
import { SearchBar } from "@/shared/ui/SearchBar";
import { Badge } from "@/shared/ui/badge";
import { Button, buttonVariants } from "@/shared/ui/button";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionSectionTrigger,
} from "@/shared/ui/accordion";
import {
  DetailPageShell,
  FilterRow,
  PageHeader,
  PageShell,
} from "@/shared/ui/page-shell";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/shared/ui/alert-dialog";
import { useFileImportZone } from "@/shared/hooks/useFileImportZone";
import { revealInFileManager } from "@/shared/lib/fileManager";
import { CreateSkillDialog } from "./CreateSkillDialog";
import {
  deleteSkillAtPath,
  exportSkill,
  importSkills,
  listSkills,
  type SkillInfo,
} from "../api/skills";

type SkillsFilter = "all" | "global" | `project:${string}`;

interface SkillsViewProps {
  onStartChatWithSkill?: (skillName: string, projectId?: string | null) => void;
}

function uniqueProjectFilters(skills: SkillInfo[]) {
  const seen = new Map<string, string>();
  for (const skill of skills) {
    for (const project of skill.projectLinks) {
      if (!seen.has(project.id)) {
        seen.set(project.id, project.name);
      }
    }
  }
  return [...seen.entries()]
    .map(([id, name]) => ({ id, name }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

function displaySourceLabel(skill: SkillInfo): string {
  return skill.sourceKind === "project" ? skill.sourceLabel : "Personal";
}

function compareSkillsByName(a: SkillInfo, b: SkillInfo) {
  return (
    a.name.localeCompare(b.name, undefined, { sensitivity: "base" }) ||
    a.name.localeCompare(b.name) ||
    a.path.localeCompare(b.path)
  );
}

function formatSkillCount(count: number) {
  return `${count} skill${count === 1 ? "" : "s"}`;
}

function downloadExport(json: string, filename: string) {
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}

function FilterButton({
  active,
  children,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      size="xs"
      variant={active ? "default" : "outline-flat"}
      onClick={onClick}
    >
      {children}
    </Button>
  );
}

function SkillDetailPage({
  skill,
  onBack,
  onEdit,
  onReveal,
  onShare,
  onStartChat,
  onDelete,
}: {
  skill: SkillInfo | null;
  onBack: () => void;
  onEdit: (skill: SkillInfo) => void;
  onReveal: (skill: SkillInfo) => void;
  onShare: (skill: SkillInfo) => void;
  onStartChat: (skill: SkillInfo) => void;
  onDelete: (skill: SkillInfo) => void;
}) {
  const { t } = useTranslation(["skills", "common"]);

  if (!skill) {
    return (
      <div className="flex h-full flex-col justify-center px-1 text-sm text-muted-foreground">
        <p className="text-sm text-foreground">{t("view.detailEmptyTitle")}</p>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("view.detailEmptyDescription")}
        </p>
      </div>
    );
  }

  return (
    <DetailPageShell>
      <div className="space-y-5 border-b border-border pb-6">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="w-fit px-0 text-muted-foreground hover:text-foreground"
          onClick={onBack}
        >
          <ArrowLeft className="size-4" />
          {t("view.backToSkills")}
        </Button>

        <PageHeader
          title={
            <div>
              <div className="flex flex-wrap gap-2">
                {skill.projectLinks.length > 0 ? (
                  [...new Set(skill.projectLinks.map((project) => project.name))].map(
                    (label) => (
                      <Badge key={label} variant="secondary" className="font-normal">
                        {label}
                      </Badge>
                    ),
                  )
                ) : (
                  <Badge variant="secondary" className="font-normal">
                    {displaySourceLabel(skill)}
                  </Badge>
                )}
              </div>
              <span className="mt-4 block text-2xl font-normal tracking-tight text-foreground">
                {skill.name}
              </span>
            </div>
          }
          titleElement="div"
          description={skill.description}
          descriptionClassName="max-w-3xl leading-relaxed"
          actions={
            <>
              <Button
                type="button"
                size="xs"
                variant="outline-flat"
                onClick={() => onStartChat(skill)}
              >
                <MessageSquarePlus className="size-3.5" />
                {t("view.startChat", { name: skill.name })}
              </Button>
              {skill.editable ? (
                <Button
                  type="button"
                  size="xs"
                  variant="outline-flat"
                  onClick={() => onEdit(skill)}
                >
                  <Pencil className="size-3.5" />
                  {t("common:actions.edit")}
                </Button>
              ) : null}
              <Button
                type="button"
                size="xs"
                variant="outline-flat"
                onClick={() => onShare(skill)}
              >
                <Share2 className="size-3.5" />
                {t("view.share")}
              </Button>
              <Button
                type="button"
                size="xs"
                variant="outline-flat"
                onClick={() => onReveal(skill)}
              >
                <FolderOpen className="size-3.5" />
                {t("view.reveal")}
              </Button>
            </>
          }
          className="items-start"
          actionsClassName="flex-wrap"
        />
      </div>

        <div className="grid gap-10 lg:grid-cols-[minmax(0,18rem)_minmax(0,1fr)]">
          <aside className="space-y-5">
            <section className="space-y-3 border-b border-border pb-5">
              <div>
                <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                  {t("view.source")}
                </p>
                <div className="mt-2 flex flex-wrap gap-2">
                  {skill.projectLinks.length > 0 ? (
                    [...new Set(skill.projectLinks.map((project) => project.name))].map(
                      (label) => (
                        <Badge key={label} variant="secondary" className="font-normal">
                          {label}
                        </Badge>
                      ),
                    )
                  ) : (
                    <Badge variant="secondary" className="font-normal">
                      {displaySourceLabel(skill)}
                    </Badge>
                  )}
                </div>
              </div>

              {skill.projectLinks.length > 0 ? (
                <div>
                  <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                    {t("view.projects")}
                  </p>
                  <div className="mt-2 space-y-1.5 text-sm text-foreground">
                    {skill.projectLinks.map((project) => (
                      <div key={`${project.id}-${project.workingDir}`}>
                        <p>{project.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {project.workingDir}
                        </p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}

              <div>
                <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                  {t("view.location")}
                </p>
                <p className="mt-2 text-sm text-foreground">{skill.directoryPath}</p>
              </div>

              <div>
                <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                  {t("view.filePath")}
                </p>
                <p className="mt-2 break-all text-sm text-muted-foreground">
                  {skill.path}
                </p>
              </div>

              {skill.symlinkedLocations.length > 0 ? (
                <div>
                  <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                    {t("view.symlinkedLocations")}
                  </p>
                  <div className="mt-2 space-y-1.5">
                    {skill.symlinkedLocations.map((location) => (
                      <p key={location} className="break-all text-sm text-muted-foreground">
                        {location}
                      </p>
                    ))}
                  </div>
                </div>
              ) : null}

              {skill.duplicateNameCount > 1 ? (
                <p className="text-xs text-muted-foreground">
                  {t("view.duplicateNames", {
                    count: skill.duplicateNameCount,
                  })}
                </p>
              ) : null}
            </section>

            {skill.supportingFiles.length > 0 ? (
              <section className="space-y-3 border-b border-border pb-5">
                <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                  {t("view.supportingFiles")}
                </p>
                <div className="space-y-1.5">
                  {skill.supportingFiles.map((file) => (
                    <p key={file} className="break-all text-sm text-muted-foreground">
                      {file}
                    </p>
                  ))}
                </div>
              </section>
            ) : null}
          </aside>

          <section className="space-y-3 pb-6">
            <div className="flex items-center justify-between gap-3 border-b border-border pb-4">
              <p className="text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                {t("view.instructions")}
              </p>
              {skill.editable ? (
                <Button
                  type="button"
                  size="xs"
                  variant="ghost-light"
                  onClick={() => onDelete(skill)}
                >
                  <Trash2 className="size-3.5" />
                  {t("common:actions.delete")}
                </Button>
              ) : null}
            </div>
            <pre className="overflow-x-auto whitespace-pre-wrap text-sm leading-relaxed text-foreground">
              {skill.instructions || " "}
            </pre>
          </section>
      </div>
    </DetailPageShell>
  );
}

export function SkillsView({ onStartChatWithSkill }: SkillsViewProps) {
  const { t } = useTranslation(["skills", "common"]);
  const [search, setSearch] = useState("");
  const [activeFilter, setActiveFilter] = useState<SkillsFilter>("all");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingSkill, setEditingSkill] = useState<
    { name: string; description: string; instructions: string; path?: string } | undefined
  >(undefined);
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [deletingSkill, setDeletingSkill] = useState<SkillInfo | null>(null);
  const [notification, setNotification] = useState<string | null>(null);
  const [activeSkillId, setActiveSkillId] = useState<string | null>(null);
  const [expandedSectionIds, setExpandedSectionIds] = useState<string[]>([]);
  const importInputRef = useRef<HTMLInputElement>(null);

  const loadSkills = useCallback(async () => {
    try {
      const result = await listSkills();
      setSkills(result);
    } catch {
      setSkills([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  const projectFilters = useMemo(() => uniqueProjectFilters(skills), [skills]);

  useEffect(() => {
    if (!activeFilter.startsWith("project:")) {
      return;
    }

    const projectId = activeFilter.slice("project:".length);
    if (!projectFilters.some((project) => project.id === projectId)) {
      setActiveFilter("all");
    }
  }, [activeFilter, projectFilters]);

  const filteredSkills = useMemo(() => {
    const searchTerm = search.trim().toLowerCase();
    return skills.filter((skill) => {
      const matchesSearch =
        searchTerm.length === 0 ||
        skill.name.toLowerCase().includes(searchTerm) ||
        skill.description.toLowerCase().includes(searchTerm) ||
        displaySourceLabel(skill).toLowerCase().includes(searchTerm);

      const matchesFilter =
        activeFilter === "all"
          ? true
          : activeFilter === "global"
              ? skill.sourceKind === "global" || skill.sourceKind === "user"
              : skill.projectLinks.some(
                  (project) => `project:${project.id}` === activeFilter,
                );

      return matchesSearch && matchesFilter;
    });
  }, [activeFilter, search, skills]);

  const groupedSkills = useMemo(() => {
    if (activeFilter === "global") {
      return [
        {
          id: "personal",
          title: t("view.filtersGlobal"),
          skills: [...filteredSkills].sort(compareSkillsByName),
        },
      ];
    }

    if (activeFilter.startsWith("project:")) {
      const projectId = activeFilter.slice("project:".length);
      const projectName =
        projectFilters.find((project) => project.id === projectId)?.name ??
        t("view.projects");

      return [
        {
          id: activeFilter,
          title: projectName,
          skills: [...filteredSkills].sort(compareSkillsByName),
        },
      ];
    }

    const personalSkills = filteredSkills
      .filter((skill) => skill.sourceKind !== "project")
      .sort(compareSkillsByName);

    const projectSections = projectFilters
      .map((project) => ({
        id: `project:${project.id}`,
        title: project.name,
        skills: filteredSkills
          .filter((skill) =>
            skill.projectLinks.some((link) => link.id === project.id),
          )
          .sort(compareSkillsByName),
      }))
      .filter((section) => section.skills.length > 0);

    return [
      ...(personalSkills.length > 0
        ? [
            {
              id: "personal",
              title: t("view.filtersGlobal"),
              skills: personalSkills,
            },
          ]
        : []),
      ...projectSections,
    ];
  }, [activeFilter, filteredSkills, projectFilters, t]);

  useEffect(() => {
    const nextIds = groupedSkills.map((section) => section.id);
    setExpandedSectionIds((prev) => {
      const stillVisible = prev.filter((id) => nextIds.includes(id));
      const newIds = nextIds.filter((id) => !stillVisible.includes(id));
      return [...stillVisible, ...newIds];
    });
  }, [groupedSkills]);

  const activeSkill =
    skills.find((skill) => skill.id === activeSkillId) ??
    null;

  const handleDelete = (skill: SkillInfo) => {
    setDeletingSkill(skill);
  };

  const handleConfirmDeleteSkill = async () => {
    if (!deletingSkill) return;
    try {
      await deleteSkillAtPath(deletingSkill.name, deletingSkill.path);
      await loadSkills();
      if (activeSkillId === deletingSkill.id) {
        setActiveSkillId(null);
      }
    } catch {
      // best-effort
    }
    setDeletingSkill(null);
  };

  const handleEdit = (skill: SkillInfo) => {
    setEditingSkill({
      name: skill.name,
      description: skill.description,
      instructions: skill.instructions,
      path: skill.path,
    });
    setDialogOpen(true);
  };

  const handleExport = async (skill: SkillInfo) => {
    try {
      const result = await exportSkill(skill.name, skill.path);
      downloadExport(result.json, result.filename);
      setNotification(t("view.exportedTo", { filename: result.filename }));
      setTimeout(() => setNotification(null), 3000);
    } catch (err) {
      console.error("Failed to export skill:", err);
    }
  };

  const handleReveal = useCallback((skill: SkillInfo) => {
    void revealInFileManager(skill.path);
  }, []);

  const handleStartChat = useCallback(
    (skill: SkillInfo) => {
      onStartChatWithSkill?.(skill.name, skill.projectLinks[0]?.id ?? null);
    },
    [onStartChatWithSkill],
  );

  const handleImportFile = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      try {
        const arrayBuffer = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(arrayBuffer));
        await importSkills(bytes, file.name);
        await loadSkills();
      } catch (error) {
        console.error("Failed to import skill:", error);
      }

      if (importInputRef.current) {
        importInputRef.current.value = "";
      }
    },
    [loadSkills],
  );

  const handleDialogClose = () => {
    setDialogOpen(false);
    setEditingSkill(undefined);
  };

  const handleNewSkill = () => {
    setEditingSkill(undefined);
    setDialogOpen(true);
  };

  const handleDropImport = useCallback(
    async (fileBytes: number[], fileName: string) => {
      try {
        await importSkills(fileBytes, fileName);
        await loadSkills();
      } catch (error) {
        console.error("Failed to import skill:", error);
      }
    },
    [loadSkills],
  );

  const {
    fileInputRef: dropFileInputRef,
    isDragOver,
    dropHandlers,
    handleFileChange: handleDropFileChange,
  } = useFileImportZone({ onImportFile: handleDropImport });

  const handleSelectSkill = (skill: SkillInfo) => {
    setActiveSkillId(skill.id);
  };

  if (activeSkill) {
    return (
      <>
        <SkillDetailPage
          skill={activeSkill}
          onBack={() => setActiveSkillId(null)}
          onEdit={handleEdit}
          onReveal={handleReveal}
          onShare={handleExport}
          onStartChat={handleStartChat}
          onDelete={handleDelete}
        />

        <CreateSkillDialog
          isOpen={dialogOpen}
          onClose={handleDialogClose}
          onCreated={loadSkills}
          editingSkill={editingSkill}
        />

        <AlertDialog
          open={!!deletingSkill}
          onOpenChange={(open) => !open && setDeletingSkill(null)}
        >
          <AlertDialogContent className="max-w-sm">
            <AlertDialogHeader>
              <AlertDialogTitle>{t("view.deleteTitle")}</AlertDialogTitle>
              <AlertDialogDescription>
                {t("view.deleteDescription", {
                  name: deletingSkill?.name ?? "",
                })}
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{t("common:actions.cancel")}</AlertDialogCancel>
              <AlertDialogAction
                className={buttonVariants({ variant: "destructive" })}
                onClick={handleConfirmDeleteSkill}
              >
                {t("common:actions.delete")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>

        {notification && (
          <div className="fixed bottom-4 right-4 z-50 rounded-lg border border-border bg-background px-4 py-3 text-sm shadow-popover animate-in fade-in slide-in-from-bottom-2">
            {notification}
          </div>
        )}
      </>
    );
  }

  return (
    <PageShell>
      <PageHeader
        title={t("view.title")}
        description={t("view.description")}
        titleClassName="font-normal text-foreground"
        actions={
          <>
            <input
              ref={importInputRef}
              type="file"
              accept=".skill.json,.json"
              className="hidden"
              onChange={handleImportFile}
            />
            <Button
              type="button"
              variant="outline-flat"
              size="xs"
              onClick={() => importInputRef.current?.click()}
            >
              <Upload className="size-3.5" />
              {t("common:actions.import")}
            </Button>
            <Button
              type="button"
              variant="outline-flat"
              size="xs"
              onClick={handleNewSkill}
            >
              <Plus className="size-3.5" />
              {t("view.newSkill")}
            </Button>
          </>
        }
      />

      <div
        {...dropHandlers}
        className={cn("rounded-2xl transition-colors", isDragOver && "bg-muted/50")}
      >
        <div className="space-y-3">
          <SearchBar
            value={search}
            onChange={setSearch}
            placeholder={t("view.searchPlaceholder")}
          />

          <FilterRow>
            <FilterButton
              active={activeFilter === "all"}
              onClick={() => setActiveFilter("all")}
            >
              {t("view.filtersAllSources")}
            </FilterButton>
            <FilterButton
              active={activeFilter === "global"}
              onClick={() => setActiveFilter("global")}
            >
              {t("view.filtersGlobal")}
            </FilterButton>
            {projectFilters.map((project) => {
              const filterValue = `project:${project.id}` as const;
              return (
                <FilterButton
                  key={project.id}
                  active={activeFilter === filterValue}
                  onClick={() => setActiveFilter(filterValue)}
                >
                  {project.name}
                </FilterButton>
              );
            })}
          </FilterRow>
        </div>
      </div>

      {!loading && filteredSkills.length > 0 ? (
        <Accordion
          type="multiple"
          value={expandedSectionIds}
          onValueChange={setExpandedSectionIds}
          className="min-h-0 space-y-6"
        >
          {groupedSkills.map((section) => (
            <AccordionItem
              key={section.id}
              value={section.id}
              className="group/skills-section overflow-hidden rounded-2xl !border !border-border-soft bg-background"
            >
              <AccordionSectionTrigger
                title={section.title}
                meta={formatSkillCount(section.skills.length)}
              />

              <AccordionContent className="pb-0">
                <div className="motion-safe:group-data-[state=closed]/skills-section:animate-accordion-content-close motion-safe:group-data-[state=open]/skills-section:animate-accordion-content-open border-t border-border-soft-divider will-change-[opacity,transform]">
                  <div className="divide-y divide-border-soft-divider">
                    {section.skills.map((skill) => (
                      <div
                        key={`${section.id}-${skill.id}`}
                        className="group px-5 py-4 transition-colors hover:bg-muted/20"
                      >
                        <button
                          type="button"
                          className="block w-full min-w-0 text-left"
                          onClick={() => handleSelectSkill(skill)}
                          aria-label={t("view.openDetails", { name: skill.name })}
                        >
                          <div className="min-w-0">
                            <div className="flex items-center gap-2">
                              <p className="text-sm font-normal text-foreground">
                                {skill.name}
                              </p>
                              <Button
                                type="button"
                                variant="inline-subtle"
                                size="xs"
                                className="opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  handleStartChat(skill);
                                }}
                                aria-label={t("view.startChat", { name: skill.name })}
                              >
                                {t("view.useInChat")}
                              </Button>
                            </div>
                            {skill.description ? (
                              <p className="mt-1 line-clamp-2 text-xs font-light text-muted-foreground">
                                {skill.description}
                              </p>
                            ) : null}
                          </div>
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      ) : null}

      {!loading && filteredSkills.length === 0 ? (
        <div
          {...dropHandlers}
          className={cn(
            "flex flex-col items-center justify-center gap-3 rounded-2xl py-16 text-muted-foreground transition-colors",
            isDragOver ? "bg-muted/40" : "border-transparent",
          )}
        >
          <AtSign className="h-10 w-10 opacity-30" />
          <div className="text-center">
            <p className="text-sm font-normal text-foreground">
              {skills.length === 0 ? t("view.emptyTitle") : t("view.noMatchesTitle")}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              {skills.length === 0
                ? t("view.emptyDescription")
                : t("view.noMatchesDescription")}
            </p>
          </div>
          {skills.length === 0 ? (
            <div className="mt-2 flex items-center gap-2">
              <Button
                type="button"
                variant="outline-flat"
                size="xs"
                onClick={handleNewSkill}
              >
                <Plus className="size-3.5" />
                {t("view.newSkill")}
              </Button>
              <Button
                type="button"
                variant="outline-flat"
                size="xs"
                onClick={() => importInputRef.current?.click()}
              >
                <Upload className="size-3.5" />
                {t("common:actions.import")}
              </Button>
            </div>
          ) : null}
        </div>
      ) : null}

      <input
        ref={dropFileInputRef}
        type="file"
        accept=".skill.json,.json"
        className="hidden"
        onChange={handleDropFileChange}
      />

      <CreateSkillDialog
        isOpen={dialogOpen}
        onClose={handleDialogClose}
        onCreated={loadSkills}
        editingSkill={editingSkill}
      />

      <AlertDialog
        open={!!deletingSkill}
        onOpenChange={(open) => !open && setDeletingSkill(null)}
      >
        <AlertDialogContent className="max-w-sm">
          <AlertDialogHeader>
            <AlertDialogTitle>{t("view.deleteTitle")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("view.deleteDescription", {
                name: deletingSkill?.name ?? "",
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common:actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              className={buttonVariants({ variant: "destructive" })}
              onClick={handleConfirmDeleteSkill}
            >
              {t("common:actions.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {notification && (
        <div className="fixed bottom-4 right-4 z-50 rounded-lg border border-border bg-background px-4 py-3 text-sm shadow-popover animate-in fade-in slide-in-from-bottom-2">
          {notification}
        </div>
      )}
    </PageShell>
  );
}
