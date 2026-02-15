<script setup lang="ts">
import { watch, onMounted, onUnmounted } from "vue";
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
import { useToast } from "./composables/useToast";
import { useProjectStore } from "./stores/useProjectStore";
import { useUIStore } from "./stores/useUIStore";
import type { PackResult } from "./types";

const toast = useToast();
const project = useProjectStore();
const ui = useUIStore();

let unlistenDrop: (() => void) | null = null;

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

// CodePack: 复制到剪贴板
async function onCopyToClipboard() {
  if (!project.fileTree) return;
  const paths = project.checkedFiles;
  if (paths.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
  try {
    const result = await invoke<PackResult>("pack_files", {
      paths,
      projectPath: project.projectPath,
      projectType: project.projectType,
      format: ui.exportFormat,
      maxFileBytes: ui.maxFileKB * 1024,
    });
    await invoke("copy_to_clipboard", { content: result.content });
    ui.copySuccess = true;
    setTimeout(() => (ui.copySuccess = false), 2000);
    if (result.skipped_files && result.skipped_files.length > 0) {
      toast.show({
        type: "info",
        message: `跳过了 ${result.skipped_files.length} 个超大文件（>${ui.maxFileKB}KB）`,
        duration: 4000,
      });
    }
    toast.show({
      type: "success",
      message: `已复制 ${result.file_count} 个文件到剪贴板（${formatTokens(result.estimated_tokens)} tokens）`,
    });
  } catch (e) {
    toast.show({ type: "error", message: `复制失败: ${e}` });
  }
}

// CodePack: 导出为文件
async function onExportToFile() {
  if (!project.fileTree) return;
  const paths = project.checkedFiles;
  if (paths.length === 0) {
    toast.show({ type: "info", message: "请先选择要导出的文件" });
    return;
  }
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
    const resultPath = await invoke<string>("export_to_file", {
      paths,
      projectPath: project.projectPath,
      projectType: project.projectType,
      savePath,
      format: ui.exportFormat,
      maxFileBytes: ui.maxFileKB * 1024,
    });
    ui.exportSuccess = true;
    setTimeout(() => (ui.exportSuccess = false), 2000);
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
              @select="onFileSelect"
              @toggle="project.onTreeChanged()"
              @context-action="project.contextAction"
            />
          </div>
        </div>
      </div>

      <!-- Code Preview (Right Panel) -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <CodePreview
          :content="project.previewContent"
          :file-path="project.selectedFilePath"
          :file-size="project.selectedFileSize"
          :is-loading="project.isLoading"
          :active-tab="ui.previewTab"
          :export-content="project.exportPreviewContent"
          :checked-count="project.checkedFiles.length"
          :checked-files="project.checkedFiles"
          @update:active-tab="ui.previewTab = $event"
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
      @copy="onCopyToClipboard"
      @export="onExportToFile"
      @update:export-format="ui.exportFormat = $event"
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
  </div>
</template>
