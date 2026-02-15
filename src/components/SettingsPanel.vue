<!-- CodePack: 设置面板（API 配置 + 插件管理） -->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import type { PluginDef, ApiConfig } from "../types";

const toast = useToast();
const activeTab = ref<"api" | "plugins">("api");

// ─── API Config ─────────────────────────────────────────────
const apiConfig = ref<ApiConfig>({
  provider: "deepseek",
  model: "deepseek-chat",
  api_key: "",
  base_url: "",
});
const showApiKey = ref(false);
const isSavingApi = ref(false);

const providers = [
  { value: "deepseek", label: "DeepSeek", hint: "便宜好用，推荐全库 Review" },
  { value: "openai", label: "OpenAI", hint: "GPT-4o / GPT-4o-mini" },
  { value: "anthropic", label: "Anthropic", hint: "Claude 3.5 Sonnet" },
  { value: "custom", label: "Custom URL", hint: "兼容 OpenAI 格式的自定义端点" },
];

const modelOptions = computed(() => {
  switch (apiConfig.value.provider) {
    case "deepseek": return ["deepseek-chat", "deepseek-reasoner"];
    case "openai": return ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "o1-mini"];
    case "anthropic": return ["claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022", "claude-3-haiku-20240307"];
    default: return [];
  }
});

const maskedKey = computed(() => {
  const k = apiConfig.value.api_key;
  if (!k || k.length < 8) return k;
  return k.substring(0, 6) + "•".repeat(Math.min(k.length - 10, 20)) + k.substring(k.length - 4);
});

async function loadApiConfig() {
  try {
    apiConfig.value = await invoke<ApiConfig>("load_api_config_cmd");
  } catch { /* use defaults */ }
}

async function saveApiConfig() {
  isSavingApi.value = true;
  try {
    await invoke("save_api_config_cmd", { config: apiConfig.value });
    toast.show({ type: "success", message: "API 配置已保存（存储在本地配置文件）" });
  } catch (e) {
    toast.show({ type: "error", message: `保存失败: ${e}` });
  } finally {
    isSavingApi.value = false;
  }
}

