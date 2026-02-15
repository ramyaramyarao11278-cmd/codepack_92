<script setup lang="ts">
defineProps<{
  presets: Record<string, string[]>;
  activePreset: string;
  showInput: boolean;
  newName: string;
  hasTree: boolean;
}>();

const emit = defineEmits<{
  (e: "load", name: string): void;
  (e: "delete", name: string): void;
  (e: "show-input"): void;
  (e: "hide-input"): void;
  (e: "update:new-name", value: string): void;
  (e: "save"): void;
}>();
</script>

<template>
  <div
    v-if="Object.keys(presets).length > 0 || showInput"
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
        @click="emit('load', name)"
      >
        {{ name }}
        <span
          class="text-dark-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity ml-0.5"
          @click.stop="emit('delete', name)"
          title="删除预设"
        >✕</span>
      </button>
      <button
        v-if="!showInput"
        class="px-1.5 py-0.5 text-xs text-dark-500 hover:text-emerald-400 border border-dashed border-dark-600 hover:border-emerald-400/30 rounded-md transition-colors"
        @click="emit('show-input')"
        title="保存当前选择为预设"
      >+</button>
    </div>
    <div v-if="showInput" class="flex items-center gap-1.5">
      <input
        :value="newName"
        class="flex-1 px-2 py-1 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
        placeholder="预设名称..."
        @input="emit('update:new-name', ($event.target as HTMLInputElement).value)"
        @keyup.enter="emit('save')"
        @keyup.escape="emit('hide-input')"
      />
      <button
        class="px-2 py-1 text-xs bg-emerald-400/15 text-emerald-400 rounded-md hover:bg-emerald-400/25 transition-colors"
        @click="emit('save')"
      >保存</button>
      <button
        class="px-2 py-1 text-xs text-dark-500 hover:text-dark-300 transition-colors"
        @click="emit('hide-input')"
      >取消</button>
    </div>
  </div>
  <!-- CodePack: 无预设时显示保存入口 -->
  <div
    v-else-if="hasTree"
    class="px-3 py-1 border-b border-dark-700 bg-dark-850"
  >
    <button
      class="text-xs text-dark-500 hover:text-emerald-400 transition-colors"
      @click="emit('show-input')"
      title="保存当前选择为预设"
    >+ 保存预设</button>
  </div>
</template>
