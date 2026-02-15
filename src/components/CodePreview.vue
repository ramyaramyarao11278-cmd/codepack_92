<!-- CodePack: 三模式预览组件（单文件预览 + 导出预览 + 统计） -->
<script setup lang="ts">
import { computed, ref, watch } from "vue";
import StatsPanel from "./StatsPanel.vue";
import { useHighlighter } from "../composables/useHighlighter";
import type { SecretMatch } from "../types";

const { highlightedHtml, isHighlighting, highlight } = useHighlighter();

const isEditing = ref(false);
const editedContent = ref("");

const props = defineProps<{
  content: string;
  filePath: string;
  fileSize: number;
  isLoading: boolean;
  activeTab: "file" | "export" | "stats" | "review";
  exportContent: string;
  checkedCount: number;
  checkedFiles: string[];
  secrets?: SecretMatch[];
  reviewContent?: string;
  isReviewing?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", tab: "file" | "export" | "stats" | "review"): void;
  (e: "update:exportContent", content: string): void;
  (e: "mask-secrets"): void;
  (e: "exclude-file"): void;
}>();

const secretsDismissed = ref(false);

const secretLines = computed(() => {
  if (!props.secrets || props.secrets.length === 0) return new Set<number>();
  return new Set(props.secrets.map((s) => s.line_number));
});

watch(() => props.filePath, () => {
  secretsDismissed.value = false;
});

function toggleEdit() {
  if (!isEditing.value) {
    editedContent.value = props.exportContent;
    isEditing.value = true;
  } else {
    emit("update:exportContent", editedContent.value);
    isEditing.value = false;
  }
}

function cancelEdit() {
  isEditing.value = false;
  editedContent.value = "";
}

// Reset editing when export content changes from outside
watch(() => props.exportContent, () => {
  if (isEditing.value) {
    isEditing.value = false;
    editedContent.value = "";
  }
});

const lines = computed(() => {
  if (!props.content) return [];
  return props.content.split("\n");
});

// CodePack: trigger shiki highlighting when file content or path changes
watch(
  [() => props.content, () => props.filePath],
  ([code, path]) => {
    if (code && path) highlight(code, path);
    else highlightedHtml.value = "";
  },
  { immediate: true }
);

const exportLines = computed(() => {
  if (!props.exportContent) return [];
  return props.exportContent.split("\n");
});