// ─── Plugins ────────────────────────────────────────────────
const plugins = ref<PluginDef[]>([]);
const isLoading = ref(false);
const showAddForm = ref(false);

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
  loadApiConfig();
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
    <div class="bg-dark-900 border border-dark-700 rounded-xl shadow-2xl w-[600px] max-h-[80vh] flex flex-col">
      <!-- Header with tabs -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-dark-700">
        <div class="flex items-center gap-4">
          <button
            class="text-sm font-medium transition-colors pb-0.5"
            :class="activeTab === 'api' ? 'text-emerald-400 border-b border-emerald-400' : 'text-dark-400 hover:text-dark-200'"
            @click="activeTab = 'api'"
          >🔑 API 配置</button>
          <button
            class="text-sm font-medium transition-colors pb-0.5"
            :class="activeTab === 'plugins' ? 'text-emerald-400 border-b border-emerald-400' : 'text-dark-400 hover:text-dark-200'"
            @click="activeTab = 'plugins'"
          >🧩 插件管理</button>
        </div>
        <button
          class="text-dark-500 hover:text-dark-300 transition-colors text-xs"
          @click="emit('close')"
        >✕ 关闭</button>
      </div>

      <!-- API Config Tab -->
      <div v-if="activeTab === 'api'" class="flex-1 overflow-auto p-5 space-y-5">
        <!-- Provider -->
        <div>
          <label class="block text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">AI Provider</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="p in providers"
              :key="p.value"
              class="flex flex-col items-start px-3 py-2 rounded-lg border text-left transition-colors"
              :class="apiConfig.provider === p.value
                ? 'bg-emerald-400/10 border-emerald-400/30 text-emerald-400'
                : 'bg-dark-800 border-dark-600 text-dark-300 hover:border-dark-500'"
              @click="apiConfig.provider = p.value"
            >
              <span class="text-xs font-medium">{{ p.label }}</span>
              <span class="text-[10px] text-dark-500 mt-0.5">{{ p.hint }}</span>
            </button>
          </div>
        </div>

        <!-- Model -->
        <div>
          <label class="block text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">Model</label>
          <div v-if="modelOptions.length > 0" class="flex flex-wrap gap-1.5">
            <button
              v-for="m in modelOptions"
              :key="m"
              class="px-2.5 py-1 text-xs rounded-md border transition-colors"
              :class="apiConfig.model === m
                ? 'bg-emerald-400/10 border-emerald-400/30 text-emerald-400'
                : 'bg-dark-800 border-dark-600 text-dark-400 hover:text-dark-200'"
              @click="apiConfig.model = m"
            >{{ m }}</button>
          </div>
          <input
            v-else
            v-model="apiConfig.model"
            class="w-full px-3 py-2 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
            placeholder="模型名称，如 gpt-4o"
          />
        </div>

        <!-- API Key -->
        <div>
          <label class="block text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">API Key</label>
          <div class="relative">
            <input
              v-model="apiConfig.api_key"
              :type="showApiKey ? 'text' : 'password'"
              class="w-full px-3 py-2 pr-20 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50 font-mono"
              :placeholder="apiConfig.provider === 'deepseek' ? 'sk-...' : apiConfig.provider === 'anthropic' ? 'sk-ant-...' : 'sk-...'"
            />
            <button
              class="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] text-dark-500 hover:text-dark-300 transition-colors"
              @click="showApiKey = !showApiKey"
            >{{ showApiKey ? '隐藏' : '显示' }}</button>
          </div>
          <p class="text-[10px] text-dark-600 mt-1.5">🔒 API Key 仅存储在本地配置文件，不会上传到任何服务器。API 调用直接从你的设备发送到 AI 服务商。</p>
        </div>

        <!-- Custom Base URL -->
        <div v-if="apiConfig.provider === 'custom'">
          <label class="block text-xs text-dark-400 font-medium uppercase tracking-wider mb-2">Base URL</label>
          <input
            v-model="apiConfig.base_url"
            class="w-full px-3 py-2 text-xs bg-dark-800 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50 font-mono"
            placeholder="https://your-api.example.com/v1"
          />
          <p class="text-[10px] text-dark-600 mt-1">需兼容 OpenAI Chat Completions 格式（/chat/completions 端点）</p>
        </div>

        <!-- Save -->
        <button
          class="w-full py-2.5 text-xs font-medium rounded-lg transition-colors"
          :class="isSavingApi
            ? 'bg-dark-700 text-dark-500 cursor-wait'
            : 'bg-emerald-400/15 text-emerald-400 hover:bg-emerald-400/25'"
          :disabled="isSavingApi"
          @click="saveApiConfig"
        >{{ isSavingApi ? '保存中...' : '💾 保存 API 配置' }}</button>
      </div>

      <!-- Plugins Tab -->
      <div v-if="activeTab === 'plugins'" class="flex-1 overflow-auto p-5 space-y-4">
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
            <label class="block text-xs text-dark-500 mb-1">检测文件（逗号分隔）</label>
            <input
              v-model="formDetectFiles"
              class="w-full px-2.5 py-1.5 text-xs bg-dark-900 border border-dark-600 rounded-md text-dark-200 placeholder-dark-500 focus:outline-none focus:border-emerald-400/50"
              placeholder="如: ProjectSettings/ProjectVersion.txt"
            />
          </div>
          <div>
            <label class="block text-xs text-dark-500 mb-1">检测目录（逗号分隔）</label>
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

        <button
          v-if="!showAddForm"
          class="w-full py-2 text-xs text-dark-500 hover:text-emerald-400 border border-dashed border-dark-600 hover:border-emerald-400/30 rounded-lg transition-colors"
          @click="showAddForm = true"
        >+ 添加插件</button>

        <div class="text-xs text-dark-600 leading-relaxed">
          <p>插件以 JSON 文件保存在系统配置目录的 <code class="text-dark-500">codepack/plugins/</code> 下。</p>
          <p class="mt-1">当项目根目录同时满足「检测文件」和「检测目录」条件时，该插件定义的项目类型将优先于内置规则。</p>
        </div>
      </div>
    </div>
  </div>
</template>
