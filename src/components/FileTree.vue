<!-- CodePack: 文件树组件，支持右键菜单 -->
<script setup lang="ts">
import { reactive, computed } from "vue";
import type { FileNode } from "../types";

const props = defineProps<{
  node: FileNode;
  depth: number;
  selectedPath: string;
  // CodePack: 共享折叠状态，由根组件传入
  collapsedState: Record<string, boolean>;
  filterText?: string;
}>();

function nodeMatchesFilter(node: FileNode, filter: string): boolean {
  if (!filter) return true;
  const lower = filter.toLowerCase();
  if (node.name.toLowerCase().includes(lower)) return true;
  if (node.is_dir && node.children) {
    return node.children.some((c) => nodeMatchesFilter(c, filter));
  }
  return false;
}

const filteredChildren = computed(() => {
  if (!props.node.children) return [];
  if (!props.filterText) return props.node.children;
  return props.node.children.filter((c) => nodeMatchesFilter(c, props.filterText!));
});

const nodeVisible = computed(() => nodeMatchesFilter(props.node, props.filterText || ""));

const emit = defineEmits<{
  (e: "select", path: string): void;
  (e: "toggle"): void;
  // CodePack: 右键菜单操作向上冒泡到 App
  (e: "context-action", action: string, ext?: string): void;
}>();

// CodePack: 右键菜单状态
const contextMenu = reactive({ show: false, x: 0, y: 0, ext: "" });

function isCollapsed(path: string): boolean {
  return props.collapsedState[path] ?? false;
}

function toggleCollapse(path: string) {
  props.collapsedState[path] = !isCollapsed(path);
}

function toggleCheck(node: FileNode) {
  const newState = !node.checked;
  setChecked(node, newState);
  emit("toggle");
}

function setChecked(node: FileNode, checked: boolean) {
  node.checked = checked;
  node.indeterminate = false;
  if (node.children) {
    for (const child of node.children) {
      setChecked(child, checked);
    }
  }
}

function onChildToggle() {
  updateIndeterminate(props.node);
  emit("toggle");
}

function updateIndeterminate(node: FileNode) {
  if (!node.children || node.children.length === 0) return;
  const allChecked = node.children.every((c) => c.checked && !c.indeterminate);
  const noneChecked = node.children.every(
    (c) => !c.checked && !c.indeterminate
  );
  node.checked = allChecked;
  node.indeterminate = !allChecked && !noneChecked;
}

// CodePack: 点击文件名预览，点击目录展开/折叠
function onFileClick(node: FileNode) {
  if (!node.is_dir) {
    emit("select", node.path);
  } else {
    toggleCollapse(node.path);
  }
}

// CodePack: 右键菜单
function onContextMenu(event: MouseEvent, node: FileNode) {
  event.preventDefault();
  const ext = !node.is_dir ? (node.name.split(".").pop()?.toLowerCase() || "") : "";
  contextMenu.show = true;
  contextMenu.x = event.clientX;
  contextMenu.y = event.clientY;
  contextMenu.ext = ext;

  // CodePack: 点击任意处关闭菜单
  const close = () => {
    contextMenu.show = false;
    document.removeEventListener("click", close);
  };
  setTimeout(() => document.addEventListener("click", close), 0);
}

function onMenuAction(action: string) {
  contextMenu.show = false;
  emit("context-action", action, contextMenu.ext);
}

function getFileIcon(name: string, isDir: boolean): string {
  if (isDir) return "📁";
  const ext = name.split(".").pop()?.toLowerCase() || "";
  const iconMap: Record<string, string> = {
    ts: "🟦", tsx: "🟦", js: "🟨", jsx: "🟨", vue: "🟩", rs: "🦀",
    py: "🐍", kt: "🟣", java: "☕", dart: "🎯", json: "📋", yaml: "📋",
    yml: "📋", toml: "📋", md: "📝", css: "🎨", scss: "🎨", html: "🌐",
    xml: "🌐", gradle: "🐘", swift: "🍎", go: "🐹", rb: "💎", php: "🐘",
    c: "🔧", cpp: "�", h: "🔧", hpp: "🔧", sql: "🗄️", sh: "⚙️",
    bat: "⚙️", txt: "📄",
  };
  return iconMap[ext] || "📄";
}
</script>

<template>
  <div>
    <div
      class="flex items-center gap-1 py-0.5 px-1 rounded cursor-pointer text-sm group transition-colors duration-100"
      :class="[
        selectedPath === node.path
          ? 'bg-emerald-400/10 text-emerald-300'
          : 'hover:bg-dark-700/50 text-dark-300',
      ]"
      :style="{ paddingLeft: `${depth * 16 + 4}px` }"
      @click="onFileClick(node)"
      @contextmenu="onContextMenu($event, node)"
    >
      <!-- Collapse arrow for directories -->
      <span
        v-if="node.is_dir && node.children && node.children.length > 0"
        class="w-4 h-4 flex items-center justify-center text-dark-500 text-xs select-none shrink-0"
        @click.stop="toggleCollapse(node.path)"
      >
        {{ isCollapsed(node.path) ? "▶" : "▼" }}
      </span>
      <span v-else class="w-4 shrink-0" />

      <!-- Checkbox -->
      <label class="flex items-center shrink-0" @click.stop>
        <input
          type="checkbox"
          :checked="node.checked"
          :indeterminate="node.indeterminate"
          class="w-3.5 h-3.5 rounded border-dark-500 bg-dark-700 text-emerald-400 focus:ring-emerald-400/30 focus:ring-offset-0 cursor-pointer"
          @change="toggleCheck(node)"
        />
      </label>

      <!-- Icon & Name -->
      <span class="text-xs shrink-0">{{ getFileIcon(node.name, node.is_dir) }}</span>
      <span class="truncate text-xs">{{ node.name }}</span>
    </div>

    <!-- Children -->
    <div v-if="node.is_dir && filteredChildren.length > 0 && (!isCollapsed(node.path) || filterText)">
      <FileTree
        v-for="child in filteredChildren"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :selected-path="selectedPath"
        :collapsed-state="collapsedState"
        :filter-text="filterText"
        @select="(p: string) => emit('select', p)"
        @toggle="onChildToggle"
        @context-action="(a: string, e?: string) => emit('context-action', a, e)"
      />
    </div>

    <!-- CodePack: 右键上下文菜单 -->
    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="fixed z-[100] min-w-[180px] py-1 bg-dark-800 border border-dark-600 rounded-lg shadow-xl"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      >
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('select-all')">
          全选
        </button>
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('select-none')">
          全不选
        </button>
        <div class="my-1 border-t border-dark-700" />
        <button
          v-if="contextMenu.ext"
          class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors"
          @click="onMenuAction('select-ext')"
        >
          只选 .{{ contextMenu.ext }} 文件
        </button>
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('select-source')">
          只选源码文件
        </button>
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('select-config')">
          只选配置文件
        </button>
        <button class="w-full px-3 py-1.5 text-xs text-left text-emerald-400 hover:bg-dark-700 transition-colors" @click="onMenuAction('select-git-changed')">
          只选 Git 变更文件
        </button>
        <div class="my-1 border-t border-dark-700" />
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('expand-all')">
          展开全部
        </button>
        <button class="w-full px-3 py-1.5 text-xs text-left text-dark-200 hover:bg-dark-700 transition-colors" @click="onMenuAction('collapse-all')">
          折叠全部
        </button>
      </div>
    </Teleport>
  </div>
</template>
