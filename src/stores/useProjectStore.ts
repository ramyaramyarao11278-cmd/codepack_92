import { defineStore } from "pinia";
import { ref, reactive, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import type { FileNode, ScanResult, ProjectConfig, PackResult, TokenEstimate, ProjectMetadata, ExportFormat } from "../types";

export const useProjectStore = defineStore("project", () => {
  const toast = useToast();

  // ─── Core State ──────────────────────────────────────────────
  const projectPath = ref("");
  const projectType = ref("");
  const projectMetadata = ref<ProjectMetadata | null>(null);
  const fileTree = ref<FileNode | null>(null);
  const isScanning = ref(false);
  const isRefreshing = ref(false);

  // ─── Preview State ───────────────────────────────────────────
  const selectedFilePath = ref("");
  const previewContent = ref("");
  const selectedFileSize = ref(0);
  const isLoading = ref(false);
  const exportPreviewContent = ref("");

  // ─── Token Estimation ────────────────────────────────────────
  const previewTokenCount = ref(0);
  const totalBytes = ref(0);

  // ─── Presets ─────────────────────────────────────────────────
  const presets = ref<Record<string, string[]>>({});
  const activePreset = ref("");

  // ─── Shared Collapse State ───────────────────────────────────
  const collapsedState = reactive<Record<string, boolean>>({});

  // ─── Computed ────────────────────────────────────────────────
  function getAllCheckedFiles(node: FileNode): string[] {
    const result: string[] = [];
    if (!node.is_dir && node.checked) result.push(node.path);
    if (node.children) {
      for (const child of node.children) result.push(...getAllCheckedFiles(child));
    }
    return result;
  }

  const checkedFiles = computed(() => {
    if (!fileTree.value) return [];
    return getAllCheckedFiles(fileTree.value);
  });

  const totalTokens = computed(() => Math.round(previewTokenCount.value));

  // ─── Tree Helpers ────────────────────────────────────────────
  function setAllChecked(node: FileNode, checked: boolean) {
    node.checked = checked;
    node.indeterminate = false;
    if (node.children) {
      for (const child of node.children) setAllChecked(child, checked);
    }
  }

  function restoreNode(node: FileNode, pathSet: Set<string>) {
    if (!node.is_dir) {
      node.checked = pathSet.has(node.path);
    } else if (node.children) {
      for (const child of node.children) restoreNode(child, pathSet);
      updateParentCheck(node);
    }
  }

  function updateParentCheck(node: FileNode) {
    if (!node.children || node.children.length === 0) return;
    const allChecked = node.children.every((c) => c.checked);
    const someChecked = node.children.some((c) => c.checked || c.indeterminate);
    node.checked = allChecked;
    node.indeterminate = !allChecked && someChecked;
  }

  function restoreCheckedState(node: FileNode, checkedPaths: string[]) {
    restoreNode(node, new Set(checkedPaths));
  }

  function collectAllFilePaths(node: FileNode): Set<string> {
    const paths = new Set<string>();
    if (!node.is_dir) paths.add(node.path);
    if (node.children) {
      for (const child of node.children) collectAllFilePaths(child).forEach((p) => paths.add(p));
    }
    return paths;
  }

  // ─── Actions ─────────────────────────────────────────────────
  async function scanDirectory(path: string) {
    projectPath.value = path;
    isScanning.value = true;
    try {
      const result = await invoke<ScanResult>("scan_directory", { path });
      projectType.value = result.project_type;
      projectMetadata.value = result.metadata;
      fileTree.value = result.tree;
      try {
        const config = await invoke<ProjectConfig | null>("load_project_config", { projectPath: path });
        if (config && config.checked_paths.length > 0) {
          restoreCheckedState(fileTree.value!, config.checked_paths);
        } else {
          setAllChecked(fileTree.value!, true);
        }
      } catch {
        setAllChecked(fileTree.value!, true);
      }
      selectedFilePath.value = "";
      previewContent.value = "";
      exportPreviewContent.value = "";
      await loadPresets();
    } catch (e) {
      toast.show({ type: "error", message: `扫描失败: ${e}` });
    } finally {
      isScanning.value = false;
    }
  }

  async function refreshFileTree() {
    if (!projectPath.value || !fileTree.value || isRefreshing.value) return;
    isRefreshing.value = true;
    const oldChecked = new Set(getAllCheckedFiles(fileTree.value));
    const oldAllFiles = collectAllFilePaths(fileTree.value);
    try {
      const result = await invoke<ScanResult>("scan_directory", { path: projectPath.value });
      projectType.value = result.project_type;
      projectMetadata.value = result.metadata;
      fileTree.value = result.tree;
      restoreNode(fileTree.value!, oldChecked);
      const newAllFiles = collectAllFilePaths(fileTree.value!);
      let added = 0, removed = 0;
      newAllFiles.forEach((p) => { if (!oldAllFiles.has(p)) added++; });
      oldAllFiles.forEach((p) => { if (!newAllFiles.has(p)) removed++; });
      exportPreviewContent.value = "";
      saveConfig();
      if (added > 0 || removed > 0) {
        const parts: string[] = [];
        if (added > 0) parts.push(`新增 ${added} 个`);
        if (removed > 0) parts.push(`移除 ${removed} 个`);
        toast.show({ type: "success", message: `已刷新，${parts.join("，")}` });
      } else {
        toast.show({ type: "success", message: `已刷新，共 ${newAllFiles.size} 个文件` });
      }
    } catch (e) {
      toast.show({ type: "error", message: `刷新失败: ${e}` });
    } finally {
      isRefreshing.value = false;
    }
  }

  async function selectFile(path: string) {
    selectedFilePath.value = path;
    isLoading.value = true;
    try {
      const [content, size] = await Promise.all([
        invoke<string>("read_file_content", { path }),
        invoke<number>("get_file_size", { path }),
      ]);
      previewContent.value = content;
      selectedFileSize.value = size;
    } catch (e) {
      previewContent.value = `// Error reading file: ${e}`;
      selectedFileSize.value = 0;
    } finally {
      isLoading.value = false;
    }
  }

  function onTreeChanged() {
    fileTree.value = { ...fileTree.value! };
    exportPreviewContent.value = "";
    saveConfig();
  }

  async function saveConfig() {
    if (!projectPath.value || !fileTree.value) return;
    const paths = getAllCheckedFiles(fileTree.value);
    try {
      await invoke("save_project_config", { projectPath: projectPath.value, checkedPaths: paths });
    } catch (e) {
      console.error("Save config failed:", e);
    }
  }

  // ─── Presets ─────────────────────────────────────────────────
  async function loadPresets() {
    if (!projectPath.value) return;
    try {
      presets.value = await invoke<Record<string, string[]>>("list_presets", { projectPath: projectPath.value });
      activePreset.value = "";
    } catch {
      presets.value = {};
    }
  }

  async function savePreset(name: string) {
    if (!fileTree.value || !projectPath.value) return;
    const paths = getAllCheckedFiles(fileTree.value);
    if (paths.length === 0) {
      toast.show({ type: "info", message: "请先选择文件再保存预设" });
      return;
    }
    try {
      await invoke("save_preset", { projectPath: projectPath.value, presetName: name, checkedPaths: paths });
      presets.value[name] = paths;
      activePreset.value = name;
      toast.show({ type: "success", message: `预设「${name}」已保存` });
    } catch (e) {
      toast.show({ type: "error", message: `保存预设失败: ${e}` });
    }
  }

  async function loadPreset(name: string) {
    if (!fileTree.value || !presets.value[name]) return;
    restoreCheckedState(fileTree.value, presets.value[name]);
    fileTree.value = { ...fileTree.value };
    activePreset.value = name;
    exportPreviewContent.value = "";
    saveConfig();
    toast.show({ type: "success", message: `已切换到预设「${name}」` });
  }

  async function deletePreset(name: string) {
    if (!projectPath.value) return;
    try {
      await invoke("delete_preset", { projectPath: projectPath.value, presetName: name });
      delete presets.value[name];
      if (activePreset.value === name) activePreset.value = "";
      toast.show({ type: "success", message: `预设「${name}」已删除` });
    } catch (e) {
      toast.show({ type: "error", message: `删除预设失败: ${e}` });
    }
  }

  // ─── Export ──────────────────────────────────────────────────
  async function refreshExportPreview(format: ExportFormat = "plain") {
    if (!fileTree.value) return;
    const paths = getAllCheckedFiles(fileTree.value);
    if (paths.length === 0) { exportPreviewContent.value = ""; return; }
    try {
      const result = await invoke<PackResult>("pack_files", {
        paths, projectPath: projectPath.value, projectType: projectType.value, format,
      });
      exportPreviewContent.value = result.content;
    } catch {
      exportPreviewContent.value = "";
    }
  }

  async function updateTokenEstimate(files: string[]) {
    if (files.length === 0) {
      previewTokenCount.value = 0;
      totalBytes.value = 0;
      return;
    }
    try {
      const est = await invoke<TokenEstimate>("estimate_tokens", { paths: files });
      previewTokenCount.value = est.tokens;
      totalBytes.value = est.total_bytes;
    } catch {
      previewTokenCount.value = 0;
      totalBytes.value = 0;
    }
  }

  // ─── Context Actions ────────────────────────────────────────
  const SOURCE_EXTS = new Set([
    "rs","ts","tsx","js","jsx","vue","svelte","py","kt","kts","java","dart","go",
    "rb","php","swift","c","cpp","h","hpp","cs","m","mm","scala","clj","ex","exs",
    "hs","lua","r","jl","sh","bash","bat","ps1","sql",
  ]);
  const CONFIG_EXTS = new Set([
    "json","yaml","yml","toml","xml","ini","cfg","conf","env","properties",
    "editorconfig","eslintrc","prettierrc","gitignore","dockerfile","makefile",
  ]);

  function setAllCollapsed(node: FileNode, collapsed: boolean) {
    if (node.is_dir) {
      collapsedState[node.path] = collapsed;
      if (node.children) {
        for (const child of node.children) setAllCollapsed(child, collapsed);
      }
    }
  }

  function selectByFilter(node: FileNode, filter: (n: FileNode) => boolean) {
    if (!node.is_dir) {
      node.checked = filter(node);
      node.indeterminate = false;
    } else if (node.children) {
      for (const child of node.children) selectByFilter(child, filter);
      updateParentCheck(node);
    }
  }

  function contextAction(action: string, ext?: string) {
    if (!fileTree.value) return;
    switch (action) {
      case "select-all": setAllChecked(fileTree.value, true); break;
      case "select-none": setAllChecked(fileTree.value, false); break;
      case "select-ext":
        if (ext) selectByFilter(fileTree.value, (n) => n.name.endsWith(`.${ext}`));
        break;
      case "select-source":
        selectByFilter(fileTree.value, (n) => {
          const e = n.name.split(".").pop()?.toLowerCase() || "";
          return SOURCE_EXTS.has(e);
        });
        break;
      case "select-config":
        selectByFilter(fileTree.value, (n) => {
          const e = n.name.split(".").pop()?.toLowerCase() || "";
          return CONFIG_EXTS.has(e);
        });
        break;
      case "expand-all": setAllCollapsed(fileTree.value, false); return;
      case "collapse-all": setAllCollapsed(fileTree.value, true); return;
      default: return;
    }
    onTreeChanged();
  }

  function closeProject() {
    fileTree.value = null;
    projectPath.value = "";
    projectType.value = "";
    projectMetadata.value = null;
    previewContent.value = "";
    exportPreviewContent.value = "";
    presets.value = {};
    activePreset.value = "";
  }

  return {
    // State
    projectPath, projectType, projectMetadata, fileTree,
    isScanning, isRefreshing,
    selectedFilePath, previewContent, selectedFileSize, isLoading,
    exportPreviewContent,
    previewTokenCount, totalBytes,
    presets, activePreset,
    collapsedState,
    // Computed
    checkedFiles, totalTokens,
    // Actions
    scanDirectory, refreshFileTree, selectFile, onTreeChanged, saveConfig,
    loadPresets, savePreset, loadPreset, deletePreset,
    refreshExportPreview, updateTokenEstimate,
    contextAction, closeProject,
    setAllChecked, restoreCheckedState,
  };
});
