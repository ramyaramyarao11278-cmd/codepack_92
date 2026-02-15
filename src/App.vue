<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { save } from "@tauri-apps/plugin-dialog";
import DropZone from "./components/DropZone.vue";
import FileTree from "./components/FileTree.vue";
import CodePreview from "./components/CodePreview.vue";
import StatusBar from "./components/StatusBar.vue";
import ToastContainer from "./components/ToastContainer.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import MetadataBar from "./components/MetadataBar.vue";
import PresetBar from "./components/PresetBar.vue";
import ExcludeRulesEditor from "./components/ExcludeRulesEditor.vue";
import ReviewPromptBar from "./components/ReviewPromptBar.vue";
import { useToast } from "./composables/useToast";
import { useProjectStore } from "./stores/useProjectStore";
import { useUIStore } from "./stores/useUIStore";
import type { PackResult, SecretMatch } from "./types";

const toast = useToast();
const project = useProjectStore();
const ui = useUIStore();

let unlistenDrop: (() => void) | null = null;

// CodePack: 安全拦截对话框状态
const securityDialog = ref({ show: false, action: '' as 'copy' | 'export' | '' });

async function doMaskedExport(action: 'copy' | 'export') {
  // Re-pack with masking: scan all checked files, mask content, then export
  const paths = project.checkedFiles;
  const result = await invoke<PackResult>("pack_files", {
    paths,
    projectPath: project.projectPath,
    projectType: project.projectType,
    format: ui.exportFormat,
    maxFileBytes: ui.maxFileKB * 1024,
  });
  // Mask all secrets in the packed content
  let content = result.content;
  for (const matches of Object.values(project.secretsMap)) {
    for (const m of matches) {
      if (content.includes(m.match_content)) {
        const prefix = m.match_content.substring(0, 3);
        content = content.replaceAll(m.match_content, prefix + "******");
      }
    }
  }
  if (action === 'copy') {
    await invoke("copy_to_clipboard", { content });
    ui.copySuccess = true;
    setTimeout(() => (ui.copySuccess = false), 2000);
    toast.show({ type: "success", message: `已脱敏并复制 ${result.file_count} 个文件到剪贴板` });
  } else {
    const projectName = project.projectPath.replace(/\\/g, "/").split("/").pop() || "project";
    const extMap = { plain: "txt", markdown: "md", xml: "xml" } as const;
    const defaultExt = extMap[ui.exportFormat];
    const savePath = await save({
      title: "导出代码（已脱敏）",
      defaultPath: `${project.projectPath}/../${projectName}_code.${defaultExt}`,
      filters: [
        { name: "Text", extensions: ["txt"] },
        { name: "Markdown", extensions: ["md"] },
        { name: "XML", extensions: ["xml"] },
      ],
    });
    if (!savePath) return;
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");
    await writeTextFile(savePath, content);
    ui.exportSuccess = true;
    setTimeout(() => (ui.exportSuccess = false), 2000);
    toast.show({ type: "success", message: `已脱敏并导出到 ${savePath}` });
  }
}

function onSecurityDialogChoice(choice: 'mask' | 'force' | 'cancel') {
  const action = securityDialog.value.action;
  securityDialog.value = { show: false, action: '' };
  if (choice === 'cancel') return;
  if (choice === 'mask') {
    doMaskedExport(action as 'copy' | 'export');
  } else {
    // Force: proceed without masking
    if (action === 'copy') doCopyToClipboard();
    else doExportToFile();
  }
}

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow();
  unlistenDrop = await appWindow.onDragDropEvent((event) => {
    const type = event.payload.type;
    if (type === "enter" || type === "over") {
      ui.isDragging = true;
    } else if (type === "drop") {
      ui.isDragging = false;
      const paths = (event.payload as any).paths as string[];
      if (paths && paths.length > 0) {
        onFolderDrop(paths[0]);
      }
    } else if (type === "leave") {
      ui.isDragging = false;
    }
  });
});

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop();
});

async function onFolderDrop(path: string) {
  ui.previewTab = "file";
  await project.scanDirectory(path);
}

function formatTokens(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return Math.round(n).toString();
}