// CodePack: 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(1) + " KB";
  return bytes + " B";
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- CodePack: 预览区顶部 tab 栏 -->
    <div class="flex items-center justify-between border-b border-dark-700 bg-dark-800/50 shrink-0">
      <div class="flex">
        <button
          class="px-4 py-2 text-xs font-medium transition-colors border-b-2"
          :class="activeTab === 'file'
            ? 'text-emerald-400 border-emerald-400 bg-dark-800/30'
            : 'text-dark-500 border-transparent hover:text-dark-300'"
          @click="emit('update:activeTab', 'file')"
        >
          单文件预览
        </button>
        <button
          class="px-4 py-2 text-xs font-medium transition-colors border-b-2"
          :class="activeTab === 'export'
            ? 'text-emerald-400 border-emerald-400 bg-dark-800/30'
            : 'text-dark-500 border-transparent hover:text-dark-300'"
          @click="emit('update:activeTab', 'export')"
        >
          导出预览
        </button>
        <button
          class="px-4 py-2 text-xs font-medium transition-colors border-b-2"
          :class="activeTab === 'stats'
            ? 'text-emerald-400 border-emerald-400 bg-dark-800/30'
            : 'text-dark-500 border-transparent hover:text-dark-300'"
          @click="emit('update:activeTab', 'stats')"
        >
          统计
        </button>
        <button
          class="px-4 py-2 text-xs font-medium transition-colors border-b-2"
          :class="activeTab === 'review'
            ? 'text-violet-400 border-violet-400 bg-dark-800/30'
            : 'text-dark-500 border-transparent hover:text-dark-300'"
          @click="emit('update:activeTab', 'review')"
        >
          <span v-if="isReviewing" class="inline-block w-1.5 h-1.5 rounded-full bg-violet-400 animate-pulse mr-1"></span>
          AI Review
        </button>
      </div>
      <!-- CodePack: 右侧信息 -->
      <div class="pr-4 text-xs text-dark-500 truncate max-w-sm">
        <template v-if="activeTab === 'file' && filePath">
          {{ filePath.replace(/\\/g, '/').split('/').pop() }}
          <span v-if="fileSize > 0" class="text-dark-600 ml-2">{{ formatSize(fileSize) }}</span>
        </template>
        <template v-else-if="activeTab === 'export' && exportContent">
          <span v-if="isEditing" class="text-yellow-400 mr-2">编辑中</span>
          共 {{ exportLines.length }} 行
          <button
            v-if="exportContent && !isEditing"
            class="ml-2 px-2 py-0.5 text-xs rounded bg-dark-700 text-dark-300 hover:bg-dark-600 hover:text-dark-200 transition-colors"
            @click="toggleEdit"
          >编辑</button>
          <template v-if="isEditing">
            <button
              class="ml-2 px-2 py-0.5 text-xs rounded bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30 transition-colors"
              @click="toggleEdit"
            >保存</button>
            <button
              class="ml-1 px-2 py-0.5 text-xs rounded bg-dark-700 text-dark-400 hover:bg-dark-600 transition-colors"
              @click="cancelEdit"
            >取消</button>
          </template>
        </template>
      </div>
    </div>

    <!-- CodePack: 统计面板 -->
    <template v-if="activeTab === 'stats'">
      <StatsPanel :checked-files="checkedFiles" :visible="activeTab === 'stats'" />
    </template>

    <!-- CodePack: 单文件预览模式 -->
    <template v-else-if="activeTab === 'file'">
      <!-- Security Warning Bar -->
      <div
        v-if="secrets && secrets.length > 0 && !secretsDismissed"
        class="flex items-center gap-2 px-4 py-2 bg-red-500/10 border-b border-red-500/20 shrink-0"
      >
        <span class="text-red-400 text-xs font-medium">⚠️ 检测到 {{ secrets.length }} 个潜在敏感信息</span>
        <div class="ml-auto flex items-center gap-1.5">
          <button
            class="px-2 py-0.5 text-xs rounded bg-red-500/20 text-red-300 hover:bg-red-500/30 transition-colors"
            @click="emit('mask-secrets')"
          >一键脱敏</button>
          <button
            class="px-2 py-0.5 text-xs rounded bg-dark-700 text-dark-400 hover:bg-dark-600 transition-colors"
            @click="secretsDismissed = true"
          >忽略</button>
          <button
            class="px-2 py-0.5 text-xs rounded bg-dark-700 text-dark-400 hover:bg-dark-600 transition-colors"
            @click="emit('exclude-file')"
          >排除此文件</button>
        </div>
      </div>

      <div v-if="isLoading" class="flex items-center justify-center flex-1">
        <div class="w-8 h-8 border-2 border-emerald-400 border-t-transparent rounded-full animate-spin" />
      </div>

      <div
        v-else-if="!filePath"
        class="flex flex-col items-center justify-center flex-1 text-dark-600"
      >
        <svg class="w-16 h-16 mb-4 text-dark-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
        </svg>
        <p class="text-sm">选择文件以预览代码</p>
      </div>

      <div v-else class="flex-1 overflow-auto font-mono text-xs leading-5">
        <!-- CodePack: shiki syntax highlighted preview -->
        <div
          v-if="highlightedHtml && !isHighlighting"
          class="shiki-preview"
          v-html="highlightedHtml"
        />
        <!-- CodePack: fallback plain text while highlighting -->
        <table v-else class="w-full border-collapse">
          <tbody>
            <tr
              v-for="(line, idx) in lines"
              :key="idx"
              class="hover:bg-dark-800/30"
              :class="{ 'bg-red-500/10': secretLines.has(idx + 1) }"
            >
              <td class="sticky left-0 w-12 px-3 text-right text-dark-600 bg-dark-900 select-none border-r border-dark-800 shrink-0">
                {{ idx + 1 }}
              </td>
              <td class="px-4 whitespace-pre text-dark-200">{{ line }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <!-- CodePack: 导出预览模式 -->
    <template v-else-if="activeTab === 'export'">
      <div
        v-if="checkedCount === 0"
        class="flex flex-col items-center justify-center flex-1 text-dark-600"
      >
        <svg class="w-16 h-16 mb-4 text-dark-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <p class="text-sm">请在左侧勾选要导出的文件</p>
      </div>

      <div v-else-if="!exportContent" class="flex items-center justify-center flex-1">
        <div class="w-8 h-8 border-2 border-emerald-400 border-t-transparent rounded-full animate-spin" />
      </div>

      <!-- CodePack: 编辑模式 -->
      <div v-else-if="isEditing" class="flex-1 overflow-hidden">
        <textarea
          v-model="editedContent"
          class="w-full h-full p-4 font-mono text-xs leading-5 text-dark-200 bg-dark-900 border-none outline-none resize-none"
          spellcheck="false"
        />
      </div>

      <!-- CodePack: 只读预览模式 -->
      <div v-else class="flex-1 overflow-auto font-mono text-xs leading-5">
        <table class="w-full border-collapse">
          <tbody>
            <tr v-for="(line, idx) in exportLines" :key="idx" class="hover:bg-dark-800/30">
              <td class="sticky left-0 w-12 px-3 text-right text-dark-600 bg-dark-900 select-none border-r border-dark-800 shrink-0">
                {{ idx + 1 }}
              </td>
              <td class="px-4 whitespace-pre text-dark-200">{{ line }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <!-- CodePack: AI Review 结果面板 -->
    <template v-else-if="activeTab === 'review'">
      <div v-if="!reviewContent && !isReviewing" class="flex flex-col items-center justify-center flex-1 text-dark-600">
        <svg class="w-16 h-16 mb-4 text-dark-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
        <p class="text-sm mb-2">点击底部 <span class="text-violet-400 font-medium">Start Review</span> 开始 AI 审查</p>
        <p class="text-xs text-dark-700">选择一个 Reviewer 角色可获得更精准的审查结果</p>
      </div>

      <div v-else-if="isReviewing && !reviewContent" class="flex flex-col items-center justify-center flex-1 text-dark-500">
        <div class="w-8 h-8 border-2 border-violet-400 border-t-transparent rounded-full animate-spin mb-4" />
        <p class="text-sm">AI 正在审查你的代码...</p>
        <p class="text-xs text-dark-600 mt-1">结果将以流式方式实时显示</p>
      </div>

      <div v-else class="flex-1 overflow-auto">
        <div class="p-5 prose prose-invert prose-sm max-w-none review-content">
          <div v-if="isReviewing" class="flex items-center gap-2 mb-3 text-xs text-violet-400">
            <div class="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
            正在接收...
          </div>
          <div class="whitespace-pre-wrap text-sm text-dark-200 leading-relaxed font-sans" v-text="reviewContent" />
        </div>
      </div>
    </template>
  </div>
</template>
