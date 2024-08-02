<script lang="ts">
	import { dev_loaded } from '$lib/stores/cookies';
	import { onMount } from 'svelte';
	import '../app.css';
	import DevLoading from './DevLoading.svelte';


    let is_dev_loaded = false;
	dev_loaded.subscribe(value => {
            is_dev_loaded = value;
    })
</script>


{#if is_dev_loaded}
	<slot></slot>
{:else}
	<DevLoading/>
{/if}

