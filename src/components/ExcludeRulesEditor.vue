<!-- CodePack: 可编辑排除规则面板 -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  rules: string[];
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: "save", rules: string[]): void;
  (e: "close"): void;
}>();

const localRules = ref<string[]>([]);
const newRule = ref("");

watch(() => props.visible, (v) => {
  if (v) localRules.value = [...props.rules];
}, { immediate: true });

function addRule() {
  const rule = newRule.value.trim();
  if (!rule || localRules.value.includes(rule)) return;
  localRules.value.push(rule);
  newRule.value = "";
}

function removeRule(idx: number) {
  localRules.value.splice(idx, 1);
}

function onSave() {
  emit("save", [...localRules.value]);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    addRule();
  }
}
</script>

<template>
  <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="emit('close')">
    <div class="w-[420px] max-h-[80vh] bg-dark-800 border border-dark-600 rounded-xl shadow-2xl flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-dark-700">
        <h3 class="text-sm font-medium text-dark-200">排除规则</h3>
        <button class="text-dark-500 hover:text-dark-300 text-xs" @click="emit('close')">✕</button>
      </div>

      <!-- Description -->
      <div class="px-4 py-2 text-xs text-dark-500 border-b border-dark-700">
        添加目录名或文件名模式来排除文件。例如：<code class="text-emerald-400">docs</code>、<code class="text-emerald-400">*.log</code>、<code class="text-emerald-400">temp</code>
      </div>

      <!-- Rules List -->
      <div class="flex-1 overflow-auto px-4 py-2 min-h-[120px] max-h-[300px]">
        <div v-if="localRules.length === 0" class="text-xs text-dark-600 text-center py-4">
          暂无自定义排除规则
        </div>
        <div
          v-for="(rule, idx) in localRules"
          :key="idx"
          class="flex items-center justify-between py-1 group"
        >
          <span class="text-xs text-dark-300 font-mono">{{ rule }}</span>
          <button
            class="text-dark-600 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity"
            @click="removeRule(idx)"
          >✕</button>
        </div>
      </div>

      <!-- Add Rule Input -->
      <div class="px-4 py-2 border-t border-dark-700">
        <div class="flex gap-2">
          <input
            v-model="newRule"
            class="flex-1 px-3 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
            placeholder="输入排除规则..."
            @keydown="onKeydown"
          />
          <button
            class="px-3 py-1.5 text-xs bg-dark-700 text-dark-300 rounded-md hover:bg-dark-600 transition-colors"
            @click="addRule"
          >添加</button>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex justify-end gap-2 px-4 py-3 border-t border-dark-700">
        <button
          class="px-4 py-1.5 text-xs text-dark-400 hover:text-dark-200 transition-colors"
          @click="emit('close')"
        >取消</button>
        <button
          class="px-4 py-1.5 text-xs bg-emerald-500 text-white rounded-md hover:bg-emerald-400 transition-colors"
          @click="onSave"
        >保存并刷新</button>
      </div>
    </div>
  </div>
</template>
