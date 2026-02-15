<script setup lang="ts">
import type { ProjectMetadata } from "../types";

defineProps<{
  metadata: ProjectMetadata | null;
}>();
</script>

<template>
  <div
    v-if="metadata && (metadata.version || metadata.runtime.length > 0 || metadata.dependencies.length > 0)"
    class="flex items-center gap-4 px-5 py-1.5 border-b border-dark-700 bg-dark-900/80 text-xs text-dark-400 shrink-0 overflow-x-auto"
  >
    <span v-if="metadata.version" class="flex items-center gap-1 shrink-0">
      <span class="text-dark-500">v</span>
      <span class="text-dark-300">{{ metadata.version }}</span>
    </span>
    <span v-if="metadata.runtime.length > 0" class="flex items-center gap-1 shrink-0">
      <span class="text-dark-500">环境</span>
      <span class="text-amber-400/80">{{ metadata.runtime.join(' · ') }}</span>
    </span>
    <span v-if="metadata.entry_point" class="flex items-center gap-1 shrink-0">
      <span class="text-dark-500">入口</span>
      <span class="text-emerald-400/70">{{ metadata.entry_point }}</span>
    </span>
    <span v-if="metadata.dependencies.length > 0" class="flex items-center gap-1 shrink-0">
      <span class="text-dark-500">依赖</span>
      <span class="text-dark-300">{{ metadata.dependencies.length }}个</span>
    </span>
    <span v-if="metadata.requirements.length > 0" class="flex items-center gap-1 truncate">
      <span class="text-dark-500 shrink-0">清单</span>
      <span class="text-sky-400/70 truncate">{{ metadata.requirements.slice(0, 6).join(', ') }}<span v-if="metadata.requirements.length > 6" class="text-dark-500"> +{{ metadata.requirements.length - 6 }}</span></span>
    </span>
  </div>
</template>
