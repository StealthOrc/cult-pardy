<script lang="ts">
	import { dev_loaded } from '$lib/stores/cookies';
	import '../app.css';
	import DevLoading from './DevLoading.svelte';
	import ToastPanel from '$lib/Toast/ToastPanel.svelte';

    let is_dev_loaded = false;
	dev_loaded.subscribe(value => {
            is_dev_loaded = value;
    })
</script>

<main class="cult-bg-gradient h-dvh w-dvw">
  {#if is_dev_loaded}
  	<ToastPanel/>
  	<slot/>
  {:else}
  	<DevLoading/>
  {/if}
</main>
