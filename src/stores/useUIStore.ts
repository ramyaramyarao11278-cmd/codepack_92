import { defineStore } from "pinia";
import { ref } from "vue";

export const useUIStore = defineStore("ui", () => {
  const previewTab = ref<"file" | "export" | "stats">("file");
  const isDragging = ref(false);
  const showSettings = ref(false);
  const showPresetInput = ref(false);
  const newPresetName = ref("");
  const copySuccess = ref(false);
  const exportSuccess = ref(false);

  function resetPresetUI() {
    showPresetInput.value = false;
    newPresetName.value = "";
  }

  return {
    previewTab, isDragging, showSettings,
    showPresetInput, newPresetName,
    copySuccess, exportSuccess,
    resetPresetUI,
  };
});
