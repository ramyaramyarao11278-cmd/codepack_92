<!-- CodePack: AI Review 角色预设选择条 -->
<script setup lang="ts">
import { ref } from "vue";
import type { ReviewPrompt } from "../types";

const props = defineProps<{
  prompts: ReviewPrompt[];
  activePrompt: string;
  hasFiles: boolean;
}>();

const emit = defineEmits<{
  (e: "select", name: string): void;
  (e: "deselect"): void;
  (e: "save", prompt: ReviewPrompt): void;
  (e: "delete", name: string): void;
}>();

const showAdd = ref(false);
const newName = ref("");
const newIcon = ref("📝");
const newInstruction = ref("");

function onSelect(name: string) {
  if (props.activePrompt === name) {
    emit("deselect");
  } else {
    emit("select", name);
  }
}

function onSave() {
  const name = newName.value.trim();
  const instruction = newInstruction.value.trim();
  if (!name || !instruction) return;
  emit("save", {
    name,
    icon: newIcon.value || "📝",
    instruction,
    builtin: false,
  });
  newName.value = "";
  newIcon.value = "📝";
  newInstruction.value = "";
  showAdd.value = false;
}
</script>

<template>
  <div v-if="hasFiles" class="border-b border-dark-700 bg-dark-850">
    <!-- Prompt selector row -->
    <div class="flex items-center gap-1.5 px-3 py-1.5 flex-wrap">
      <span class="text-[10px] text-dark-500 uppercase tracking-wider mr-1 shrink-0">Review</span>
      <button
        v-for="p in prompts"
        :key="p.name"
        class="group flex items-center gap-1 px-2 py-0.5 text-xs rounded-md border transition-colors"
        :class="
          activePrompt === p.name
            ? 'bg-violet-400/15 text-violet-400 border-violet-400/30'
            : 'bg-dark-800 text-dark-400 border-dark-600 hover:text-dark-200 hover:border-dark-500'
        "
        :title="p.instruction.substring(0, 120) + '...'"
        @click="onSelect(p.name)"
      >
        <span>{{ p.icon }}</span>
        <span>{{ p.name }}</span>
        <span
          v-if="!p.builtin"
          class="text-dark-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity ml-0.5"
          @click.stop="emit('delete', p.name)"
          title="删除预设"
        >✕</span>
      </button>
      <button
        v-if="!showAdd"
        class="px-1.5 py-0.5 text-xs text-dark-500 hover:text-violet-400 border border-dashed border-dark-600 hover:border-violet-400/30 rounded-md transition-colors"
        @click="showAdd = true"
        title="添加自定义角色"
      >+</button>
    </div>

    <!-- Add custom prompt form -->
    <div v-if="showAdd" class="px-3 pb-2 flex flex-col gap-1.5">
      <div class="flex items-center gap-1.5">
        <input
          v-model="newIcon"
          class="w-8 px-1 py-1 text-xs text-center bg-dark-800 border border-dark-600 rounded-md text-dark-200 focus:outline-none focus:border-violet-400/50"
          maxlength="2"
          placeholder="📝"
        />
        <input
          v-model="newName"
          class="flex-1 px-2 py-1 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-violet-400/50"
          placeholder="角色名称..."
          @keyup.escape="showAdd = false"
        />
      </div>
      <textarea
        v-model="newInstruction"
        class="w-full px-2 py-1 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-violet-400/50 resize-none"
        rows="3"
        placeholder="Review 指令... (e.g. Focus on error handling and edge cases)"
        @keyup.escape="showAdd = false"
      />
      <div class="flex items-center gap-1.5 justify-end">
        <button
          class="px-2 py-1 text-xs text-dark-500 hover:text-dark-300 transition-colors"
          @click="showAdd = false"
        >取消</button>
        <button
          class="px-2 py-1 text-xs bg-violet-400/15 text-violet-400 rounded-md hover:bg-violet-400/25 transition-colors"
          @click="onSave"
        >保存</button>
      </div>
    </div>
  </div>
</template>
