<!-- CodePack: 全局 Toast 通知组件 -->
<script setup lang="ts">
import { useToast } from "../composables/useToast";
import type { ToastItem } from "../composables/useToast";

const { toasts, dismiss } = useToast();

function iconForType(type: ToastItem["type"]): string {
  switch (type) {
    case "success":
      return "✓";
    case "error":
      return "✕";
    case "info":
      return "ℹ";
  }
}
</script>

<template>
  <div class="fixed bottom-16 left-1/2 -translate-x-1/2 z-50 flex flex-col-reverse items-center gap-2 pointer-events-none">
    <Transition
      v-for="toast in toasts"
      :key="toast.id"
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 translate-y-4 scale-95"
      enter-to-class="opacity-100 translate-y-0 scale-100"
      leave-active-class="transition-all duration-300 ease-in"
      leave-from-class="opacity-100 translate-y-0 scale-100"
      leave-to-class="opacity-0 translate-y-4 scale-95"
    >
      <div
        v-if="toast.visible"
        class="pointer-events-auto flex items-center gap-2.5 px-4 py-2.5 rounded-lg shadow-lg backdrop-blur-sm border max-w-md"
        :class="{
          'bg-emerald-500/15 border-emerald-500/30 text-emerald-300': toast.type === 'success',
          'bg-red-500/15 border-red-500/30 text-red-300': toast.type === 'error',
          'bg-blue-500/15 border-blue-500/30 text-blue-300': toast.type === 'info',
        }"
      >
        <!-- CodePack: Toast 图标 -->
        <span
          class="w-5 h-5 rounded-full flex items-center justify-center text-xs font-bold shrink-0"
          :class="{
            'bg-emerald-500/20 text-emerald-400': toast.type === 'success',
            'bg-red-500/20 text-red-400': toast.type === 'error',
            'bg-blue-500/20 text-blue-400': toast.type === 'info',
          }"
        >
          {{ iconForType(toast.type) }}
        </span>

        <!-- CodePack: Toast 消息文字 -->
        <span class="text-xs text-dark-100 leading-relaxed">{{ toast.message }}</span>

        <!-- CodePack: Toast 可选操作链接 -->
        <button
          v-if="toast.action"
          class="text-xs font-medium shrink-0 ml-1 hover:underline"
          :class="{
            'text-emerald-400': toast.type === 'success',
            'text-red-400': toast.type === 'error',
            'text-blue-400': toast.type === 'info',
          }"
          @click="toast.action!.onClick()"
        >
          {{ toast.action!.label }}
        </button>

        <!-- CodePack: 关闭按钮 -->
        <button
          class="text-dark-500 hover:text-dark-300 text-xs ml-1 shrink-0"
          @click="dismiss(toast.id)"
        >
          ✕
        </button>
      </div>
    </Transition>
  </div>
</template>
