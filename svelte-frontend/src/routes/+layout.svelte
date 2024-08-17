<script lang="ts">
	import { showToast, ToastType } from '$lib/stores/toastStore';
	import { dev_loaded } from '$lib/stores/cookies';
	import { onMount } from 'svelte';
	import '../app.css';
	import DevLoading from './DevLoading.svelte';
	import ToastPanel from '$lib/Toast/ToastPanel.svelte';

    let is_dev_loaded = false;
	dev_loaded.subscribe(value => {
            is_dev_loaded = value;
    })

	function triggerToast() {
		//Get a random number between 1 and 4
		const random = Math.floor(Math.random() * 4) + 1;
		const pos = Math.floor(Math.random() * 2) + 1;
		const posi = pos === 1 ? 'top-right': 'top-middle'
		const type : ToastType = random === 1 ? ToastType.SUCCESS : random === 2 ? ToastType.INFO : random === 3 ? ToastType.WARNING : ToastType.ERROR;
		showToast(`This is a toast message!`, type, 2000, posi);
  }



</script>

{#if is_dev_loaded}
	<ToastPanel/>
	<slot>
		
	</slot>
{:else}
	<DevLoading/>
{/if}
