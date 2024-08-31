
import { dev } from '$app/environment';
import { writable } from 'svelte/store';

let toastStore: ReturnType<typeof createToastStore> | null = null;

export function newToastStore(): ReturnType<typeof createToastStore> {
    if (toastStore != null) {
      return toastStore;
    }
    console.log("Creating new toast store");
    toastStore = createToastStore();
    return toastStore;
  }
  export interface Toast {
    id: number;
    message: string;
    duration: number;
    type: ToastType;
    visible: boolean;
    timeout: number;
}



export enum ToastType {
    SUCCESS = "success",
    INFO = "info",
    WARNING = "warning",
    ERROR = "error",
    LOADING = "loading"
}

interface ToastState {
    timer: boolean
    timerTask : NodeJS.Timeout | null;
    toasts: Toast[];
}
if(dev) {
    if (import.meta.hot) {
        import.meta.hot.accept((newModule ) => {
            if (newModule != undefined) {
                newModule.toastStore = newToastStore();
            }
        });
    }
}

function createToastStore() {
    const { subscribe, update } = writable<ToastState>({timer:false, timerTask: null,toasts: []});
    const addional_time = 100;

    function addToast(message: string, type : ToastType = ToastType.INFO ,duration: number = 3000) {
        const id = Math.floor(Math.random() * 1000000);
        const newToast : Toast  =  { id: id, message, type :type, duration, visible: true, timeout: duration };
        update(toast => {
            toast.toasts.push(newToast);
            if (!toast.timer) { 
                toast.timerTask = setInterval(startTimer, 100);
                toast.timer = true;
        }
        return toast;
      });
      
    }
  
    function startTimer(){
        update(toast => {
        if (toast == null) {
            return toast;
        }
        if (toast.toasts.length == 0) {
            clearInterval(toast.timerTask!);
            toast.timer = false;
            return toast
        }
        const current = toast.toasts[0];
        if (current.timeout <= 0 - addional_time) {
            toast.toasts.shift();
            if (toast.toasts.length == 0) {
                clearInterval(toast.timerTask!);
                toast.timer = false
            }
            return toast;
        } 
        current.timeout -= 100;
        if (toast.toasts[0].id == current.id) toast.toasts[0] = current;
        return toast; 
        });
    }

  
    function dismissToast(id: number) {
      update(state => {
        state.toasts = state.toasts.filter(t => t.id !== id);
        if (state.toasts.length == 0) {
            clearInterval(state.timerTask!);
            state.timer = false
        }
        return state;
      });
    }
  
    return {
      subscribe,
      addToast,
      dismissToast,
    };
  }

export function showToast(message: string, type : ToastType = ToastType.INFO ,duration: number = 3000) {
  newToastStore().addToast(message, type, duration);
}

export function ToastInfo(message: string, duration: number = 3000) {
    newToastStore().addToast(message, ToastType.INFO ,duration);
}
export function ToastSucces(message: string, duration: number = 3000) {
    newToastStore().addToast(message, ToastType.SUCCESS ,duration);
}

export function ToastWarning(message: string, duration: number = 3000) {
    newToastStore().addToast(message, ToastType.WARNING ,duration);
}

export function ToastError(message: string, duration: number = 3000) {
    newToastStore().addToast(message, ToastType.ERROR ,duration);
}



