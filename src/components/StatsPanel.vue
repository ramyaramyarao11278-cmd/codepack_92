<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectStats } from "../types";

const props = defineProps<{
  checkedFiles: string[];
  visible: boolean;
}>();

const stats = ref<ProjectStats | null>(null);
const isLoading = ref(false);

async function loadStats() {
  if (props.checkedFiles.length === 0) {
    stats.value = null;
    return;
  }
  isLoading.value = true;
  try {
    stats.value = await invoke<ProjectStats>("get_project_stats", {
      paths: props.checkedFiles,
    });
  } catch {
    stats.value = null;
  } finally {
    isLoading.value = false;
  }
}

watch(
  () => props.visible,
  (v) => {
    if (v) loadStats();
  }
);

watch(
  () => props.checkedFiles,
  () => {
    if (props.visible) loadStats();
  },
  { deep: true }
);

function formatBytes(bytes: number): string {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + " KB";
  return bytes + " B";
}

function formatLines(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return n.toString();
}

// CodePack: 语言对应的颜色
const langColors: Record<string, string> = {
  Rust: "#dea584",
  TypeScript: "#3178c6",
  JavaScript: "#f1e05a",
  Vue: "#41b883",
  Svelte: "#ff3e00",
  Python: "#3572a5",
  Kotlin: "#a97bff",
  Java: "#b07219",
  Dart: "#00b4ab",
  Go: "#00add8",
  Ruby: "#701516",
  PHP: "#4f5d95",
  Swift: "#f05138",
  C: "#555555",
  "C++": "#f34b7d",
  "C/C++ Header": "#f34b7d",
  "C#": "#178600",
  HTML: "#e34c26",
  CSS: "#563d7c",
  "CSS (preprocessor)": "#c6538c",
  JSON: "#a0a0a0",
  YAML: "#cb171e",
  TOML: "#9c4221",
  Markdown: "#083fa1",
  Shell: "#89e051",
  SQL: "#e38c00",
};

function getLangColor(lang: string): string {
  return langColors[lang] || "#8e8ea0";
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div v-if="isLoading" class="flex-1 flex items-center justify-center text-dark-500 text-sm">
      加载统计中...
    </div>
    <div v-else-if="!stats || stats.total_files === 0" class="flex-1 flex items-center justify-center text-dark-500 text-sm">
      请选择文件以查看统计
    </div>
    <div v-else class="flex-1 overflow-auto p-4 space-y-4">
      <!-- 总览 -->
      <div class="grid grid-cols-3 gap-3">
        <div class="bg-dark-800 rounded-lg p-3 text-center">
          <div class="text-2xl font-bold text-dark-100">{{ stats.total_files }}</div>
          <div class="text-xs text-dark-500 mt-1">文件数</div>
        </div>
        <div class="bg-dark-800 rounded-lg p-3 text-center">
          <div class="text-2xl font-bold text-dark-100">{{ formatLines(stats.total_lines) }}</div>
          <div class="text-xs text-dark-500 mt-1">代码行数</div>
        </div>
        <div class="bg-dark-800 rounded-lg p-3 text-center">
          <div class="text-2xl font-bold text-dark-100">{{ formatBytes(stats.total_bytes) }}</div>
          <div class="text-xs text-dark-500 mt-1">总大小</div>
        </div>
      </div>

      <!-- 语言分布条 -->
      <div>
        <div class="text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">语言分布</div>
        <div class="flex h-3 rounded-full overflow-hidden bg-dark-800">
          <div
            v-for="lang in stats.languages"
            :key="lang.language"
            :style="{
              width: (lang.line_count / stats.total_lines * 100) + '%',
              backgroundColor: getLangColor(lang.language),
              minWidth: lang.line_count > 0 ? '2px' : '0',
            }"
            :title="`${lang.language}: ${(lang.line_count / stats.total_lines * 100).toFixed(1)}%`"
            class="transition-all"
          />
        </div>
      </div>

      <!-- 语言列表 -->
      <div class="space-y-1.5">
        <div
          v-for="lang in stats.languages"
          :key="lang.language"
          class="flex items-center justify-between text-xs py-1.5 px-2 rounded-md hover:bg-dark-800/50 transition-colors"
        >
          <div class="flex items-center gap-2">
            <span
              class="w-2.5 h-2.5 rounded-full shrink-0"
              :style="{ backgroundColor: getLangColor(lang.language) }"
            />
            <span class="text-dark-200">{{ lang.language }}</span>
          </div>
          <div class="flex items-center gap-4 text-dark-500">
            <span>{{ lang.file_count }} 文件</span>
            <span>{{ formatLines(lang.line_count) }} 行</span>
            <span class="w-12 text-right">{{ (lang.line_count / stats.total_lines * 100).toFixed(1) }}%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
