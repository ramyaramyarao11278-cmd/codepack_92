<!-- CodePack: 插件管理面板 -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import type { PluginDef } from "../types";

const toast = useToast();

const plugins = ref<PluginDef[]>([]);
const isLoading = ref(false);
const showAddForm = ref(false);

// 新插件表单
const form = ref<PluginDef>({
  name: "",
  version: "1.0",
  detect_files: [],
  detect_dirs: [],
  exclude_dirs: [],
  source_extensions: [],
});
const formDetectFiles = ref("");
const formDetectDirs = ref("");
const formExcludeDirs = ref("");
const formSourceExts = ref("");

const emit = defineEmits<{
  (e: "close"): void;
}>();

onMounted(() => {
  loadPlugins();
});

async function loadPlugins() {
  isLoading.value = true;
  try {
    plugins.value = await invoke<PluginDef[]>("list_plugins");
  } catch {
    plugins.value = [];
  } finally {
    isLoading.value = false;
  }
}

function resetForm() {
  form.value = {
    name: "",
    version: "1.0",
    detect_files: [],
    detect_dirs: [],
    exclude_dirs: [],
    source_extensions: [],
  };
  formDetectFiles.value = "";
  formDetectDirs.value = "";
  formExcludeDirs.value = "";
  formSourceExts.value = "";
}

function parseCommaList(str: string): string[] {
  return str
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

async function onSavePlugin() {
  if (!form.value.name.trim()) {
    toast.show({ type: "info", message: "请输入插件名称" });
    return;
  }
  form.value.detect_files = parseCommaList(formDetectFiles.value);
  form.value.detect_dirs = parseCommaList(formDetectDirs.value);
  form.value.exclude_dirs = parseCommaList(formExcludeDirs.value);
  form.value.source_extensions = parseCommaList(formSourceExts.value);

  if (form.value.detect_files.length === 0 && form.value.detect_dirs.length === 0) {
    toast.show({ type: "info", message: "至少需要一个检测文件或检测目录" });
    return;
  }

  try {
    await invoke("save_plugin", { plugin: form.value });
    toast.show({ type: "success", message: `插件「${form.value.name}」已保存` });
    showAddForm.value = false;
    resetForm();
    await loadPlugins();
  } catch (e) {
    toast.show({ type: "error", message: `保存失败: ${e}` });
  }
}

async function onDeletePlugin(name: string) {
  try {
    await invoke("delete_plugin", { name });
    toast.show({ type: "success", message: `插件「${name}」已删除` });
    await loadPlugins();
  } catch (e) {
    toast.show({ type: "error", message: `删除失败: ${e}` });
  }
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
    <div class="bg-dark-900 border border-dark-700 rounded-xl shadow-2xl w-[560px] max-h-[80vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-dark-700">
        <div class="text-sm font-semibold text-dark-100">插件管理</div>
        <button
          class="text-dark-500 hover:text-dark-300 transition-colors text-xs"
          @click="emit('close')"
        >✕ 关闭</button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-auto p-5 space-y-4">
        <!-- 已安装插件列表 -->
        <div>
          <div class="text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">已安装插件</div>
          <div v-if="isLoading" class="text-xs text-dark-500">加载中...</div>
          <div v-else-if="plugins.length === 0" class="text-xs text-dark-500">
            暂无插件。插件可以自定义项目类型识别规则、排除目录和源码扩展名。
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="plugin in plugins"
              :key="plugin.name"
              class="bg-dark-800 rounded-lg p-3 group"
            >
              <div class="flex items-center justify-between mb-1.5">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-dark-100">{{ plugin.name }}</span>
                  <span v-if="plugin.version" class="text-xs text-dark-500">v{{ plugin.version }}</span>
                </div>
                <button
                  class="text-xs text-dark-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-all"
                  @click="onDeletePlugin(plugin.name)"
                >删除</button>
              </div>
              <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-dark-400">
                <span v-if="plugin.detect_files.length > 0">
                  <span class="text-dark-500">检测文件:</span> {{ plugin.detect_files.join(', ') }}
                </span>
                <span v-if="plugin.detect_dirs.length > 0">
                  <span class="text-dark-500">检测目录:</span> {{ plugin.detect_dirs.join(', ') }}
                </span>
                <span v-if="plugin.exclude_dirs.length > 0">
                  <span class="text-dark-500">排除:</span> {{ plugin.exclude_dirs.join(', ') }}
                </span>
                <span v-if="plugin.source_extensions.length > 0">
                  <span class="text-dark-500">扩展名:</span> .{{ plugin.source_extensions.join(', .') }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 添加插件表单 -->
        <div v-if="showAddForm" class="bg-dark-800 rounded-lg p-4 space-y-3">
          <div class="text-xs text-dark-400 font-medium uppercase tracking-wider">新建插件</div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs text-dark-500 mb-1">项目类型名称 *</label>
              <input
                v-model="form.name"
                class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
                placeholder="如: Unity"
              />
            </div>
            <div>
              <label class="block text-xs text-dark-500 mb-1">版本</label>
              <input
                v-model="form.version"
                class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
                placeholder="1.0"
              />
            </div>
          </div>
          <div>
            <label class="block text-xs text-dark-500 mb-1">检测文件（逗号分隔，项目根目录下存在这些文件则匹配）</label>
            <input
              v-model="formDetectFiles"
              class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
              placeholder="如: ProjectSettings/ProjectVersion.txt"
            />
          </div>
          <div>
            <label class="block text-xs text-dark-500 mb-1">检测目录（逗号分隔，项目根目录下存在这些目录则匹配）</label>
            <input
              v-model="formDetectDirs"
              class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
              placeholder="如: Assets, Packages"
            />
          </div>
          <div>
            <label class="block text-xs text-dark-500 mb-1">额外排除目录（逗号分隔）</label>
            <input
              v-model="formExcludeDirs"
              class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
              placeholder="如: Library, Temp, Logs"
            />
          </div>
          <div>
            <label class="block text-xs text-dark-500 mb-1">额外源码扩展名（逗号分隔，不含点号）</label>
            <input
              v-model="formSourceExts"
              class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
              placeholder="如: cs, shader, compute, asmdef"
            />
          </div>
          <div class="flex items-center gap-2 pt-1">
            <button
              class="px-3 py-1.5 text-xs bg-emerald-400/15 text-emerald-400 rounded-md hover:bg-emerald-400/25 transition-colors"
              @click="onSavePlugin"
            >保存插件</button>
            <button
              class="px-3 py-1.5 text-xs text-dark-500 hover:text-dark-300 transition-colors"
              @click="showAddForm = false; resetForm()"
            >取消</button>
          </div>
        </div>

        <!-- 添加按钮 -->
        <button
          v-if="!showAddForm"
          class="w-full py-2 text-xs text-dark-500 hover:text-emerald-400 border border-dashed border-dark-600 hover:border-emerald-400/30 rounded-lg transition-colors"
          @click="showAddForm = true"
        >+ 添加插件</button>

        <!-- 说明 -->
        <div class="text-xs text-dark-600 leading-relaxed">
          <p>插件以 JSON 文件保存在系统配置目录的 <code class="text-dark-500">codepack/plugins/</code> 下。</p>
          <p class="mt-1">当项目根目录同时满足「检测文件」和「检测目录」条件时，该插件定义的项目类型将优先于内置规则。</p>
        </div>
      </div>
    </div>
  </div>
</template>
