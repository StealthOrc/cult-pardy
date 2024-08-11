<script lang="ts">
  import { newToastStore, type Toast } from '$lib/stores/toastStore';
  import { onMount } from 'svelte';

  const toastStore = newToastStore();
  let toasts: Toast[] = [];

  onMount(() => {
    const unsubscribe = toastStore.subscribe(state => {
      toasts = state.toasts.slice(0, 3).reverse();
    });

    return () => {
      unsubscribe();
    };
  });

  function handleClick(toast: Toast) {
    toastStore.dismissToast(toast.id);
  }

  function currentProgress(toast: Toast) {
    return Math.max(0, (toast.timeout / toast.duration) * 100) + '%';
  }
</script>

<div class="fixed top-4 right-4 space-y-2 z-50">
  {#each toasts as toast (toast.id)}
    <div
      class={`relative p-3 rounded-lg shadow-lg overflow-hidden transition-transform 
      ${toast.position === 'top-right' ? 'right-4' : 'left-1/2 transform -translate-x-1/2'}
       ${toast.type === 'success' ? 'bg-green-500 text-white' : toast.type === 'info' ? 'bg-blue-600 text-white' : toast.type === 'warning' ? 'bg-yellow-400 text-gray-800' : 'bg-red-500 text-white'}
        ${toast.visible ? 'opacity-100' : 'opacity-0'} 
        ${toast.visible ? 'translate-y-0' : 'translate-y-4'} max-w-[300px] min-w-[200px]`}>
      <div class="flex items-center mb-2">
        <span class="mr-4 break-words">{toast.message}</span>
        <button class="text-gray-400 hover:text-white" aria-label="Close" on:click={() => handleClick(toast)}>
          &times;
        </button>
      </div>
      <div class="absolute bottom-0 left-0 h-1 bg-blue-400 rounded-b-lg transition-all" style="width: {currentProgress(toast)}"></div>
    </div>
  {/each}
</div>
