<script lang="ts">
  import { newToastStore, showToast, ToastType, type Toast } from '$lib/stores/toastStore';
  import { onMount } from 'svelte';
  import { X } from 'lucide-svelte';

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

	function triggerToast() {
		//Get a random number between 1 and 4
		const random = Math.floor(Math.random() * 4) + 1;
		const type : ToastType = random === 1 ? ToastType.SUCCESS : random === 2 ? ToastType.INFO : random === 3 ? ToastType.WARNING : ToastType.ERROR;
		showToast(`This is a toast message!`, type, 2000);
  }




</script>
<button class="absolute bottom-0 right-0 m-2 z-50 cult-btn-menu" on:click={triggerToast}>Show Toast</button>
<div class="absolute inset-x-0 top-0 flex flex-col space-y-2 my-2 mx-2 z-50 justify-center">
  {#each toasts as toast (toast.id)}
    <div
      class={`relative p-3 rounded-lg shadow-lg overflow-hidden transition-transform 
       ${toast.type === 'success' ? 'bg-green-500 text-white' : toast.type === 'info' ? 'bg-blue-600 text-white' : toast.type === 'warning' ? 'bg-yellow-400 text-gray-800' : 'bg-red-500 text-white'}
        ${toast.visible ? 'opacity-100' : 'opacity-0'} 
        ${toast.visible ? 'translate-y-0' : 'translate-y-4'} max-w-[300px] min-w-[200px]`}>
      <div class="flex items-center mb-2">
        <span class="mr-4 break-words align-middle">{toast.message}</span>
        <button class="text-black hover:text-white" aria-label="Close" on:click={() => handleClick(toast)}>
          <X/>
        </button>
      </div>
      <div class="absolute bottom-0 left-0 h-1 bg-blue-400 rounded-b-lg transition-all" style="width: {currentProgress(toast)}"></div>
    </div>
  {/each}
</div>
