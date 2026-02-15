export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileNode[];
  checked: boolean;
  indeterminate?: boolean;
}

export interface ProjectMetadata {
  name: string;
  project_type: string;
  version: string | null;
  description: string | null;
  dependencies: string[];
  dev_dependencies: string[];
  entry_point: string | null;
  runtime: string[];
  requirements: string[];
}

export interface ScanResult {
  project_type: string;
  tree: FileNode;
  total_files: number;
  metadata: ProjectMetadata;
}

export interface ProjectConfig {
  project_path: string;
  checked_paths: string[];
  excluded_paths: string[];
  last_opened: string;
  presets: Record<string, string[]>;
  pinned: boolean;
}

export interface AppConfig {
  projects: Record<string, ProjectConfig>;
}

export interface PluginDef {
  name: string;
  version: string;
  detect_files: string[];
  detect_dirs: string[];
  exclude_dirs: string[];
  source_extensions: string[];
}

export interface LangStat {
  language: string;
  extension: string;
  file_count: number;
  line_count: number;
  byte_count: number;
}

export interface ProjectStats {
  total_files: number;
  total_lines: number;
  total_bytes: number;
  languages: LangStat[];
}

export type ExportFormat = "plain" | "markdown" | "xml";

export interface ChangedFile {
  path: string;
  status: string;
}

export interface GitStatus {
  is_repo: boolean;
  branch: string;
  changed_files: ChangedFile[];
}

export interface SkippedFile {
  path: string;
  reason: string;
  size_bytes: number;
}

// CodePack: pack_files 返回结构
export interface PackResult {
  content: string;
  file_count: number;
  total_bytes: number;
  estimated_tokens: number;
  skipped_files: SkippedFile[];
}

// CodePack: 扫描进度事件
export interface ScanProgress {
  phase: string;
  files_found: number;
  message: string;
}

// CodePack: estimate_tokens 返回结构
export interface TokenEstimate {
  tokens: number;
  total_bytes: number;
}
