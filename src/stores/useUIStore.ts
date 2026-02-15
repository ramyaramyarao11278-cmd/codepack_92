import { defineStore } from "pinia";
import { ref } from "vue";
import type { ExportFormat } from "../types";

export const useUIStore = defineStore("ui", () => {
  const previewTab = ref<"file" | "export" | "stats">("file");
  const isDragging = ref(false);
  const showSettings = ref(false);
  const showPresetInput = ref(false);
  const newPresetName = ref("");
  const copySuccess = ref(false);
  const exportSuccess = ref(false);
  const exportFormat = ref<ExportFormat>("plain");
  const maxFileKB = ref(1024); // 1MB default, in KB
  const treeFilter = ref("");

  function resetPresetUI() {
    showPresetInput.value = false;
    newPresetName.value = "";
  }

  return {
    previewTab, isDragging, showSettings,
    showPresetInput, newPresetName,
    copySuccess, exportSuccess, exportFormat, maxFileKB, treeFilter,
    resetPresetUI,
  };
});
