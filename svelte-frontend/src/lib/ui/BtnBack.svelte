<script lang="ts">
	import type { ButtonProps } from '$lib/types';
    import { ArrowLeft } from 'lucide-svelte';
	import BtnBase from './BtnBase.svelte';
    
    let { onclick = () => {}, text = "Back"  }: ButtonProps = $props();	
</script>

{#snippet Icon()}
    <ArrowLeft class="mr-2"/>
{/snippet}

<BtnBase {onclick} {text} {Icon}/>