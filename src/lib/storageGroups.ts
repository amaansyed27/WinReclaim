import type { DriveInfo, Finding } from "../types";

export type StorageGroupId =
  | "system"
  | "browsers"
  | "developer"
  | "android"
  | "media"
  | "projects"
  | "applications"
  | "user-data"
  | "other";

export interface StorageGroupDefinition {
  id: StorageGroupId;
  label: string;
  description: string;
}

export const storageGroups: StorageGroupDefinition[] = [
  { id: "system", label: "Windows and system", description: "Windows, shared system data, updates and temporary storage." },
  { id: "browsers", label: "Browsers and web runtimes", description: "Browser profiles, extensions, web views and browser caches." },
  { id: "developer", label: "Developer tools and package managers", description: "Editors, SDK support files, package caches and build tooling." },
  { id: "android", label: "Android development", description: "SDKs, emulator devices, system images and Android projects." },
  { id: "media", label: "Media and recordings", description: "Videos, captures, recordings and generated media." },
  { id: "projects", label: "Projects and downloads", description: "Source trees, downloaded repositories and project dependencies." },
  { id: "applications", label: "Installed applications", description: "Installed programs and application-managed data." },
  { id: "user-data", label: "User data", description: "Documents, synced files and other personal storage." },
  { id: "other", label: "Other large locations", description: "Large folders that do not yet match a more specific deterministic category." }
];

export function groupForFinding(finding: Finding): StorageGroupId {
  const path = finding.path.toLocaleLowerCase();
  const rule = finding.ruleId.toLocaleLowerCase();
  const category = finding.category.toLocaleLowerCase();

  if (
    rule.startsWith("windows.") ||
    rule.startsWith("system_drive.") ||
    path.includes("\\windows\\") ||
    path.includes("\\programdata\\") ||
    category.includes("windows")
  ) return "system";

  if (
    path.includes("\\google\\chrome") ||
    path.includes("\\microsoft\\edge") ||
    path.includes("\\mozilla\\firefox") ||
    path.includes("\\firefox\\") ||
    path.includes("\\ebwebview\\") ||
    path.includes("\\webview") ||
    category.includes("browser")
  ) return "browsers";

  if (
    path.includes("\\.android\\") ||
    path.includes("\\android\\sdk") ||
    path.includes("androidstudioprojects") ||
    category.includes("android")
  ) return "android";

  if (
    path.includes("\\videos\\") ||
    path.includes("screen recordings") ||
    path.includes("recordings") ||
    path.includes("\\captures\\") ||
    category.includes("media")
  ) return "media";

  if (
    rule.startsWith("project.") ||
    path.includes("\\downloads\\") ||
    path.includes("\\documents\\") ||
    path.includes("\\desktop\\") ||
    path.includes("\\source\\") ||
    path.includes("\\repos\\") ||
    path.includes("\\projects\\") ||
    path.includes("\\develop\\") ||
    path.includes("\\workspace\\")
  ) return "projects";

  if (
    rule.startsWith("npm.") ||
    rule.startsWith("pip.") ||
    rule.startsWith("uv.") ||
    rule.startsWith("cargo.") ||
    rule.startsWith("gradle.") ||
    rule.startsWith("playwright.") ||
    path.includes("\\.cargo\\") ||
    path.includes("\\.codex\\") ||
    path.includes("\\.vscode\\") ||
    path.includes("\\code\\") ||
    path.includes("\\npm\\") ||
    path.includes("\\uv\\") ||
    path.includes("\\ms-playwright") ||
    path.includes("\\flutter\\") ||
    category.includes("developer") ||
    category.includes("package") ||
    category.includes("build")
  ) return "developer";

  if (
    path.includes("\\program files\\") ||
    path.includes("\\program files (x86)\\") ||
    path.includes("\\appdata\\local\\programs\\") ||
    category.includes("installed") ||
    category.includes("application")
  ) return "applications";

  if (
    path.includes("\\onedrive\\") ||
    path.includes("\\documents\\") ||
    path.includes("\\pictures\\") ||
    path.includes("\\music\\")
  ) return "user-data";

  return "other";
}

export function driveForFinding(finding: Finding, drives: DriveInfo[]): DriveInfo | undefined {
  const path = normalizePath(finding.path);
  return drives
    .slice()
    .sort((a, b) => b.root.length - a.root.length)
    .find((drive) => path.startsWith(normalizePath(drive.root)));
}

export function friendlyFindingName(finding: Finding): string {
  const original = finding.displayName.replace(/\s*\(unclassified\)$/i, "").trim();
  const parts = finding.path.split(/[\\/]+/).filter(Boolean);
  const leaf = parts.at(-1) ?? original;
  const parent = parts.at(-2) ?? "";
  const grandparent = parts.at(-3) ?? "";
  const lowerPath = finding.path.toLocaleLowerCase();

  if (lowerPath.includes("\\.codex\\sessions\\")) {
    return leaf === "sessions" ? "Codex sessions" : `Codex sessions — ${leaf}`;
  }
  if (leaf.toLocaleLowerCase() === "user data" && parent) {
    return `${cleanSegment(parent)} user data`;
  }
  if (leaf.toLocaleLowerCase() === "packages" && lowerPath.includes("\\appdata\\local\\packages")) {
    return "Windows app packages";
  }
  if (/^\d{4}$/.test(leaf) && parent) {
    return `${cleanSegment(parent)} — ${leaf}`;
  }
  if (leaf.toLocaleLowerCase() === "bin" && parent) {
    return `${cleanSegment(parent)} binaries`;
  }
  if (/^[a-f0-9]{32,}$/i.test(leaf) && parent) {
    return `${cleanSegment(parent)} data — ${leaf.slice(0, 8)}…`;
  }
  if (original.toLocaleLowerCase() === leaf.toLocaleLowerCase() && grandparent) {
    return original;
  }
  return original || cleanSegment(leaf);
}

export function shortPath(path: string, drive?: DriveInfo): string {
  if (!drive) return path;
  const normalizedDrive = normalizePath(drive.root);
  const normalizedPath = normalizePath(path);
  if (!normalizedPath.startsWith(normalizedDrive)) return path;
  const relative = path.slice(drive.root.length).replace(/^[\\/]+/, "");
  return relative || drive.root;
}

function normalizePath(path: string): string {
  return path.replace(/\//g, "\\").toLocaleLowerCase();
}

function cleanSegment(value: string): string {
  return value
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (character) => character.toUpperCase());
}
