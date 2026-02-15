<!-- CodePack: 增强状态栏，显示文件大小和 token 状态提示 -->
<script setup lang="ts">
import { computed } from "vue";

import type { ExportFormat } from "../types";

const props = defineProps<{
  fileCount: number;
  tokenCount: number;
  totalBytes: number;
  hasFiles: boolean;
  copySuccess: boolean;
  exportSuccess: boolean;
  exportFormat: ExportFormat;
  includeDiff: boolean;
  isGitRepo: boolean;
  isReviewing: boolean;
}>();

const emit = defineEmits<{
  (e: "copy"): void;
  (e: "export"): void;
  (e: "review"): void;
  (e: "update:exportFormat", value: ExportFormat): void;
  (e: "update:includeDiff", value: boolean): void;
}>();

const formatLabels: Record<ExportFormat, string> = {
  plain: "Plain",
  markdown: "Markdown",
  xml: "XML",
};

function formatNumber(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return n.toString();
}

function formatSize(bytes: number): string {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + " KB";
  return bytes + " B";
}

// CodePack: token 数量状态提示
const tokenStatus = computed(() => {
  const t = props.tokenCount;
  if (t <= 0) return null;
  if (t < 30000) return { text: "适合大部分 AI 模型", color: "text-emerald-400" };
  if (t < 100000) return { text: "建议分模块导出", color: "text-yellow-400" };
  return { text: "超出大部分 AI 上下文限制", color: "text-red-400" };
});

const canAct = computed(() => props.hasFiles && props.fileCount > 0);
</script>

<template>
  <footer class="flex items-center justify-between px-4 py-2.5 border-t border-dark-700 bg-dark-950 shrink-0">
    <!-- CodePack: 左侧统计信息 -->
    <div class="flex items-center gap-3 text-xs text-dark-400 min-w-0">
      <span v-if="hasFiles" class="flex items-center gap-1.5 shrink-0">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        已选 <span class="text-dark-200 font-medium">{{ fileCount }}</span> 个文件
      </span>
      <!-- CodePack: 文件大小 -->
      <span v-if="hasFiles && totalBytes > 0" class="text-dark-500 shrink-0">
        {{ formatSize(totalBytes) }}
      </span>
      <span v-if="hasFiles" class="flex items-center gap-1.5 shrink-0">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
        </svg>
        预估 <span class="text-dark-200 font-medium">{{ formatNumber(tokenCount) }}</span> tokens
      </span>
      <!-- CodePack: token 状态提示 -->
      <span v-if="tokenStatus" :class="tokenStatus.color" class="text-[11px] shrink-0">
        {{ tokenStatus.text }}
      </span>
    </div>

    <!-- CodePack: 右侧操作按钮 -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- CodePack: 导出格式选择器 -->
      <div v-if="hasFiles" class="flex items-center gap-0.5 bg-dark-800 rounded-md border border-dark-700 p-0.5">
        <button
          v-for="fmt in (['plain', 'markdown', 'xml'] as ExportFormat[])"
          :key="fmt"
          class="px-2 py-1 text-[11px] rounded transition-colors"
          :class="exportFormat === fmt
            ? 'bg-dark-600 text-dark-100'
            : 'text-dark-500 hover:text-dark-300'"
          @click="emit('update:exportFormat', fmt)"
        >{{ formatLabels[fmt] }}</button>
      </div>
      <label
        v-if="hasFiles && isGitRepo"
        class="flex items-center gap-1 text-xs cursor-pointer select-none"
        :class="includeDiff ? 'text-yellow-400' : 'text-dark-500 hover:text-dark-300'"
        title="在导出中包含 Git Diff"
      >
        <input
          type="checkbox"
          :checked="includeDiff"
          class="w-3 h-3 rounded border-dark-500 bg-dark-700 text-yellow-400 focus:ring-yellow-400/30 focus:ring-offset-0 cursor-pointer"
          @change="emit('update:includeDiff', !includeDiff)"
        />
        Diff
      </label>
      <button
        :disabled="!canAct || isReviewing"
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-150"
        :class="isReviewing
          ? 'bg-violet-500/20 text-violet-400 border border-violet-500/30 animate-pulse cursor-wait'
          : canAct
            ? 'bg-violet-500/10 text-violet-400 border border-violet-500/20 hover:bg-violet-500/20'
            : 'bg-dark-800 text-dark-600 cursor-not-allowed border border-dark-700'"
        @click="canAct && !isReviewing && emit('review')"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
        {{ isReviewing ? 'Reviewing...' : 'Start Review' }}
      </button>
      <button
        :disabled="!canAct"
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-150"
        :class="copySuccess
          ? 'bg-emerald-500 text-white'
          : canAct
            ? 'bg-dark-700 text-dark-200 hover:bg-dark-600 hover:text-white'
            : 'bg-dark-800 text-dark-600 cursor-not-allowed'"
        @click="canAct && emit('copy')"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path v-if="!copySuccess" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        {{ copySuccess ? "已复制 ✓" : "复制到剪贴板" }}
      </button>

      <button
        :disabled="!canAct"
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-150"
        :class="exportSuccess
          ? 'bg-emerald-500 text-white'
          : canAct
            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 hover:bg-emerald-500/20'
            : 'bg-dark-800 text-dark-600 cursor-not-allowed border border-dark-700'"
        @click="canAct && emit('export')"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path v-if="!exportSuccess" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        {{ exportSuccess ? "已导出 ✓" : "导出为文件" }}
      </button>
    </div>
  </footer>
</template>
