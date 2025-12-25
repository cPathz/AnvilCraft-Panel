
export type ToastType = "success" | "error" | "info" | "warning";

export interface Toast {
    id: string;
    type: ToastType;
    message: string;
    duration?: number;
}

class ToastStore {
    toasts = $state<Toast[]>([]);

    add(message: string, type: ToastType = "info", duration = 3000) {
        const id = crypto.randomUUID();
        const toast = { id, message, type, duration };
        this.toasts.push(toast);

        if (duration > 0) {
            setTimeout(() => {
                this.remove(id);
            }, duration);
        }
    }

    remove(id: string) {
        this.toasts = this.toasts.filter((t) => t.id !== id);
    }

    success(message: string, duration = 3000) {
        this.add(message, "success", duration);
    }

    error(message: string, duration = 4000) {
        this.add(message, "error", duration);
    }

    info(message: string, duration = 3000) {
        this.add(message, "info", duration);
    }

    warning(message: string, duration = 3000) {
        this.add(message, "warning", duration);
    }
}

export const toast = new ToastStore();