// CodePack: 安全检查 + 复制到剪贴板
function onCopyToClipboard() {
  if (!project.fileTree) return;
  if (project.checkedFiles.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
  if (project.totalSecretCount > 0) {
    securityDialog.value = { show: true, action: 'copy' };
    return;
  }
  doCopyToClipboard();
}

async function doCopyToClipboard() {
  const paths = project.checkedFiles;
  try {
    if (project.exportPreviewContent) {
      await invoke("copy_to_clipboard", { content: project.exportPreviewContent });
      ui.copySuccess = true;
      setTimeout(() => (ui.copySuccess = false), 2000);
      toast.show({ type: "success", message: "已复制编辑后的内容到剪贴板" });
      return;
    }
    const useExtended = ui.includeDiff || !!project.activeInstruction;
    const packCmd = useExtended ? "pack_files_extended" : "pack_files";
    const packArgs: Record<string, unknown> = {
      paths,
      projectPath: project.projectPath,
      projectType: project.projectType,
      format: ui.exportFormat,
      maxFileBytes: ui.maxFileKB * 1024,
    };
    if (ui.includeDiff) packArgs.includeDiff = true;
    if (project.activeInstruction) packArgs.instruction = project.activeInstruction;
    const result = await invoke<PackResult>(packCmd, packArgs);
    await invoke("copy_to_clipboard", { content: result.content });
    ui.copySuccess = true;
    setTimeout(() => (ui.copySuccess = false), 2000);
    if (result.skipped_files && result.skipped_files.length > 0) {
      const reasons = result.skipped_files.map((f: { path: string; reason: string }) => f.reason);
      const binary = reasons.filter((r: string) => r.includes("binary")).length;
      const oversized = reasons.filter((r: string) => r.includes("limit")).length;
      const parts: string[] = [];
      if (oversized > 0) parts.push(`${oversized} 个超大文件`);
      if (binary > 0) parts.push(`${binary} 个二进制文件`);
      toast.show({
        type: "info",
        message: `跳过了 ${parts.join("、")}`,
        duration: 4000,
      });
    }
    const diffNote = ui.includeDiff ? " +Diff" : "";
    toast.show({
      type: "success",
      message: `已复制 ${result.file_count} 个文件到剪贴板（${formatTokens(result.estimated_tokens)} tokens${diffNote}）`,
    });
  } catch (e) {
    toast.show({ type: "error", message: `复制失败: ${e}` });
  }
}

// CodePack: 安全检查 + 导出为文件
function onExportToFile() {
  if (!project.fileTree) return;
  if (project.checkedFiles.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
  if (project.totalSecretCount > 0) {
    securityDialog.value = { show: true, action: 'export' };
    return;
  }
  doExportToFile();
}

async function doExportToFile() {
  const paths = project.checkedFiles;
  try {
    const projectName = project.projectPath.replace(/\\/g, "/").split("/").pop() || "project";
    const extMap = { plain: "txt", markdown: "md", xml: "xml" } as const;
    const defaultExt = extMap[ui.exportFormat];
    const savePath = await save({
      title: "导出代码",
      defaultPath: `${project.projectPath}/../${projectName}_code.${defaultExt}`,
      filters: [
        { name: "Text", extensions: ["txt"] },
        { name: "Markdown", extensions: ["md"] },
        { name: "XML", extensions: ["xml"] },
      ],
    });
    if (!savePath) return;

    // If user has edited the export preview, write that directly
    if (project.exportPreviewContent) {
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(savePath, project.exportPreviewContent);
      ui.exportSuccess = true;
      setTimeout(() => (ui.exportSuccess = false), 2000);
      toast.show({
        type: "success",
        message: `已导出编辑后的内容到 ${savePath}`,
        action: { label: "打开目录", onClick: () => invoke("open_directory", { path: savePath }) },
        duration: 5000,
      });
      return;
    }

    const useExtended = ui.includeDiff || !!project.activeInstruction;
    if (useExtended) {
      const extArgs: Record<string, unknown> = {
        paths,
        projectPath: project.projectPath,
        projectType: project.projectType,
        format: ui.exportFormat,
        maxFileBytes: ui.maxFileKB * 1024,
      };
      if (ui.includeDiff) extArgs.includeDiff = true;
      if (project.activeInstruction) extArgs.instruction = project.activeInstruction;
      const result = await invoke<PackResult>("pack_files_extended", extArgs);
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(savePath, result.content);
    } else {
      await invoke<string>("export_to_file", {
        paths,
        projectPath: project.projectPath,
        projectType: project.projectType,
        savePath,
        format: ui.exportFormat,
        maxFileBytes: ui.maxFileKB * 1024,
      });
    }
    ui.exportSuccess = true;
    setTimeout(() => (ui.exportSuccess = false), 2000);
    toast.show({
      type: "success",
      message: `已导出到 ${savePath}`,
      action: {
        label: "打开目录",
        onClick: () => invoke("open_directory", { path: savePath }),
      },
      duration: 5000,
    });
  } catch (e) {
    toast.show({ type: "error", message: `导出失败: ${e}` });
  }
}

function onFileSelect(path: string) {
  ui.previewTab = "file";
  project.selectFile(path);
}

function onSavePreset() {
  const name = ui.newPresetName.trim();
  if (!name) return;
  project.savePreset(name);
  ui.resetPresetUI();
}

function onCloseProject() {
  project.closeProject();
  ui.resetPresetUI();
  ui.changedOnly = false;
}

async function onStartReview() {
  if (!project.fileTree || project.checkedFiles.length === 0) {
    toast.show({ type: "info", message: "请先选择要 Review 的文件" });
    return;
  }
  // Pack the content first, then send to AI
  const useExtended = ui.includeDiff || !!project.activeInstruction;
  const packCmd = useExtended ? "pack_files_extended" : "pack_files";
  const packArgs: Record<string, unknown> = {
    paths: project.checkedFiles,
    projectPath: project.projectPath,
    projectType: project.projectType,
    format: ui.exportFormat,
    maxFileBytes: ui.maxFileKB * 1024,
  };
  if (ui.includeDiff) packArgs.includeDiff = true;
  if (project.activeInstruction) packArgs.instruction = project.activeInstruction;
  try {
    const result = await invoke<PackResult>(packCmd, packArgs);
    ui.previewTab = "review";
    await project.startReview(result.content);
  } catch (e) {
    toast.show({ type: "error", message: `打包失败: ${e}` });
  }
}

function toggleChangedOnly() {
  ui.changedOnly = !ui.changedOnly;
  if (ui.changedOnly) {
    project.contextAction('select-git-changed');
  } else {
    project.setAllChecked(project.fileTree!, true);
    project.onTreeChanged();
  }
}

// CodePack: 当前预览文件的敏感信息
const currentFileSecrets = computed<SecretMatch[]>(() => {
  if (!project.selectedFilePath || !project.secretsMap) return [];
  // secretsMap uses relative paths, selectedFilePath is absolute
  for (const [relPath, matches] of Object.entries(project.secretsMap)) {
    if (project.selectedFilePath.replace(/\\/g, "/").endsWith(relPath)) {
      return matches;
    }
  }
  return [];
});

async function onMaskSecrets() {
  if (!project.selectedFilePath) return;
  const masked = await project.maskFileSecrets(project.selectedFilePath);
  if (masked !== null) {
    project.previewContent = masked;
    toast.show({ type: "success", message: "已脱敏当前文件预览内容" });
  }
}

function onExcludeFile() {
  if (!project.fileTree || !project.selectedFilePath) return;
  // Uncheck the current file in the tree
  function uncheckFile(node: import("./types").FileNode, path: string) {
    if (!node.is_dir && node.path === path) {
      node.checked = false;
      return true;
    }
    if (node.children) {
      for (const child of node.children) {
        if (uncheckFile(child, path)) return true;
      }
    }
    return false;
  }
  uncheckFile(project.fileTree, project.selectedFilePath);
  project.onTreeChanged();
  toast.show({ type: "info", message: "已从导出中排除此文件" });
}

// CodePack: 切换到导出预览 tab 或格式变化时自动刷新
watch([() => ui.previewTab, () => ui.exportFormat], ([tab, _fmt]) => {
  if (tab === "export" && project.checkedFiles.length > 0) {
    project.refreshExportPreview(ui.exportFormat, ui.maxFileKB * 1024);
  }
});

// CodePack: debounced token estimation
let tokenTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => project.checkedFiles,
  (files) => {
    if (tokenTimer) clearTimeout(tokenTimer);
    project.exportPreviewContent = "";
    if (files.length === 0) {
      project.previewTokenCount = 0;
      project.totalBytes = 0;
      return;
    }
    tokenTimer = setTimeout(() => project.updateTokenEstimate(files), 300);
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
          v-if="project.projectType"
          class="px-2 py-0.5 text-xs rounded-full bg-emerald-400/10 text-emerald-400 border border-emerald-400/20"
        >
          {{ project.projectType }}
        </span>
      </div>
      <div class="flex items-center gap-3">
        <span
          v-if="project.gitStatus?.is_repo"
          class="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-emerald-400/10 text-emerald-400 border border-emerald-400/20"
          :title="`Branch: ${project.gitStatus.branch} | ${project.gitStatus.changed_files.length} changed`"
        >
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 16 16"><path d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.493 2.493 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z"/></svg>
          {{ project.gitStatus.branch }}
          <span v-if="project.gitStatus.changed_files.length > 0" class="text-yellow-400">+{{ project.gitStatus.changed_files.length }}</span>
        </span>
        <div v-if="project.projectPath" class="text-xs text-dark-500 truncate max-w-md">
          {{ project.projectPath }}
        </div>
        <button
          class="text-dark-500 hover:text-emerald-400 transition-colors"
          title="插件管理"
          @click="ui.showSettings = true"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 010-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
    </header>

    <!-- CodePack: 项目元数据信息栏 -->
    <MetadataBar :metadata="project.projectMetadata" />

    <!-- Main Content -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Drop Zone / File Tree (Left Panel) -->
      <div
        class="w-80 border-r border-dark-700 flex flex-col bg-dark-900 shrink-0"
      >
        <DropZone
          v-if="!project.fileTree"
          :is-scanning="project.isScanning"
          :is-dragging="ui.isDragging"
          :scan-message="project.scanProgress?.message"
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
              <button
                v-if="project.gitStatus?.is_repo"
                class="flex items-center gap-1 px-1.5 py-0.5 text-xs rounded transition-colors"
                :class="ui.changedOnly
                  ? 'bg-yellow-400/15 text-yellow-400 border border-yellow-400/30'
                  : 'text-dark-500 hover:text-yellow-400'"
                :title="ui.changedOnly ? '显示全部文件' : '只选 Git 变更文件'"
                @click="toggleChangedOnly"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
                </svg>
                <span v-if="ui.changedOnly">Changed</span>
                <span v-if="project.gitStatus?.changed_files.length" class="text-yellow-400/70">{{ project.gitStatus.changed_files.filter(f => f.status !== 'deleted').length }}</span>
              </button>
              <button
                class="text-dark-500 hover:text-yellow-400 transition-colors"
                title="排除规则"
                @click="ui.showExcludeEditor = true"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 3c2.755 0 5.455.232 8.083.678.533.09.917.556.917 1.096v1.044a2.25 2.25 0 01-.659 1.591l-5.432 5.432a2.25 2.25 0 00-.659 1.591v2.927a2.25 2.25 0 01-1.244 2.013L9.75 21v-6.568a2.25 2.25 0 00-.659-1.591L3.659 7.409A2.25 2.25 0 013 5.818V4.774c0-.54.384-1.006.917-1.096A48.32 48.32 0 0112 3z" />
                </svg>
              </button>
              <button
                class="text-dark-500 hover:text-emerald-400 transition-colors"
                :class="{ 'animate-spin-slow': project.isRefreshing }"
                :disabled="project.isRefreshing"
                title="刷新文件树"
                @click="project.refreshFileTree()"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                class="text-xs text-dark-500 hover:text-dark-300 transition-colors"
                @click="onCloseProject"
              >
                ✕ Close
              </button>
            </div>
          </div>
          <!-- CodePack: Preset 切换栏 -->
          <PresetBar
            :presets="project.presets"
            :active-preset="project.activePreset"
            :show-input="ui.showPresetInput"
            :new-name="ui.newPresetName"
            :has-tree="!!project.fileTree"
            @load="project.loadPreset($event)"
            @delete="project.deletePreset($event)"
            @show-input="ui.showPresetInput = true"
            @hide-input="ui.resetPresetUI()"
            @update:new-name="ui.newPresetName = $event"
            @save="onSavePreset"
          />
          <!-- CodePack: 文件搜索过滤 -->
          <div class="px-2 py-1.5 border-b border-dark-700 bg-dark-850">
            <div class="relative">
              <svg class="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-dark-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <input
                v-model="ui.treeFilter"
                class="w-full pl-7 pr-6 py-1 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
                placeholder="搜索文件..."
              />
              <button
                v-if="ui.treeFilter"
                class="absolute right-1.5 top-1/2 -translate-y-1/2 text-dark-500 hover:text-dark-300 text-xs"
                @click="ui.treeFilter = ''"
              >✕</button>
            </div>
          </div>
          <div class="flex-1 overflow-auto p-2">
            <FileTree
              :node="project.fileTree"
              :depth="0"
              :selected-path="project.selectedFilePath"
              :collapsed-state="project.collapsedState"
              :filter-text="ui.treeFilter"
              :risky-files="project.riskyFiles"
              @select="onFileSelect"
              @toggle="project.onTreeChanged()"
              @context-action="project.contextAction"
            />
          </div>
        </div>
      </div>

      <!-- Code Preview (Right Panel) -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <!-- CodePack: Review 角色预设选择条 -->
        <ReviewPromptBar
          :prompts="project.reviewPrompts"
          :active-prompt="project.activeReviewPrompt"
          :has-files="!!project.fileTree"
          @select="project.activeReviewPrompt = $event"
          @deselect="project.activeReviewPrompt = ''"
          @save="project.saveReviewPrompt($event)"
          @delete="project.deleteReviewPrompt($event)"
        />
        <CodePreview
          :content="project.previewContent"
          :file-path="project.selectedFilePath"
          :file-size="project.selectedFileSize"
          :is-loading="project.isLoading"
          :active-tab="ui.previewTab"
          :export-content="project.exportPreviewContent"
          :checked-count="project.checkedFiles.length"
          :checked-files="project.checkedFiles"
          :secrets="currentFileSecrets"
          :review-content="project.reviewContent"
          :is-reviewing="project.isReviewing"
          @update:active-tab="ui.previewTab = $event"
          @update:export-content="project.exportPreviewContent = $event"
          @mask-secrets="onMaskSecrets"
          @exclude-file="onExcludeFile"
        />
      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar
      :file-count="project.checkedFiles.length"
      :token-count="project.totalTokens"
      :total-bytes="project.totalBytes"
      :has-files="!!project.fileTree"
      :copy-success="ui.copySuccess"
      :export-success="ui.exportSuccess"
      :export-format="ui.exportFormat"
      :include-diff="ui.includeDiff"
      :is-git-repo="!!project.gitStatus?.is_repo"
      :is-reviewing="project.isReviewing"
      @copy="onCopyToClipboard"
      @export="onExportToFile"
      @review="onStartReview"
      @update:export-format="ui.exportFormat = $event"
      @update:include-diff="ui.includeDiff = $event"
    />

    <!-- CodePack: Toast 通知容器 -->
    <ToastContainer />

    <!-- CodePack: 插件管理面板 -->
    <SettingsPanel v-if="ui.showSettings" @close="ui.showSettings = false" />

    <!-- CodePack: 排除规则编辑器 -->
    <ExcludeRulesEditor
      :rules="project.excludeRules"
      :visible="ui.showExcludeEditor"
      @save="(r: string[]) => { project.saveExcludeRules(r); ui.showExcludeEditor = false; }"
      @close="ui.showExcludeEditor = false"
    />

    <!-- CodePack: 安全拦截对话框 -->
    <Teleport to="body">
      <div
        v-if="securityDialog.show"
        class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60"
        @click.self="onSecurityDialogChoice('cancel')"
      >
        <div class="bg-dark-800 border border-dark-600 rounded-xl shadow-2xl p-6 max-w-md w-full mx-4">
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">🛡️</span>
            <h3 class="text-base font-semibold text-dark-100">检测到敏感信息</h3>
          </div>
          <p class="text-sm text-dark-300 mb-2">
            在已选文件中检测到 <span class="text-red-400 font-medium">{{ project.totalSecretCount }}</span> 个潜在敏感信息
            （{{ Object.keys(project.secretsMap).length }} 个文件）。
          </p>
          <p class="text-xs text-dark-500 mb-5">
            包含 API Key、密码等敏感数据可能导致安全风险。建议脱敏后再发送给 AI。
          </p>
          <div class="flex items-center gap-2 justify-end">
            <button
              class="px-3 py-1.5 text-xs rounded-lg bg-dark-700 text-dark-400 hover:bg-dark-600 hover:text-dark-200 transition-colors"
              @click="onSecurityDialogChoice('cancel')"
            >取消</button>
            <button
              class="px-3 py-1.5 text-xs rounded-lg bg-yellow-500/15 text-yellow-400 hover:bg-yellow-500/25 transition-colors"
              @click="onSecurityDialogChoice('force')"
            >依然{{ securityDialog.action === 'copy' ? '复制' : '导出' }}</button>
            <button
              class="px-3 py-1.5 text-xs rounded-lg bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30 transition-colors font-medium"
              @click="onSecurityDialogChoice('mask')"
            >自动脱敏并{{ securityDialog.action === 'copy' ? '复制' : '导出' }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
