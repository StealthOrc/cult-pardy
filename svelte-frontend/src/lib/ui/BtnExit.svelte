<script lang="ts">
	import type { ButtonProps } from '$lib/types';
    import { DoorOpen } from 'lucide-svelte';
	import BtnBase from './BtnBase.svelte';
    
    let { onclick = () => {}, text = "Exit"  }: ButtonProps = $props();	

</script>

{#snippet Icon()}
    <DoorOpen class="mr-2"/>
{/snippet}

<BtnBase {onclick} {text} {Icon}/>