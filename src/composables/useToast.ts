// CodePack: 全局 Toast 通知 composable
import { ref, readonly } from "vue";

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface ToastItem {
  id: number;
  type: "success" | "error" | "info";
  message: string;
  action?: ToastAction;
  visible: boolean;
}

const toasts = ref<ToastItem[]>([]);
let nextId = 0;

export function useToast() {
  function show(opts: {
    type: "success" | "error" | "info";
    message: string;
    action?: ToastAction;
    duration?: number;
  }) {
    const id = nextId++;
    const item: ToastItem = {
      id,
      type: opts.type,
      message: opts.message,
      action: opts.action,
      visible: true,
    };
    toasts.value.push(item);

    const duration = opts.duration ?? 3000;
    setTimeout(() => {
      dismiss(id);
    }, duration);
  }

  function dismiss(id: number) {
    const item = toasts.value.find((t) => t.id === id);
    if (item) {
      item.visible = false;
      // CodePack: 等待淡出动画完成后移除
      setTimeout(() => {
        toasts.value = toasts.value.filter((t) => t.id !== id);
      }, 300);
    }
  }

  return {
    toasts: readonly(toasts),
    show,
    dismiss,
  };
}
