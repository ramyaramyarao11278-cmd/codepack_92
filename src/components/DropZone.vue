<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

const props = defineProps<{
  isScanning: boolean;
  isDragging: boolean;
  scanMessage?: string;
}>();

const emit = defineEmits<{
  (e: "folder-drop", path: string): void;
}>();

async function onClickOpen() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择项目文件夹",
    });
    if (selected) {
      emit("folder-drop", selected as string);
    }
  } catch (e) {
    console.error("Open dialog failed:", e);
  }
}
</script>

<template>
  <div
    class="flex flex-col items-center justify-center h-full p-6 transition-all duration-200"
    :class="isDragging ? 'bg-emerald-400/5' : ''"
  >
    <div
      class="flex flex-col items-center justify-center w-full h-64 border-2 border-dashed rounded-xl transition-all duration-200 cursor-pointer"
      :class="
        isDragging
          ? 'border-emerald-400 bg-emerald-400/5'
          : 'border-dark-600 hover:border-dark-400'
      "
      @click="onClickOpen"
    >
      <div v-if="isScanning" class="flex flex-col items-center gap-3">
        <div
          class="w-10 h-10 border-2 border-emerald-400 border-t-transparent rounded-full animate-spin"
        />
        <span class="text-sm text-dark-400">{{ scanMessage || '扫描中...' }}</span>
      </div>
      <div v-else class="flex flex-col items-center gap-3">
        <svg
          class="w-12 h-12 text-dark-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
          />
        </svg>
        <div class="text-center">
          <p class="text-sm text-dark-300">拖拽项目文件夹到这里</p>
          <p class="text-xs text-dark-500 mt-1">或点击选择文件夹</p>
        </div>
      </div>
    </div>
  </div>
</template>
