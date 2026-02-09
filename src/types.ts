export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileNode[];
  checked: boolean;
  indeterminate?: boolean;
}

export interface ScanResult {
  project_type: string;
  tree: FileNode;
  total_files: number;
}

export interface ProjectConfig {
  project_path: string;
  checked_paths: string[];
  excluded_paths: string[];
  last_opened: string;
}

export interface AppConfig {
  projects: Record<string, ProjectConfig>;
}

// CodePack: pack_files 返回结构
export interface PackResult {
  content: string;
  file_count: number;
  total_bytes: number;
  estimated_tokens: number;
}

// CodePack: estimate_tokens 返回结构
export interface TokenEstimate {
  tokens: number;
  total_bytes: number;
}
