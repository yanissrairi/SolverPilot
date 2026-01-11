export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
  actions?: ToastAction[];
}

class ToastStore {
  toasts = $state<Toast[]>([]);

  add(type: ToastType, message: string, duration = 5000, actions?: ToastAction[]) {
    const id = crypto.randomUUID();
    const toast: Toast = { id, type, message, duration, actions };
    this.toasts.push(toast);

    // Don't auto-dismiss if there are actions
    if (duration > 0 && !actions) {
      setTimeout(() => {
        this.remove(id);
      }, duration);
    }
  }

  remove(id: string) {
    this.toasts = this.toasts.filter(t => t.id !== id);
  }

  success(message: string, duration?: number) {
    this.add('success', message, duration);
  }

  error(message: string, duration?: number) {
    this.add('error', message, duration);
  }

  info(message: string, duration?: number) {
    this.add('info', message, duration);
  }

  warning(message: string, duration?: number, actions?: ToastAction[]) {
    this.add('warning', message, duration, actions);
  }
}

export const toast = new ToastStore();
