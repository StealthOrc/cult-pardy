<script lang="ts">
	import { dev_loaded } from '$lib/stores/cookies';
	import { onMount } from 'svelte';
	import '../app.css';
	import DevLoading from './DevLoading.svelte';


    let is_dev_loaded = false;
	dev_loaded.subscribe(value => {
            is_dev_loaded = value;
    })
	let loaded = false;


    onMount(async () => {
		while (!is_dev_loaded) {
			await new Promise(r => setTimeout(r, 250));
		}	
		loaded = true;
	})
</script>


{#if is_dev_loaded && loaded}
	<slot></slot>
{:else}
	<DevLoading/>
{/if}

