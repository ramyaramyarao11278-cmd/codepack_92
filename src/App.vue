<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { save } from "@tauri-apps/plugin-dialog";
import DropZone from "./components/DropZone.vue";
import FileTree from "./components/FileTree.vue";
import CodePreview from "./components/CodePreview.vue";
import StatusBar from "./components/StatusBar.vue";
import ToastContainer from "./components/ToastContainer.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import { useToast } from "./composables/useToast";
import type { FileNode, ScanResult, ProjectConfig, PackResult, TokenEstimate, ProjectMetadata } from "./types";

const toast = useToast();

const projectPath = ref<string>("");
const projectType = ref<string>("");
const projectMetadata = ref<ProjectMetadata | null>(null);
const fileTree = ref<FileNode | null>(null);
const selectedFilePath = ref<string>("");
const previewContent = ref<string>("");
const isLoading = ref(false);
const isScanning = ref(false);
// CodePack: 预览模式 tab 状态
const previewTab = ref<"file" | "export" | "stats">("file");
const exportPreviewContent = ref<string>("");
const selectedFileSize = ref<number>(0);
// CodePack: 按钮成功状态
const copySuccess = ref(false);
const exportSuccess = ref(false);
// CodePack: 共享折叠状态，传给 FileTree
const collapsedState = reactive<Record<string, boolean>>({});
// CodePack: 刷新中状态
const isRefreshing = ref(false);
// CodePack: Preset 状态
const presets = ref<Record<string, string[]>>({});
const activePreset = ref<string>("");
const showPresetInput = ref(false);
const newPresetName = ref("");
// CodePack: Settings 面板状态
const showSettings = ref(false);

function getAllCheckedFiles(node: FileNode): string[] {
  const result: string[] = [];
  if (!node.is_dir && node.checked) {
    result.push(node.path);
  }
  if (node.children) {
    for (const child of node.children) {
      result.push(...getAllCheckedFiles(child));
    }
  }
  return result;
}

const checkedFiles = computed(() => {
  if (!fileTree.value) return [];
  return getAllCheckedFiles(fileTree.value);
});

const totalTokens = computed(() => Math.round(previewTokenCount.value));

const previewTokenCount = ref(0);
const totalBytes = ref(0);
const isDragging = ref(false);
let unlistenDrop: (() => void) | null = null;

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow();
  unlistenDrop = await appWindow.onDragDropEvent((event) => {
    const type = event.payload.type;
    if (type === "enter" || type === "over") {
      isDragging.value = true;
    } else if (type === "drop") {
      isDragging.value = false;
      const paths = (event.payload as any).paths as string[];
      if (paths && paths.length > 0) {
        onFolderDrop(paths[0]);
      }
    } else if (type === "leave") {
      isDragging.value = false;
    }
  });
});

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop();
});

