<!-- CodePack: 双模式预览组件（单文件预览 + 导出预览） -->
<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  content: string;
  filePath: string;
  fileSize: number;
  isLoading: boolean;
  // CodePack: 导出预览模式相关
  activeTab: "file" | "export";
  exportContent: string;
  checkedCount: number;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", tab: "file" | "export"): void;
}>();

const lines = computed(() => {
  if (!props.content) return [];
  return props.content.split("\n");
});

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
      </div>
      <!-- CodePack: 右侧信息 -->
      <div class="pr-4 text-xs text-dark-500 truncate max-w-sm">
        <template v-if="activeTab === 'file' && filePath">
          {{ filePath.replace(/\\/g, '/').split('/').pop() }}
          <span v-if="fileSize > 0" class="text-dark-600 ml-2">{{ formatSize(fileSize) }}</span>
        </template>
        <template v-else-if="activeTab === 'export' && exportContent">
          共 {{ exportLines.length }} 行
        </template>
      </div>
    </div>

    <!-- CodePack: 单文件预览模式 -->
    <template v-if="activeTab === 'file'">
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
        <table class="w-full border-collapse">
          <tbody>
            <tr v-for="(line, idx) in lines" :key="idx" class="hover:bg-dark-800/30">
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
    <template v-else>
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
  </div>
</template>