async function onFolderDrop(path: string) {
  projectPath.value = path;
  isScanning.value = true;
  try {
    const result = await invoke<ScanResult>("scan_directory", { path });
    projectType.value = result.project_type;
    projectMetadata.value = result.metadata;
    fileTree.value = result.tree;
    try {
      const config = await invoke<ProjectConfig | null>("load_project_config", {
        projectPath: path,
      });
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
    previewTab.value = "file";
    exportPreviewContent.value = "";
    // CodePack: 加载 presets
    await loadPresets();
  } catch (e) {
    toast.show({ type: "error", message: `扫描失败: ${e}` });
  } finally {
    isScanning.value = false;
  }
}

function setAllChecked(node: FileNode, checked: boolean) {
  node.checked = checked;
  node.indeterminate = false;
  if (node.children) {
    for (const child of node.children) {
      setAllChecked(child, checked);
    }
  }
}

function restoreCheckedState(node: FileNode, checkedPaths: string[]) {
  const pathSet = new Set(checkedPaths);
  restoreNode(node, pathSet);
}

// CodePack: 收集树中所有文件路径
function collectAllFilePaths(node: FileNode): Set<string> {
  const paths = new Set<string>();
  if (!node.is_dir) paths.add(node.path);
  if (node.children) {
    for (const child of node.children) {
      collectAllFilePaths(child).forEach((p) => paths.add(p));
    }
  }
  return paths;
}

// CodePack: 手动刷新文件树，保留勾选和展开状态
async function refreshFileTree() {
  if (!projectPath.value || !fileTree.value || isRefreshing.value) return;
  isRefreshing.value = true;

  // 记录刷新前状态
  const oldChecked = new Set(getAllCheckedFiles(fileTree.value));
  const oldAllFiles = collectAllFilePaths(fileTree.value);
  // collapsedState 是 reactive 对象，刷新后直接复用

  try {
    const result = await invoke<ScanResult>("scan_directory", { path: projectPath.value });
    projectType.value = result.project_type;
    projectMetadata.value = result.metadata;
    fileTree.value = result.tree;

    // 新文件默认不勾选，已有文件恢复旧勾选状态
    restoreNode(fileTree.value!, oldChecked);

    // 计算新增和移除
    const newAllFiles = collectAllFilePaths(fileTree.value!);
    let added = 0;
    let removed = 0;
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

function restoreNode(node: FileNode, pathSet: Set<string>) {
  if (!node.is_dir) {
    node.checked = pathSet.has(node.path);
  } else {
    if (node.children) {
      for (const child of node.children) {
        restoreNode(child, pathSet);
      }
      updateParentCheck(node);
    }
  }
}

function updateParentCheck(node: FileNode) {
  if (!node.children || node.children.length === 0) return;
  const allChecked = node.children.every((c) => c.checked);
  const someChecked = node.children.some((c) => c.checked || c.indeterminate);
  node.checked = allChecked;
  node.indeterminate = !allChecked && someChecked;
}

// CodePack: 点击文件名时预览，同时获取文件大小
async function onFileSelect(path: string) {
  selectedFilePath.value = path;
  previewTab.value = "file";
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
    await invoke("save_project_config", {
      projectPath: projectPath.value,
      checkedPaths: paths,
    });
  } catch (e) {
    console.error("Save config failed:", e);
  }
}

// CodePack: Preset 管理函数
async function loadPresets() {
  if (!projectPath.value) return;
  try {
    const result = await invoke<Record<string, string[]>>("list_presets", {
      projectPath: projectPath.value,
    });
    presets.value = result;
    activePreset.value = "";
  } catch {
    presets.value = {};
  }
}

async function onSavePreset() {
  const name = newPresetName.value.trim();
  if (!name || !fileTree.value || !projectPath.value) return;
  const paths = getAllCheckedFiles(fileTree.value);
  if (paths.length === 0) {
    toast.show({ type: "info", message: "请先选择文件再保存预设" });
    return;
  }
  try {
    await invoke("save_preset", {
      projectPath: projectPath.value,
      presetName: name,
      checkedPaths: paths,
    });
    presets.value[name] = paths;
    activePreset.value = name;
    showPresetInput.value = false;
    newPresetName.value = "";
    toast.show({ type: "success", message: `预设「${name}」已保存` });
  } catch (e) {
    toast.show({ type: "error", message: `保存预设失败: ${e}` });
  }
}

async function onLoadPreset(name: string) {
  if (!fileTree.value || !presets.value[name]) return;
  const paths = presets.value[name];
  restoreCheckedState(fileTree.value, paths);
  fileTree.value = { ...fileTree.value };
  activePreset.value = name;
  exportPreviewContent.value = "";
  saveConfig();
  toast.show({ type: "success", message: `已切换到预设「${name}」` });
}

async function onDeletePreset(name: string) {
  if (!projectPath.value) return;
  try {
    await invoke("delete_preset", {
      projectPath: projectPath.value,
      presetName: name,
    });
    delete presets.value[name];
    if (activePreset.value === name) activePreset.value = "";
    toast.show({ type: "success", message: `预设「${name}」已删除` });
  } catch (e) {
    toast.show({ type: "error", message: `删除预设失败: ${e}` });
  }
}

// CodePack: 复制到剪贴板 - 完善版
async function onCopyToClipboard() {
  if (!fileTree.value) return;
  const paths = getAllCheckedFiles(fileTree.value);
  if (paths.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
  try {
    const result = await invoke<PackResult>("pack_files", {
      paths,
      projectPath: projectPath.value,
      projectType: projectType.value,
    });
    await invoke("copy_to_clipboard", { content: result.content });
    copySuccess.value = true;
    setTimeout(() => (copySuccess.value = false), 2000);
    const tokenStr = formatTokens(result.estimated_tokens);
    toast.show({
      type: "success",
      message: `已复制 ${result.file_count} 个文件到剪贴板（${tokenStr} tokens）`,
    });
  } catch (e) {
    toast.show({ type: "error", message: `复制失败: ${e}` });
  }
}

// CodePack: 导出为文件 - 使用 save dialog
async function onExportToFile() {
  if (!fileTree.value) return;
  const paths = getAllCheckedFiles(fileTree.value);
  if (paths.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
  try {
    const projectName = projectPath.value
      .replace(/\\/g, "/")
      .split("/")
      .pop() || "project";

    // CodePack: 调用 Tauri save dialog
    const savePath = await save({
      title: "导出代码",
      defaultPath: `${projectPath.value}/../${projectName}_code.txt`,
      filters: [
        { name: "Text", extensions: ["txt"] },
        { name: "Markdown", extensions: ["md"] },
      ],
    });
    if (!savePath) return;

    const resultPath = await invoke<string>("export_to_file", {
      paths,
      projectPath: projectPath.value,
      projectType: projectType.value,
      savePath,
    });
    exportSuccess.value = true;
    setTimeout(() => (exportSuccess.value = false), 2000);
    toast.show({
      type: "success",
      message: `已导出到 ${resultPath}`,
      action: {
        label: "打开目录",
        onClick: () => invoke("open_directory", { path: resultPath }),
      },
      duration: 5000,
    });
  } catch (e) {
    toast.show({ type: "error", message: `导出失败: ${e}` });
  }
}

function formatTokens(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return Math.round(n).toString();
}

// CodePack: 生成导出预览内容
async function refreshExportPreview() {
  if (!fileTree.value) return;
  const paths = getAllCheckedFiles(fileTree.value);
  if (paths.length === 0) {
    exportPreviewContent.value = "";
    return;
  }
  try {
    const result = await invoke<PackResult>("pack_files", {
      paths,
      projectPath: projectPath.value,
      projectType: projectType.value,
    });
    exportPreviewContent.value = result.content;
  } catch {
    exportPreviewContent.value = "";
  }
}

// CodePack: 切换到导出预览 tab 时自动刷新
watch(previewTab, (tab) => {
  if (tab === "export" && !exportPreviewContent.value && checkedFiles.value.length > 0) {
    refreshExportPreview();
  }
  // stats tab 由 StatsPanel 组件内部 watch 处理
});

// CodePack: 右键菜单操作处理
const SOURCE_EXTS = new Set([
  "rs","ts","tsx","js","jsx","vue","svelte","py","kt","kts","java","dart","go",
  "rb","php","swift","c","cpp","h","hpp","cs","m","mm","scala","clj","ex","exs",
  "hs","lua","r","jl","sh","bash","bat","ps1","sql",
]);
const CONFIG_EXTS = new Set([
  "json","yaml","yml","toml","xml","ini","cfg","conf","env","properties",
  "editorconfig","eslintrc","prettierrc","gitignore","dockerfile","makefile",
]);

// CodePack: 展开/折叠全部目录
function setAllCollapsed(node: FileNode, collapsed: boolean) {
  if (node.is_dir) {
    collapsedState[node.path] = collapsed;
    if (node.children) {
      for (const child of node.children) {
        setAllCollapsed(child, collapsed);
      }
    }
  }
}

function selectByFilter(node: FileNode, filter: (n: FileNode) => boolean) {
  if (!node.is_dir) {
    node.checked = filter(node);
    node.indeterminate = false;
  } else if (node.children) {
    for (const child of node.children) {
      selectByFilter(child, filter);
    }
    updateParentCheck(node);
  }
}

function onContextAction(action: string, ext?: string) {
  if (!fileTree.value) return;
  switch (action) {
    case "select-all":
      setAllChecked(fileTree.value, true);
      break;
    case "select-none":
      setAllChecked(fileTree.value, false);
      break;
    case "select-ext":
      if (ext) {
        selectByFilter(fileTree.value, (n) => n.name.endsWith(`.${ext}`));
      }
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
    case "expand-all":
      setAllCollapsed(fileTree.value, false);
      return; // CodePack: 不触发 onTreeChanged，仅更新视觉状态
    case "collapse-all":
      setAllCollapsed(fileTree.value, true);
      return;
    default:
      return;
  }
  onTreeChanged();
}

// CodePack: debounced token + size estimation
let tokenTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  checkedFiles,
  (files) => {
    if (tokenTimer) clearTimeout(tokenTimer);
    exportPreviewContent.value = "";
    if (files.length === 0) {
      previewTokenCount.value = 0;
      totalBytes.value = 0;
      return;
    }
    tokenTimer = setTimeout(async () => {
      try {
        const est = await invoke<TokenEstimate>("estimate_tokens", { paths: files });
        previewTokenCount.value = est.tokens;
        totalBytes.value = est.total_bytes;
      } catch {
        previewTokenCount.value = 0;
        totalBytes.value = 0;
      }
    }, 300);
  },
  { deep: true }
);
</script>

<template>
  <div class="flex flex-col h-screen bg-dark-900 text-dark-100">
    <!-- Header -->
    <header
      class="flex items-center justify-between px-5 py-3 border-b border-dark-700 bg-dark-950 shrink-0"
    >
      <div class="flex items-center gap-3">
        <div class="text-xl font-bold tracking-tight">
          <span class="text-emerald-400">Code</span
          ><span class="text-dark-200">Pack</span>
        </div>
        <span
          v-if="projectType"
          class="px-2 py-0.5 text-xs rounded-full bg-emerald-400/10 text-emerald-400 border border-emerald-400/20"
        >
          {{ projectType }}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <div v-if="projectPath" class="text-xs text-dark-500 truncate max-w-md">
          {{ projectPath }}
        </div>
        <button
          class="text-dark-500 hover:text-emerald-400 transition-colors"
          title="插件管理"
          @click="showSettings = true"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 010-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
    </header>

    <!-- CodePack: 项目元数据信息栏 -->
    <div
      v-if="projectMetadata && (projectMetadata.version || projectMetadata.runtime.length > 0 || projectMetadata.dependencies.length > 0)"
      class="flex items-center gap-4 px-5 py-1.5 border-b border-dark-700 bg-dark-900/80 text-xs text-dark-400 shrink-0 overflow-x-auto"
    >
      <span v-if="projectMetadata.version" class="flex items-center gap-1 shrink-0">
        <span class="text-dark-500">v</span>
        <span class="text-dark-300">{{ projectMetadata.version }}</span>
      </span>
      <span v-if="projectMetadata.runtime.length > 0" class="flex items-center gap-1 shrink-0">
        <span class="text-dark-500">环境</span>
        <span class="text-amber-400/80">{{ projectMetadata.runtime.join(' · ') }}</span>
      </span>
      <span v-if="projectMetadata.entry_point" class="flex items-center gap-1 shrink-0">
        <span class="text-dark-500">入口</span>
        <span class="text-emerald-400/70">{{ projectMetadata.entry_point }}</span>
      </span>
      <span v-if="projectMetadata.dependencies.length > 0" class="flex items-center gap-1 shrink-0">
        <span class="text-dark-500">依赖</span>
        <span class="text-dark-300">{{ projectMetadata.dependencies.length }}个</span>
      </span>
      <span v-if="projectMetadata.requirements.length > 0" class="flex items-center gap-1 truncate">
        <span class="text-dark-500 shrink-0">清单</span>
        <span class="text-sky-400/70 truncate">{{ projectMetadata.requirements.slice(0, 6).join(', ') }}<span v-if="projectMetadata.requirements.length > 6" class="text-dark-500"> +{{ projectMetadata.requirements.length - 6 }}</span></span>
      </span>
    </div>

    <!-- Main Content -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Drop Zone / File Tree (Left Panel) -->
      <div
        class="w-80 border-r border-dark-700 flex flex-col bg-dark-900 shrink-0"
      >
        <DropZone
          v-if="!fileTree"
          :is-scanning="isScanning"
          :is-dragging="isDragging"
          @folder-drop="onFolderDrop"
        />
        <div v-else class="flex flex-col h-full">
          <div
            class="flex items-center justify-between px-3 py-2 border-b border-dark-700 bg-dark-800/50"
          >
            <span class="text-xs text-dark-400 font-medium uppercase tracking-wider"
              >Files</span
            >
            <div class="flex items-center gap-2">
              <!-- CodePack: 刷新按钮 -->
              <button
                class="text-dark-500 hover:text-emerald-400 transition-colors"
                :class="{ 'animate-spin-slow': isRefreshing }"
                :disabled="isRefreshing"
                title="刷新文件树"
                @click="refreshFileTree"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                class="text-xs text-dark-500 hover:text-dark-300 transition-colors"
                @click="
                  fileTree = null;
                  projectPath = '';
                  projectType = '';
                  projectMetadata = null;
                  previewContent = '';
                  exportPreviewContent = '';
                  presets = {};
                  activePreset = '';
                  showPresetInput = false;
                "
              >
                ✕ Close
              </button>
            </div>
          </div>
          <!-- CodePack: Preset 切换栏 -->
          <div
            v-if="Object.keys(presets).length > 0 || showPresetInput"
            class="px-3 py-1.5 border-b border-dark-700 bg-dark-850 flex flex-col gap-1.5"
          >
            <div class="flex items-center gap-1.5 flex-wrap">
              <button
                v-for="name in Object.keys(presets)"
                :key="name"
                class="group flex items-center gap-1 px-2 py-0.5 text-xs rounded-md border transition-colors"
                :class="
                  activePreset === name
                    ? 'bg-emerald-400/15 text-emerald-400 border-emerald-400/30'
                    : 'bg-dark-800 text-dark-400 border-dark-600 hover:text-dark-200 hover:border-dark-500'
                "
                @click="onLoadPreset(name)"
              >
                {{ name }}
                <span
                  class="text-dark-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity ml-0.5"
                  @click.stop="onDeletePreset(name)"
                  title="删除预设"
                >✕</span>
              </button>
              <button
                v-if="!showPresetInput"
                class="px-1.5 py-0.5 text-xs text-dark-500 hover:text-emerald-400 border border-dashed border-dark-600 hover:border-emerald-400/30 rounded-md transition-colors"
                @click="showPresetInput = true"
                title="保存当前选择为预设"
              >+</button>
            </div>
            <div v-if="showPresetInput" class="flex items-center gap-1.5">
              <input
                v-model="newPresetName"
                class="flex-1 px-2 py-1 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
                placeholder="预设名称..."
                @keyup.enter="onSavePreset"
                @keyup.escape="showPresetInput = false; newPresetName = ''"
              />
              <button
                class="px-2 py-1 text-xs bg-emerald-400/15 text-emerald-400 rounded-md hover:bg-emerald-400/25 transition-colors"
                @click="onSavePreset"
              >保存</button>
              <button
                class="px-2 py-1 text-xs text-dark-500 hover:text-dark-300 transition-colors"
                @click="showPresetInput = false; newPresetName = ''"
              >取消</button>
            </div>
          </div>
          <!-- CodePack: 无预设时显示保存入口 -->
          <div
            v-else-if="fileTree"
            class="px-3 py-1 border-b border-dark-700 bg-dark-850"
          >
            <button
              class="text-xs text-dark-500 hover:text-emerald-400 transition-colors"
              @click="showPresetInput = true"
              title="保存当前选择为预设"
            >+ 保存预设</button>
          </div>
          <div class="flex-1 overflow-auto p-2">
            <FileTree
              :node="fileTree"
              :depth="0"
              :selected-path="selectedFilePath"
              :collapsed-state="collapsedState"
              @select="onFileSelect"
              @toggle="onTreeChanged"
              @context-action="onContextAction"
            />
          </div>
        </div>
      </div>

      <!-- Code Preview (Right Panel) -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <CodePreview
          :content="previewContent"
          :file-path="selectedFilePath"
          :file-size="selectedFileSize"
          :is-loading="isLoading"
          :active-tab="previewTab"
          :export-content="exportPreviewContent"
          :checked-count="checkedFiles.length"
          :checked-files="checkedFiles"
          @update:active-tab="previewTab = $event"
        />
      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar
      :file-count="checkedFiles.length"
      :token-count="totalTokens"
      :total-bytes="totalBytes"
      :has-files="!!fileTree"
      :copy-success="copySuccess"
      :export-success="exportSuccess"
      @copy="onCopyToClipboard"
      @export="onExportToFile"
    />

    <!-- CodePack: Toast 通知容器 -->
    <ToastContainer />

    <!-- CodePack: 插件管理面板 -->
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
  </div>
</template>
